# Interactive supervisor: design, planned commands and stop policy

Status: **implemented, not executed**. The components were prepared in
`7fa15a0` (channel backend, branching policy, entry point) and `e57b67f`
(`supervise` in `scripts/test_safe.py`), then corrected after review in
`4e8d0b2` and `6abe5c6`: one deadline for write and read, framing validated
before any field is touched, grace periods inside the supervised loop, path
normalisation of the worker manifest, the interrupted episode recognised by
the ledger, and an idempotence probe against the live worker. Neither has
launched the worker binary next to the controller. The mode is to be
reviewed before it is built or run.

## Shape

```text
scripts/test_safe.py --interactive-manifest EPISODE.json
                     --interactive-ledger target/exploration/<episode>/
                     --interactive-policy branching
  │
  ├─ stage 1  compile residuum-worker        own scope, 45 s, existing cargo controls
  │           (cargo build --offline --locked -p residuum-boundary --bin residuum-worker --jobs 1)
  │
  └─ stage 2  controller+worker              ONE scope: MemoryMax=1G, MemorySwapMax=0,
              ├─ residuum-worker --manifest <ledger>/worker-manifest.json
              │     stdin  ← pipe A (read end)        stdout → pipe B (write end)
              │     stderr → NN.worker-stderr.txt
              └─ python3 -I -c <bootstrap> run_episode.main
                    --manifest EPISODE.json --ledger <ledger> --policy branching
                    --to-worker <fd of pipe A write end> --from-worker <fd of pipe B read end>
                    stdin ← /dev/null   stdout → /dev/null   stderr → NN.controller-stderr.txt
```

The supervisor creates both pipes, starts both processes with
`start_new_session=True`, passes only the two controller descriptors with
`pass_fds`, closes its own copies, and supervises. The controller's
`ChannelBackend.from_descriptors` adopts the descriptors; the controller
has no `Popen` and no path to the binary. The worker receives only the
`worker` section of the episode manifest, written by the supervisor into
the ledger directory; the controller validates the whole episode manifest.

## Joint budget and accounting

| Control | Value | Where |
|---|---|---|
| memory | 1 GiB, no swap, CPU quota 100 % | the scope of stage 2, shared by both processes |
| preventive stop | cgroup `memory.peak` ≥ 768 MiB | polled every 20 ms, covers both processes and page cache |
| growth | +64 MiB within one second once the **sum** of both RSS exceeds 128 MiB | polled on both PIDs |
| time | 15 s for the whole exchange, like a fixture run | from the start of stage 2 |
| compile | 45 s, own scope | stage 1 |
| per-request deadline | 10 s inside the controller (`ChannelBackend.request_timeout`), one deadline covering the complete non-blocking write (partial writes resumed with `select`) and the read of the answer | bounded by the 15 s of the scope |
| grace periods | 2 s for the surviving process after the other exits, inside the 15 s, watched by the same loop with the same memory and time checks | stage 2 |
| line sizes | 16 MiB on the controller side; the worker's own `TransportLimits` | both ends |

## Stop policy

Any of these ends stage 2; on every end both process groups receive
SIGKILL and are waited for, so no child survives:

| Reason | Trigger | Recorded as |
|---|---|---|
| `timeout` | 15 s elapsed | stop reason |
| `near_memory_limit` | cgroup peak ≥ 768 MiB | stop reason |
| `abnormal_memory_growth` | joint RSS growth rule | stop reason |
| `worker_lost` | the worker exits while the controller is still running | stop reason; the controller gets the grace period to see EOF or a broken pipe, record `episode.aborted` and exit 4; if it does not exit within the grace it is killed, and then the ledger has no terminal event |
| `worker_did_not_exit` | the controller finished and closed its channels but the worker did not exit within the grace | stop reason |
| `controller_nonzero_exit` | controller exit code ≠ 0 (3 budget, 4 technical abort, 5 manifest, 2 usage) | stop reason |
| `worker_nonzero_exit` | worker exit code ≠ 0 (5 incomplete line, 6 stream, 3/4 start-up) | stop reason |
| `cgroup_oom` | memory events | stop reason |

Normal end: the controller finishes its episode, runs the idempotence
probe, closes the channels, exits 0; the worker reads EOF on stdin and
exits 0; the result row carries `exit_status` (controller) and
`worker_exit_status` (worker), elapsed time, RSS and cgroup peak. Technical
stops never produce a normative statement. The supervisor's `result.json`
is the authoritative record of a technical end: the controller records
`episode.aborted` when it gets the chance, but a kill at the end of a grace
period can prevent it, and `Ledger.reconstruct()` then reports
`interrupted: true` with no terminal event instead of pretending to know how
the episode ended. No candidate is marked incompatible in either case.

## Files produced

```text
target/test-diagnostics/<run>/01.*            compile stage
target/test-diagnostics/<run>/02.output.txt   supervisor log (pids, exits, times)
target/test-diagnostics/<run>/02.worker-stderr.txt
target/test-diagnostics/<run>/02.controller-stderr.txt   one JSON summary line at the end
target/test-diagnostics/<run>/02.result.json  exit statuses, elapsed, RSS, cgroup peak, stop reason
<ledger>/worker-manifest.json                 what the worker loaded
<ledger>/events.jsonl, actions.jsonl, …       the episode ledger (full lines as artefacts)
```

## Exclusions

The mode takes only its three options and rejects fixtures, tests,
formatting, CLI, worker-file and python-test options. The ledger directory
must be under `target/` and empty or absent; ledgers are never reused. The
policy name must be an identifier known to `tools/normative_explorer/policies.py`.

## Worker manifest and paths

The worker resolves `imports` and `workdir` against the directory of the
manifest file it receives. Since the supervisor writes the worker section
into the ledger directory, relative paths would silently change meaning.
`normalize_worker_manifest` resolves them against the directory of the
original episode manifest before writing, and records every change in
`<ledger>/worker-manifest-normalization.json` (original value, resolved
absolute path, base). The episode manifest itself is not modified, so the
controller's frozen hash is unaffected.

## The decisive check, after review

Run once with the `branching` policy on the Riggs episode manifest in
condition C. The policy chooses each action from the last view: it repeats
advances while results are pending, proposes after the historical closure
completes, evaluates until the report arrives, asks for minimal supports
when compatible, then opens a forward session on
`killingWithoutAcquisitionPurpose`, a dossier no recording covers, and
advances it to completion before finishing. To verify, independently of
the controller's summary: the ledger reconstructs every action; the
worker's answers for the uncovered dossier are compared with
`close_program` on that dossier through a boundary test; session
identifiers, budgets, the `unresolved_parameters` and protocol error
paths, and provenance (derivations with records) are checked from the
ledger artefacts, and the stderr summary's `decisions` show the branches
taken. Idempotence is checked against the same live worker, not by running
the episode twice: after the episode, the entry point resends the first
completed worker action with the same identifier and content
(`Controller.resend`), and the record `action.resent` states whether the
answer was identical, whether the worker's reported budget and steps were
unchanged, and that the controller charged nothing. Running the episode
twice would show reproducibility, which is a different property.
