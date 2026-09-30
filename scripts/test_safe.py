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
    samples = collections.deque()
    reason = None
    started = time.monotonic()
    with (report / f'{index:02d}.output.txt').open('w') as output:
        process = subprocess.Popen(request['command'], cwd=ROOT, stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
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
                if (request['stage'] == 'test' and rss >= 128 * MIB and samples
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


def run_bounded(report, rows, stage, name, command, timeout):
    index = len(rows) + 1
    request = dict(stage=stage, name=name, command=command, timeout=timeout)
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
          f'{row["peak_rss_bytes"] / MIB:.2f} MiB RSS', flush=True)
    if row['stop_reason']:
        raise RuntimeError(f'{name}: {row["stop_reason"]}; no remaining tests executed. '
                           f'See {report / f"{index:02d}.output.txt"}')
    return report / f'{index:02d}.output.txt'


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


def main():
    if len(sys.argv) == 4 and sys.argv[1] == '--worker':
        return worker(Path(sys.argv[2]), int(sys.argv[3]))
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--test', action='append', default=[], help='Exact active test name; repeatable')
    parser.add_argument('--binary', type=Path, help='Use an existing binary without compiling')
    parser.add_argument('--compile-only', action='store_true')
    parser.add_argument('--fmt-check', action='store_true')
    parser.add_argument('--format', action='store_true')
    args = parser.parse_args()
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    report = ROOT / 'target' / 'test-diagnostics' / f'{stamp}-{os.getpid()}'
    report.mkdir(parents=True)
    print(f'Reports: {report}', flush=True)
    rows = []
    if args.fmt_check or args.format:
        command = ['cargo', 'fmt', '--all'] + (['--check'] if args.fmt_check else [])
        run_bounded(report, rows, 'format', 'cargo fmt', command, 30)
        return 0
    binary = args.binary.resolve() if args.binary else None
    if binary is None:
        output = run_bounded(report, rows, 'compile', 'compile only',
                             ['cargo', 'test', '--offline', '--locked', '--no-run', '--lib',
                              '--jobs', '1', '--message-format=json'], 45)
        binaries = []
        for line in output.read_text().splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if (message.get('reason') == 'compiler-artifact' and message.get('executable')
                    and message.get('profile', {}).get('test')
                    and message.get('target', {}).get('name') == 'residuum'):
                binaries.append(Path(message['executable']))
        if len(binaries) != 1:
            raise RuntimeError('Expected one compiled library test binary; refusing execution')
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
