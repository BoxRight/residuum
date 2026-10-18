# Worktree integration, 2026-10-14

The user authorized merging every worktree with unmerged commits, then explicitly
authorized committing and merging the pending stepwise audit edits. Integration
was prepared and verified in `/tmp/residuum-integrate-worktrees` before advancing
main. Nothing was pushed. The v1 reference remains `f6a83e3`.

## Integrated branches

| Branch | Integrated tip | Content |
|---|---|---|
| `feat/residual-explicit-bindings` | `7ea1b7a` | Scoped typed bindings, legacy empty-map delegation, boundary regressions and `.res` source queries. |
| `12hectareas-runtime` | `de6b71b` | 1 MiB module source-graph budget; process memory, swap and test limits unchanged. |
| `12hectareas-legal-partition` | `30f91d2` | Literal ontology partition and preservation tests; includes the runtime branch. |
| `feat/stepwise-query` | `1f7b274` | Previously committed sessions at `e62232a`, plus the now committed shared-operation audit edits. |
| `experiment-peano-after` | `3aaa321` | Already reachable from main before this integration; no additional merge needed. |

The stepwise audit commit is `1f7b274`, **Share residual operations and derivation
retention across evaluation schedules**. Its previously modified files and new
residual engine are preserved in that commit. The audit also has its own
[verification record](stepwise-shared-engine-audit.md).

## Conflict decisions

The newer bindings implementation wins over the stepwise branch's earlier copy:
empty maps continue to delegate to legacy APIs, typed boundary policies remain
intact, and the agreed source syntax and explicitly scoped active interface stay
available. Source and trace adapters compose through shared scope selection,
binding resolution and guarded projection. `ActiveResidualSession::from_view`
accepts that existing projection while retaining the original module for
certification. No second scope resolver or activation projection was added.

The later audit merge retains those decisions and shares residual operations
and proof retention. The compiler's `rule_use_substitution` now also checks that
the selected RuleUse identifies the supplied original Rule before mapping it.

## Reuse versus parallel machinery

Locations refer to the integrated files; function names are stable even if line
numbers move with documentation or subsequent edits.

| New component | Existing component reused | Actual call path | New logic and why it is necessary |
|---|---|---|---|
| Source trace dispatch (`src/bin/residuum/cli/steps.rs::render`) | Loader, typed query options and bound query sessions | `execute_file_steps` → `load_formal_module` → `render` → `begin_residual_query_with_bindings` or `begin_active_residual_query_in_scope_with_bindings` | Select the requested session and display bounded progress. No source step syntax or inference. |
| Scoped bound sessions (`src/residual/bindings.rs::begin_residual_query_with_bindings`, `begin_active_query`) | Existing `scoped_rules`, `horn_view`, `active_view`, `resolve`, `preflight` | Scope/view → same scoped resolver and original head conflict check → `ResidualSession::new(...).constrained(...)`; active → `ActiveResidualSession::from_view` | Carry producer-scoped constraints into an existing resumable session. Empty unscoped active sessions use the original constructor. |
| Shared residual operations (`src/residual/engine.rs`) | Operations extracted from original DFS, original `query::match_goal`, `query::constrained_goal`, `derive::substitute_antecedent`, `substitution_record` | Full `program::query_with_constraints` and session constructor → `validate_search`; `Search::event` and layered `advance` → `classify_event`, `expand_rule`; both → leaf/rule/tensor proof helpers | Extraction avoids duplicate validation, policy, substitution and proof algorithms. Typed unresolved diagnostics remain explicit; legacy error text is retained for unconstrained DFS. |
| Layered search (`src/residual/steps.rs::ResidualSession`) | Shared operations above, existing `program::certify_candidates` | `step` → layered frontier expansion → completed proof trees → `certify_candidates` → `replay_proof` → `instantiate_from_witness` | A separate continuation scheduler is needed to expose one rule layer per step. DFS and layered allocation/budget counts differ; completed proof operations and forward certificates are shared. |
| Active completion (`src/residual/steps.rs::ActiveResidualSession::step`) | Existing `active::certify` and original `derive::close_program` | Completed guarded residual search → `active::certify` → hypotheses plus evidence in a private closure → original `close_program`; activation metadata resolves original producers | No new certificate or guard semantics. Bindings remain query constraints and real Known/module are immutable. |
| Shared activation check (`src/derive.rs::activation_bindings`) | Existing trigger polarity/name/arity checks and typed matcher | Full, incremental and stepwise activation drivers → `activation_bindings` | Move repeated trigger checks into the existing function; preserve each driver's schedule. |
| Shared retention (`src/derive.rs::retain_derivation`) | Existing proof identity, logical event identity, duplicate suppression and clock stamping | `close_with_programs`, `IncrementalClosure::update_with_clock`, `ProgramIncrementalClosure::retain` and `ForwardSession::step` → `retain_derivation` | Extract repeated retention. Event insertion/index updates remain with their original driver. |
| Synchronous forward session (`src/derive/steps.rs::ForwardSession::step`) | Existing `KnownIndex`, `instantiate_from_view`, `activation_bindings`, `instantiate_program`, shared retention | Frozen Known view → original match/instantiate functions → T(X) witnesses → shared retention → append delta | A second scheduler is intentional to expose frozen-snapshot T(X). Bindings alone do not require it. It changes intermediate rounds/order and independent derivedAt allocation; selected tests verify completed logical events and complete proof multiplicity. |
| RuleUse scope mapping (`src/programs.rs::rule_use_substitution`) | Compiler mapping already extracted in the bindings branch, existing typed term substitution | Both `compile_program` and `bindings::resolve` → the same helper | Preserve shared versus original Rule parameters, hygiene, partial application and fixed expressions. No independently reconstructed mapping. |

A concrete bound query follows this path: source
`rule rightCompliance.actEffective = gDate` → `elab::lower_query_options` →
existing `lower_term` and parameter lookup → `resolve` canonicalizes the typed
PropositionTime to `Time(At("gDate"))` → scoped rules or Program compilation
(`compile_program` → `rule_use_substitution` → `substitute_rule_body`) →
`preflight` checks the original head plus explicit constraints. During search,
`engine::expand_rule` invokes `query::constrained_goal`, which uses the original
head matcher and `bind_typed`; the original `substitute_antecedent` specializes
the body. DFS or the selected layered scheduler constructs ordered proof trees
through shared helpers, then `certify_candidates` replays them with
`instantiate_from_witness`. Active queries additionally preserve guarded
producers and call `active::certify` against original `close_program`. Supplied
bindings remain in the returned envelope and canonical proof substitutions.

No independent unifier, substitution algorithm or proof replay is introduced.
The typed-input structural validator remains new interface validation rather
than wholesale source elaboration reuse. Layered residual continuations and
synchronous forward scheduling remain explicit parallel scheduling machinery;
their budget/order differences are documented, not claimed to be legacy traces.
Derivative snapshots still use full snapshot matching rather than replacing the
existing semi-naive incremental scheduler. No additional matching or assignability
policy change was made to resolve this integration.

## Verification

Final combined code: **24 selected library tests and 15 selected CLI tests passed
individually**, sequentially through `python3 scripts/test_safe.py`:

* Library: `target/test-diagnostics/20261014T165742027742Z-1590048`.
* CLI: `target/test-diagnostics/20261014T170049607708Z-1592626`.
* The original stepwise audit worktree also passed 11 selected tests before its
  commit: `target/test-diagnostics/20261014T165416011272Z-1587190`.
* Actual compliance source trace (both Rule and Program scopes):
  `target/test-diagnostics/20261014T170623791667Z-1598127`.
* Actual active source trace (pending, Unit and alternative activation):
  `target/test-diagnostics/20261014T170749330118Z-1600197`.

These check legacy empty-map results/errors/full provenance, independent Rule
and Program specialization, explicit conflict versus incompatible head behavior,
record order and root subtyping, bound session provenance, active guards and
original certification, synchronous T and derivative behavior, incremental
proof retention, default selection, FOUR, source query/trace composition,
source budgets and literal partition preservation. Comparisons include event
kind, metadata, times, canonical substitutions, ordered antecedents, original
producers and proof multiplicity, ignoring only independent derivedAt values.

All runs retain 1 GiB cgroups, zero swap, one Cargo job and one harness thread.
No full suite or ignored CNL tests ran. This is focused integration verification,
not a claim that every historical regression was rerun.

## Worktree cleanup

After fast-forwarding main, the feature tips and the integration tip are all
reachable from main. Feature worktrees have no remaining tracked edits. Removal
is optional; keep their ignored test diagnostics if needed before removing them.
Keep `/home/maitreya/residuum` as the main checkout and the active Codex worktree
while this session uses it. No worktree or branch is deleted by this integration.

**Verdict:** existing matching, typed substitution, specialization, guarded
projection and forward certification execute directly. Residual operations,
RuleUse mapping, activation checks and derivation retention are shared helpers.
Parallel machinery consists of the explicitly requested layered search and
synchronous/derivative scheduling, not a second binding inference implementation.
Reuse is traced above and verified by focused tests; exhaustive equivalence for
all modules and equal budget exhaustion between DFS and layered search is not
claimed.
