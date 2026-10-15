#!/usr/bin/env python3
"""Compile and run Rust tests in separate systemd cgroups with enforced limits."""

import argparse
import collections
import datetime
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parent.parent
MIB = 1024 * 1024
MEMORY_LIMIT = 1024 * MIB
STOP_MEMORY = 768 * MIB


def group_path():
    for line in Path('/proc/self/cgroup').read_text().splitlines():
        if line.startswith('0::'):
            return Path('/sys/fs/cgroup') / line[3:].lstrip('/')
    raise RuntimeError('cgroup v2 is required; refusing unbounded execution')


def kill_process_group(process):
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def worker(report, index):
    request = json.loads((report / f'{index:02d}.request.json').read_text())
    group = group_path()
    if ((group / 'memory.max').read_text().strip() != str(MEMORY_LIMIT)
            or (group / 'memory.swap.max').read_text().strip() != '0'):
        raise RuntimeError('RAM/swap limits are not enforced; refusing execution')
    if request.get('supervise'):
        return supervise(report, index, request, group)
    samples = collections.deque()
    reason = None
    started = time.monotonic()
    stdin_file = open(request['stdin'], 'rb') if request.get('stdin') else None
    stdout_file = (report / f'{index:02d}.stdout.jsonl').open('wb') if request.get('stdout') else None
    with (report / f'{index:02d}.output.txt').open('w') as output:
        # Protocol stdout, when requested, is kept apart from diagnostics.
        process = subprocess.Popen(request['command'], cwd=ROOT,
                                   stdin=stdin_file if stdin_file else None,
                                   stdout=stdout_file if stdout_file else output,
                                   stderr=output if stdout_file else subprocess.STDOUT,
                                   start_new_session=True)
        try:
            while process.poll() is None:
                now = time.monotonic()
                peak = int((group / 'memory.peak').read_text())
                if now - started >= request['timeout']:
                    reason = 'timeout'
                elif peak >= STOP_MEMORY:
                    reason = 'near_memory_limit'
                rss = 0
                try:
                    for line in Path(f'/proc/{process.pid}/status').read_text().splitlines():
                        if line.startswith('VmRSS:'):
                            rss = int(line.split()[1]) * 1024
                            break
                except FileNotFoundError:
                    pass
                while samples and samples[0][0] < now - 1.0:
                    samples.popleft()
                if (request['stage'] in ('test', 'fixture', 'worker', 'python') and rss >= 128 * MIB and samples
                        and rss - min(sample[1] for sample in samples) >= 64 * MIB):
                    reason = reason or 'abnormal_memory_growth'
                samples.append((now, rss))
                if reason:
                    kill_process_group(process)
                    break
                time.sleep(0.02)
            status = process.wait(timeout=2)
        except BaseException:
            kill_process_group(process)
            process.wait(timeout=2)
            raise
        finally:
            for handle in (stdin_file, stdout_file):
                if handle:
                    handle.close()
    events = dict(line.split() for line in (group / 'memory.events').read_text().splitlines())
    peak_group = int((group / 'memory.peak').read_text())
    if peak_group >= STOP_MEMORY:
        reason = reason or 'near_memory_limit'
    if int(events.get('oom', 0)) or int(events.get('oom_kill', 0)):
        reason = reason or 'cgroup_oom'
    if status != 0:
        reason = reason or 'nonzero_exit'
    if request['stage'] == 'test' and reason is None:
        output = (report / f'{index:02d}.output.txt').read_text()
        if 'test result: ok. 1 passed;' not in output:
            reason = 'expected_exactly_one_passing_test'
    row = dict(name=request['name'], stage=request['stage'], exit_status=status,
               elapsed_seconds=time.monotonic() - started,
               peak_rss_bytes=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss * 1024,
               cgroup_peak_bytes=peak_group, memory_events=events, stop_reason=reason)
    (report / f'{index:02d}.result.json').write_text(json.dumps(row, indent=2) + '\n')
    return 0 if reason is None else 1


def supervise(report, index, request, group):
    """Interactive mode: the Rust worker and the Python controller run in this
    scope, joined by two pipes the supervisor creates. The controller gets
    the descriptors on its command line and never launches anything. Joint
    accounting: the cgroup peak covers both; RSS growth is watched on the sum
    of both processes; one timeout for the whole exchange. Any stop kills
    both process groups; no child survives."""
    spec = request['supervise']
    to_r, to_w = os.pipe()        # controller -> worker stdin
    from_r, from_w = os.pipe()    # worker stdout -> controller
    controller_command = [arg.replace('{to_worker}', str(to_w)).replace('{from_worker}', str(from_r))
                          for arg in spec['controller']]
    samples = collections.deque()
    reason = None
    started = time.monotonic()
    processes = {}
    with (report / f'{index:02d}.output.txt').open('w') as log, \
            (report / f'{index:02d}.worker-stderr.txt').open('w') as worker_err, \
            (report / f'{index:02d}.controller-stderr.txt').open('w') as controller_err:
        try:
            processes['worker'] = subprocess.Popen(spec['worker'], cwd=ROOT, stdin=to_r, stdout=from_w,
                                                   stderr=worker_err, start_new_session=True)
            processes['controller'] = subprocess.Popen(controller_command, cwd=ROOT, stdin=subprocess.DEVNULL,
                                                       stdout=subprocess.DEVNULL, stderr=controller_err,
                                                       pass_fds=(to_w, from_r), start_new_session=True)
            for fd in (to_r, to_w, from_r, from_w):
                os.close(fd)
            print(f'worker pid {processes["worker"].pid}; controller pid {processes["controller"].pid}',
                  file=log, flush=True)
            exits = {}
            while len(exits) < 2:
                now = time.monotonic()
                for name, process in processes.items():
                    if name not in exits and process.poll() is not None:
                        exits[name] = process.returncode
                        print(f'{name} exited {process.returncode} at {now - started:.3f}s', file=log, flush=True)
                if 'worker' in exits and 'controller' not in exits:
                    reason = reason or 'worker_lost'
                peak = int((group / 'memory.peak').read_text())
                if now - started >= request['timeout']:
                    reason = reason or 'timeout'
                elif peak >= STOP_MEMORY:
                    reason = reason or 'near_memory_limit'
                rss = 0
                for name, process in processes.items():
                    if name in exits:
                        continue
                    try:
                        for line in Path(f'/proc/{process.pid}/status').read_text().splitlines():
                            if line.startswith('VmRSS:'):
                                rss += int(line.split()[1]) * 1024
                                break
                    except FileNotFoundError:
                        pass
                while samples and samples[0][0] < now - 1.0:
                    samples.popleft()
                if rss >= 128 * MIB and samples and rss - min(sample[1] for sample in samples) >= 64 * MIB:
                    reason = reason or 'abnormal_memory_growth'
                samples.append((now, rss))
                if reason:
                    break
                if 'controller' in exits and 'worker' not in exits:
                    # The controller closed its channels; the worker sees EOF and
                    # should exit on its own within a short grace period.
                    try:
                        processes['worker'].wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        reason = 'worker_did_not_exit'
                        break
                    continue
                time.sleep(0.02)
        finally:
            for name, process in processes.items():
                kill_process_group(process)
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    pass
            for name, process in processes.items():
                print(f'{name} final status {process.returncode}', file=log, flush=True)
    events = dict(line.split() for line in (group / 'memory.events').read_text().splitlines())
    peak_group = int((group / 'memory.peak').read_text())
    if peak_group >= STOP_MEMORY:
        reason = reason or 'near_memory_limit'
    if int(events.get('oom', 0)) or int(events.get('oom_kill', 0)):
        reason = reason or 'cgroup_oom'
    controller_status = processes['controller'].returncode if 'controller' in processes else None
    worker_status = processes['worker'].returncode if 'worker' in processes else None
    if controller_status != 0:
        reason = reason or 'controller_nonzero_exit'
    if worker_status != 0:
        reason = reason or 'worker_nonzero_exit'
    row = dict(name=request['name'], stage=request['stage'], exit_status=controller_status,
               worker_exit_status=worker_status, elapsed_seconds=time.monotonic() - started,
               peak_rss_bytes=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss * 1024,
               cgroup_peak_bytes=peak_group, memory_events=events, stop_reason=reason)
    (report / f'{index:02d}.result.json').write_text(json.dumps(row, indent=2) + '\n')
    return 0 if reason is None else 1


def run_bounded(report, rows, stage, name, command, timeout, diagnostics=None,
                stdin=None, stdout=False, supervise_spec=None):
    index = len(rows) + 1
    request = dict(stage=stage, name=name, command=command, timeout=timeout,
                   stdin=str(stdin) if stdin else None, stdout=bool(stdout),
                   supervise=supervise_spec)
    (report / f'{index:02d}.request.json').write_text(json.dumps(request, indent=2) + '\n')
    unit = f'residuum-safe-{os.getpid()}-{index}'
    scope = ['/usr/bin/systemd-run', '--user', '--scope', '--quiet', f'--unit={unit}',
             '--property=MemoryMax=1G', '--property=MemorySwapMax=0',
             '--property=CPUQuota=100%', sys.executable, str(Path(__file__).resolve()),
             '--worker', str(report), str(index)]
    with (report / f'{index:02d}.scope.txt').open('w') as output:
        try:
            completed = subprocess.run(scope, stdout=output, stderr=subprocess.STDOUT,
                                       timeout=timeout + 10)
        except subprocess.TimeoutExpired:
            subprocess.run(['/usr/bin/systemctl', '--user', 'kill', '--signal=SIGKILL',
                            '--kill-whom=all', f'{unit}.scope'], timeout=5, check=False,
                           stdout=output, stderr=subprocess.STDOUT)
            raise RuntimeError(f'{name}: scope timeout; no remaining tests executed')
    result_path = report / f'{index:02d}.result.json'
    if not result_path.exists():
        raise RuntimeError(f'{name}: isolation failed; see {report / f"{index:02d}.scope.txt"}')
    row = json.loads(result_path.read_text())
    if completed.returncode != 0:
        row['stop_reason'] = row['stop_reason'] or 'scope_nonzero_exit'
    rows.append(row)
    save_report(report, rows)
    print(f'{name} | {row["exit_status"]} | {row["elapsed_seconds"]:.4f}s | '
          f'{row["peak_rss_bytes"] / MIB:.2f} MiB RSS', flush=True, file=diagnostics)
    if row['stop_reason']:
        raise RuntimeError(f'{name}: {row["stop_reason"]}; no remaining tests executed. '
                           f'See {report / f"{index:02d}.output.txt"}')
    return report / f'{index:02d}.output.txt'


def build_worker_binary(report, rows):
    """Compile residuum-worker with the existing cargo controls (own scope, 45 s)."""
    output = run_bounded(report, rows, 'compile', 'compile residuum-worker',
                         ['cargo', 'build', '--offline', '--locked', '-p', 'residuum-boundary',
                          '--bin', 'residuum-worker', '--jobs', '1', '--message-format=json'], 45)
    binaries = []
    for line in output.read_text().splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (message.get('reason') == 'compiler-artifact' and message.get('executable')
                and message.get('target', {}).get('name') == 'residuum-worker'
                and 'bin' in message.get('target', {}).get('kind', [])):
            binaries.append(Path(message['executable']))
    if len(binaries) != 1:
        raise RuntimeError('Expected one compiled residuum-worker binary; refusing execution')
    return binaries[0]


def save_report(report, rows):
    (report / 'results.json').write_text(json.dumps(rows, indent=2) + '\n')
    lines = ['| Nombre | Exit status | Tiempo (s) | Pico RSS (MiB) | Resultado |',
             '|---|---:|---:|---:|---|']
    for row in rows:
        lines.append(f'| {row["name"]} | {row["exit_status"]} | {row["elapsed_seconds"]:.4f} | '
                     f'{row["peak_rss_bytes"] / MIB:.2f} | {row["stop_reason"] or "passed"} |')
    (report / 'report.md').write_text('\n'.join(lines) + '\n')


def parse_names(output):
    return {line[:-6] for line in output.read_text().splitlines() if line.endswith(': test')}


def emit_output(path, stream):
    # Rendering happens inside the limited worker. Do not load a potentially
    # large result into the launcher process outside that cgroup.
    with path.open(encoding='utf-8') as output:
        while chunk := output.read(64 * 1024):
            stream.write(chunk)
    stream.flush()


def main():
    if len(sys.argv) == 4 and sys.argv[1] == '--worker':
        return worker(Path(sys.argv[2]), int(sys.argv[3]))
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--test', action='append', default=[], help='Exact active test name; repeatable')
    parser.add_argument('--test-target', choices=('lib', 'cli', 'boundary'), default=None,
                        help='Compile only the selected test target (default: lib); '
                             'boundary selects the residuum-boundary crate')
    parser.add_argument('--binary', type=Path, help='Use an existing binary without compiling')
    parser.add_argument('--compile-only', action='store_true')
    parser.add_argument('--fmt-check', action='store_true')
    parser.add_argument('--format', action='store_true')
    parser.add_argument('--fixture', type=Path, help='Run one source-defined Formal fixture with the bounded example runner')
    parser.add_argument('--cli', choices=('run', 'query', 'explain'), help='Run one CLI command on --fixture under the same limits')
    parser.add_argument('--steps', type=int, help='Bounded stepwise CLI transitions (1..4096)')
    parser.add_argument('--trace', action='store_true', help='Show every stepwise CLI transition')
    parser.add_argument('--incremental', action='store_true', help='Forward incremental comparison of base/delta seed blocks; requires --fixture')
    parser.add_argument('--forward', action='store_true', help='Module-aware forward closure with active verb programs; requires --fixture')
    parser.add_argument('--library', type=Path, help='Compose one shared declaration fixture; requires --fixture and --incremental')
    parser.add_argument('--worker-manifest', type=Path,
                        help='Run residuum-worker once, bounded, with this manifest; requires --worker-requests')
    parser.add_argument('--worker-requests', type=Path,
                        help='Finite file of one request per line fed to residuum-worker on stdin')
    parser.add_argument('--python-test', action='append', default=[],
                        help='Exact unittest id under tools/, run individually with python3 -I; repeatable')
    parser.add_argument('--interactive-manifest', type=Path,
                        help='Episode manifest (worker and controller sections) for one supervised '
                             'controller/worker exchange; requires --interactive-ledger and --interactive-policy')
    parser.add_argument('--interactive-ledger', type=Path,
                        help='Empty or absent directory under target/ for the episode ledger')
    parser.add_argument('--interactive-policy', help='Scripted policy name of the simulated provider')
    args = parser.parse_args()
    if (args.steps is not None or args.trace) and not args.cli:
        parser.error('--steps and --trace require --cli')
    if args.steps is not None and not 1 <= args.steps <= 4096:
        parser.error('--steps must be in 1..4096')
    if args.test_target is not None and (args.fixture or args.fmt_check or args.format):
        parser.error('--test-target is only for test compilation/execution')
    if args.cli and (not args.fixture or args.forward or args.incremental or args.library):
        parser.error('--cli requires --fixture and cannot be combined with other fixture modes')
    if args.incremental and not args.fixture:
        parser.error('--incremental requires --fixture')
    if args.forward and not args.fixture:
        parser.error('--forward requires --fixture')
    if args.forward and args.incremental:
        parser.error('--forward and --incremental are separate experiment modes')
    if args.library and not (args.fixture and args.incremental):
        parser.error('--library requires --fixture and --incremental')
    if bool(args.worker_manifest) != bool(args.worker_requests):
        parser.error('--worker-manifest and --worker-requests go together')
    if args.worker_manifest and (args.fixture or args.test or args.compile_only or args.binary
                                 or args.test_target or args.fmt_check or args.format
                                 or args.cli or args.steps is not None or args.trace
                                 or args.incremental or args.forward or args.library):
        parser.error('the worker mode takes only --worker-manifest and --worker-requests; '
                     'it cannot be combined with fixtures, tests, formatting or CLI options')
    if args.python_test and (args.fixture or args.test or args.compile_only or args.binary
                             or args.test_target or args.fmt_check or args.format or args.cli
                             or args.steps is not None or args.trace or args.incremental
                             or args.forward or args.library or args.worker_manifest):
        parser.error('--python-test takes only unittest ids; it cannot be combined with other modes')
    for name in args.python_test:
        if not name.startswith('tools.') or not all(part.isidentifier() for part in name.split('.')):
            parser.error(f'{name} is not a dotted unittest id under tools.')
    interactive = (args.interactive_manifest, args.interactive_ledger, args.interactive_policy)
    if any(interactive) and not all(interactive):
        parser.error('--interactive-manifest, --interactive-ledger and --interactive-policy go together')
    if all(interactive) and (args.fixture or args.test or args.compile_only or args.binary or args.test_target
                             or args.fmt_check or args.format or args.cli or args.steps is not None or args.trace
                             or args.incremental or args.forward or args.library or args.worker_manifest
                             or args.python_test):
        parser.error('the interactive mode takes only its three options')
    if all(interactive) and not args.interactive_policy.isidentifier():
        parser.error('--interactive-policy must be an identifier')
    if args.test_target is None:
        args.test_target = 'lib'
    if args.fixture and (args.test or args.binary or args.compile_only or args.fmt_check or args.format):
        parser.error('--fixture cannot be combined with test, binary, compile, or format options')
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    report = ROOT / 'target' / 'test-diagnostics' / f'{stamp}-{os.getpid()}'
    report.mkdir(parents=True)
    diagnostics = sys.stderr if args.cli else sys.stdout
    print(f'Reports: {report}', flush=True, file=diagnostics)
    rows = []
    if args.fixture:
        fixture = args.fixture.resolve()
        if not fixture.is_file() or fixture.stat().st_size > 64 * 1024:
            raise RuntimeError('Expected one regular fixture file of at most 64 KiB')
        library = args.library.resolve() if args.library else None
        if library and (not library.is_file() or library.stat().st_size + fixture.stat().st_size > 64 * 1024):
            raise RuntimeError('Expected one regular library; combined fixtures must be at most 64 KiB')
        target = 'residuum' if args.cli else 'run_fixture'
        target_kind = 'bin' if args.cli else 'example'
        output = run_bounded(report, rows, 'compile', f'compile {target}',
                             ['cargo', 'build', '--offline', '--locked', f'--{target_kind}', target,
                              '--jobs', '1', '--message-format=json'], 45, diagnostics=diagnostics)
        binaries = []
        for line in output.read_text().splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if (message.get('reason') == 'compiler-artifact' and message.get('executable')
                    and message.get('target', {}).get('name') == target
                    and target_kind in message.get('target', {}).get('kind', [])):
                binaries.append(Path(message['executable']))
        if len(binaries) != 1:
            raise RuntimeError('Expected one compiled runner binary; refusing execution')
        try:
            output = run_bounded(report, rows, 'fixture', str(fixture.relative_to(ROOT))
                                 if fixture.is_relative_to(ROOT) else fixture.name,
                                 [str(binaries[0])] + ([args.cli] if args.cli else [])
                                 + (['--incremental'] if args.incremental else [])
                                 + (['--forward'] if args.forward else []) + [str(fixture)]
                                 + (['--library', str(library)] if library else [])
                                 + (['--steps', str(args.steps)] if args.steps is not None else [])
                                 + (['--trace'] if args.trace else []), 15,
                                 diagnostics=diagnostics)
        except RuntimeError:
            # Expose semantic errors to CLI callers as well as keeping the report.
            if args.cli and rows and rows[-1]['stage'] == 'fixture':
                emit_output(report / f'{len(rows):02d}.output.txt', sys.stderr)
            raise
        emit_output(output, sys.stdout)
        return 0
    if args.interactive_manifest:
        manifest = args.interactive_manifest.resolve()
        ledger_dir = args.interactive_ledger.resolve()
        if not manifest.is_file() or manifest.stat().st_size > 4 * 1024 * 1024:
            raise RuntimeError('Expected one regular episode manifest of at most 4 MiB')
        if not ledger_dir.is_relative_to(ROOT / 'target'):
            raise RuntimeError('The episode ledger must live under target/')
        if ledger_dir.exists() and any(ledger_dir.iterdir()):
            raise RuntimeError('The episode ledger directory must be empty or absent; ledgers are never reused')
        ledger_dir.mkdir(parents=True, exist_ok=True)
        # The worker receives only the worker section, written by the supervisor;
        # the controller validates the whole episode manifest itself.
        document = json.loads(manifest.read_text())
        if not isinstance(document, dict) or 'worker' not in document:
            raise RuntimeError('The episode manifest must contain a worker section')
        worker_manifest = ledger_dir / 'worker-manifest.json'
        worker_manifest.write_text(json.dumps(document['worker'], indent=2, ensure_ascii=False) + '\n')
        binary = build_worker_binary(report, rows)
        bootstrap = (f'import sys; sys.path.insert(0, {str(ROOT)!r}); '
                     f'from tools.normative_explorer import run_episode; '
                     f'sys.exit(run_episode.main(sys.argv[1:]))')
        spec = dict(
            worker=[str(binary), '--manifest', str(worker_manifest)],
            controller=[sys.executable, '-I', '-c', bootstrap, '--manifest', str(manifest), '--ledger',
                        str(ledger_dir), '--policy', args.interactive_policy, '--to-worker', '{to_worker}',
                        '--from-worker', '{from_worker}'],
        )
        # One scope for both processes; 15 s for the whole exchange, like a fixture run.
        run_bounded(report, rows, 'interactive', 'controller+worker', ['supervised'], 15, supervise_spec=spec)
        print(f'episode ledger: {ledger_dir}', flush=True)
        return 0
    if args.python_test:
        # Python unit tests of the controller, one process per test id, each in
        # its own scope with the same controls; isolated mode ignores the
        # environment and user site-packages so only the repository is imported.
        if len(args.python_test) != len(set(args.python_test)):
            raise RuntimeError('Duplicate python test selection')
        for name in args.python_test:
            # Isolated mode drops the working directory from sys.path; the
            # repository root is added explicitly and nothing else.
            prelude = f'import sys, unittest; sys.path.insert(0, {str(ROOT)!r}); '
            # A module or class identifier would run several tests; the
            # authorisation is for exactly one, counted before anything runs.
            count = run_bounded(report, rows, 'list', f'count {name}', [
                sys.executable, '-I', '-c',
                prelude + f'print(unittest.TestLoader().loadTestsFromName({name!r}).countTestCases())'], 15)
            counted = count.read_text().strip().splitlines()
            if len(counted) != 1 or counted[0] != '1':
                raise RuntimeError(f'{name}: selects {counted[-1] if counted else "no"} tests; exactly one is required')
            output = run_bounded(report, rows, 'python', name, [
                sys.executable, '-I', '-c',
                prelude + f'unittest.main(module=None, argv=["unittest", "-v", {name!r}])'], 30)
            text = output.read_text()
            if 'Ran 1 test' not in text or 'skipped' in text or not text.rstrip().endswith('OK'):
                raise RuntimeError(f'{name}: expected exactly one passing, unskipped test')
        print(f'{len(args.python_test)} python tests passed individually.', flush=True)
        return 0
    if args.worker_manifest:
        manifest = args.worker_manifest.resolve()
        requests = args.worker_requests.resolve()
        if not manifest.is_file() or manifest.stat().st_size > 4 * 1024 * 1024:
            raise RuntimeError('Expected one regular manifest file of at most 4 MiB')
        if not requests.is_file() or requests.stat().st_size > 1024 * 1024:
            raise RuntimeError('Expected one regular requests file of at most 1 MiB')
        if not requests.read_bytes().endswith(b'\n'):
            raise RuntimeError('The requests file must end with a newline; a trailing partial line is not executed')
        binary = build_worker_binary(report, rows)
        # One bounded process in its own scope with the same controls as a fixture
        # run: requests on stdin, protocol lines to NN.stdout.jsonl, diagnostics to
        # NN.output.txt, 15 s like fixtures, memory growth and stop controls.
        run_bounded(report, rows, 'worker', 'residuum-worker',
                    [str(binary), '--manifest', str(manifest)], 15,
                    stdin=requests, stdout=True)
        print(f'worker protocol output: {report / f"{len(rows):02d}.stdout.jsonl"}', flush=True)
        return 0
    if args.fmt_check or args.format:
        command = ['cargo', 'fmt', '--all'] + (['--check'] if args.fmt_check else [])
        run_bounded(report, rows, 'format', 'cargo fmt', command, 30)
        return 0
    binary = args.binary.resolve() if args.binary else None
    if binary is None:
        if args.test_target == 'lib':
            target_args, target_kind, target_name = ['--lib'], 'lib', 'residuum'
        elif args.test_target == 'boundary':
            target_args, target_kind, target_name = (['-p', 'residuum-boundary', '--lib'], 'lib',
                                                     'residuum_boundary')
        else:
            target_args, target_kind, target_name = ['--bin', 'residuum'], 'bin', 'residuum'
        output = run_bounded(report, rows, 'compile', 'compile only',
                             ['cargo', 'test', '--offline', '--locked', '--no-run'] + target_args + [
                              '--jobs', '1', '--message-format=json'], 45)
        binaries = []
        for line in output.read_text().splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if (message.get('reason') == 'compiler-artifact' and message.get('executable')
                    and message.get('profile', {}).get('test')
                    and message.get('target', {}).get('name') == target_name
                    and target_kind in message.get('target', {}).get('kind', [])):
                binaries.append(Path(message['executable']))
        if len(binaries) != 1:
            raise RuntimeError('Expected one compiled test binary for the selected target; refusing execution')
        binary = binaries[0]
    if args.compile_only:
        return 0
    listed = run_bounded(report, rows, 'list', 'list tests',
                         [str(binary), '--list', '--format=terse'], 15)
    ignored = run_bounded(report, rows, 'list', 'list ignored tests',
                          [str(binary), '--list', '--ignored', '--format=terse'], 15)
    active = parse_names(listed) - parse_names(ignored)
    if not active:
        raise RuntimeError('No active tests found; refusing empty validation')
    selected = args.test or sorted(active)
    if len(selected) != len(set(selected)):
        raise RuntimeError('Duplicate test selection')
    for name in selected:
        if name not in active:
            raise RuntimeError(f'{name} is unknown or ignored; refusing execution')
    (report / 'tests.txt').write_text('\n'.join(selected) + '\n')
    for name in selected:
        run_bounded(report, rows, 'test', name,
                    [str(binary), '--exact', name, '--test-threads=1', '--nocapture'], 15)
    print(f'{len(selected)} tests passed individually; {len(parse_names(ignored))} tests ignored.', flush=True)
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (RuntimeError, OSError, subprocess.SubprocessError) as error:
        print(f'STOPPED: {error}', file=sys.stderr, flush=True)
        sys.exit(1)
