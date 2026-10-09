#!/usr/bin/env python3
"""Run selected individual measurements through test_safe.py, serially.

No compilation, direct test-binary execution, limit changes or full-suite run.
Every failure stops the sequence, preserving the raw safe-runner reports.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
TEST = 'derive::profile_tests::bounded_program_scaling_observes_work_without_changing_evaluation'
CASES = ('depth0', 'depth2', 'depth4', 'jobs4', 'jobs9', 'jobs25',
         'multiplications', 'proofs1', 'proofs4', 'proofs16')


def fields(line):
    result = {}
    for key, value in re.findall(r'(\w+)=([^\s]+)', line):
        try:
            result[key] = float(value) if '.' in value else int(value)
        except ValueError:
            result[key] = value
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True,
                        help='Existing library-test binary; compilation remains a separate safe-runner step')
    parser.add_argument('--repetitions', type=int, choices=range(1, 6), default=3)
    args = parser.parse_args()
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error('Compile the selected test with test_safe.py first')
    output = ROOT / 'target/program-scaling-results.json'
    rows = []
    metadata = dict(test=TEST, binary=str(binary), repetitions=args.repetitions,
                    scope='bounded source fixtures; cold closures; profiler enabled',
                    rows=rows)
    for case in CASES:
        for evaluator in ('reference', 'incremental'):
            for repetition in range(1, args.repetitions + 1):
                env = dict(os.environ, RESIDUUM_SCALING_CASE=case,
                           RESIDUUM_SCALING_EVALUATOR=evaluator)
                run = subprocess.run([sys.executable, str(ROOT / 'scripts/test_safe.py'),
                                      '--binary', str(binary), '--test', TEST],
                                     cwd=ROOT, env=env, text=True, capture_output=True)
                reports = re.findall(r'Reports: (.+)', run.stdout + run.stderr)
                report = Path(reports[-1]) if reports else None
                if run.returncode != 0 or report is None:
                    metadata['stopped'] = dict(case=case, evaluator=evaluator,
                                              repetition=repetition, report=str(report),
                                              exit_status=run.returncode)
                    output.write_text(json.dumps(metadata, indent=2) + '\n')
                    print(run.stdout + run.stderr, file=sys.stderr)
                    return 1
                records = json.loads((report / 'results.json').read_text())
                tests = [(i, r) for i, r in enumerate(records, 1) if r['stage'] == 'test']
                if len(tests) != 1 or tests[0][1]['stop_reason'] is not None:
                    raise RuntimeError(f'Unexpected safe-runner result: {report}')
                index, resource = tests[0]
                text = (report / f'{index:02d}.output.txt').read_text()
                profiles = [line for line in text.splitlines() if line.startswith('PROFILE END ')]
                counts = [line for line in text.splitlines() if line.startswith('SCALING ')]
                if len(profiles) != 1 or len(counts) != 1:
                    raise RuntimeError(f'Incomplete profile: {report}')
                row = dict(case=case, evaluator=evaluator, repetition=repetition,
                           report=str(report), profile=fields(profiles[0]),
                           counts=fields(counts[0]), resource=resource)
                rows.append(row)
                output.write_text(json.dumps(metadata, indent=2) + '\n')
                print(f"{case} {evaluator} {repetition}: "
                      f"{row['profile']['wall_ms']:.3f} ms closure; "
                      f"{resource['peak_rss_bytes']/1048576:.2f} MiB RSS", flush=True)
    print(f'Results: {output}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
