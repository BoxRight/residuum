# Pilot 004-R run-004: integrated explorer

Purpose: to observe the integrated system. The record covers which tools the
explorer uses, which hypotheses it proposes, what it verifies and how it
responds to the information available. The explorer is not required to use
any particular tool or to reach a refutation.

## What changes from run-003

Only the capabilities available to the explorer. Corpus, valuations, order
(Brookside → Casa Fiesta → Flowers), model (`claude-sonnet-5-5`, adaptive
thinking, effort medium), feedback condition, explore/revise instructions,
budgets (40 proposals, 40 verified evaluations, 30 turns per phase,
2 clauses), US$20 safety ceiling and 400 requests are those of run-003.

### Capabilities available to the explorer

| Tool | Implementation (existing, unchanged) |
|---|---|
| `propose_rule`, `evaluate_rule`, `commit`, `end_phase` | run-003 generative cycle: admission, attributed certificate, independent closure checker, commit gate |
| `inspect_theory` | `worker::Request::InspectTheory` |
| `begin_forward` + `advance_session` | worker forward sessions (`ForwardSession`), rounds with derivations |
| `begin_residual` + `advance_session` | worker residual sessions (`ResidualSession`), search trees and residual conditions |
| `begin_active_residual` + `advance_session` | worker active residual sessions (`ActiveResidualSession`) |
| `minimal_supports` | worker frontier analysis over the joint universe |
| `compare_partition` | worker partition comparison: region pool, signatures, discriminants, inclusion |
| (always in the view) | closures of LegalGeneral plus the committed hypotheses, with justifications, antecedents, substitutions and provenance |

Navigation budget: 60 calls and 4000 engine steps per trajectory (new
manifest fields; absent fields mean navigation is off, so run-001 to run-003
reproduce unchanged).

### Integration (no new algorithm, no change to engine semantics)

- `src/experiments/generative.rs`: the `Navigate` and `Malformed` actions. At
  each new dossier the harness rebuilds a `worker::Worker` over the presented
  dossiers only. It registers every admitted hypothesis as a worker candidate,
  so each hypothesis is its own `O ∪ h` theory. Hypothesis ids are translated
  to and from candidate ids, and the trajectory's navigation budgets are
  enforced and reported. Proposals and candidate evaluation through the worker
  are refused (they go through `propose_rule` and `evaluate_rule`). The worker
  never receives the reserved valuations.
- `src/experiments/proposal.rs`: `relative` and `compose` are made visible
  within `experiments` so the goal parser can reuse them. Visibility only.
- `boundary/src/bin/residuum-trajectory.rs`: decodes the navigation tools into
  the worker's wire requests, elaborates goals written in Formal against the
  trajectory imports, and renders the worker's wire response with events as
  Formal text (responses over 40,000 characters are truncated with a marker).
  An undecodable action is refused with its reason and no longer ends the
  phase.
- `tools/pilot004r/driver.py`: the seven navigation tools, offered only when
  navigation is enabled, plus one neutral paragraph in the instructions that
  names them and their budget. No rule, head, target or hypothesis is
  suggested.

## Local verification (no model, no cost)

- `pilot004r_navigation_reaches_existing_worker_operations` exercises every
  operation, the id translation, refusals, the error for an unpresented
  dossier, rebuilding across dossiers, the budgets shown, and that no
  valuation marker appears in any view.
- `pilot004r_navigation_budget_and_disabled_navigation` covers budget
  exhaustion and navigation disabled.
- All 16 Pilot 004-R tests and the contrast test pass individually (runner
  report `20261020T221606962699Z-3011810`).
- `../run-004-local-check/` runs the scripted provider through the real
  bridge and driver with goals in Formal. It made 10 navigation calls, 2
  refusals (unknown tool, undecodable goal), 2 proposals, 1 evaluation and
  1 commit. Ledger archived in `../run-004-local-check/run/`.
- The run-003 local check, rerun with this code, gives an identical report
  apart from the two new zero counters.

## Known limits of this configuration

- The driver resends the complete view every turn, so context grows with each
  turn (views of 6k–43k characters in the local check). If the provider's
  context limit is reached, the API rejects the request, the driver records it
  as a ceiling and ends the remaining phases.
- The runner deadline is 1500 s. If it is exceeded, the run stops; the
  `turns.jsonl` written so far remains, but there is no final report.
- Navigation sessions do not survive the presentation of a new dossier.
- An engine outcome such as `UnresolvedParameters` in an active residual query
  over LegalGeneral is returned as it is, as the worker reports it.
