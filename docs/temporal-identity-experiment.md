# Explicit temporal identity before constructor terms

This change belongs only to `experiment-peano-after`. The frozen v1 tag and main
remain unchanged. It corrects identity and substitution; it does not yet add
constructor terms, source grammar, arithmetic primitives or TimeExpr-to-Term
conversions.

## Typed representation

`TimeExpr` retains its structural representation:

```rust
TimeExpr::At(name)       // temporal constant
TimeExpr::Var(name)      // temporal lambda parameter
TimeExpr::After(inner)   // structural constructor over PropositionTime
```

The elaborator resolves parameter scope once and records the distinction in the
typed expression. Ground experiment inputs use `At`; rule parameters use `Var`.
`Term::Const` and `Term::Var` already distinguish ordinary argument positions.
Program compilation preserves the same distinction when converting temporal
bindings into temporal expressions. State transition indices reuse these typed
expressions; the State runtime and storage representation are unchanged.

Matching and residual goal matching bind only `Var`. Constants compare literally,
including beneath `After`. Substitution traverses `After`, replaces `Var` and
leaves `At` untouched. Rule parameter lists still validate binder scope; they no
longer decide whether an `At` node secretly denotes a variable. Program compilation
retains its existing namespace for shared and local binders.

An unresolved temporal variable renders as `?name` in diagnostics. This is not new
source syntax. Ground times and their CLI output retain the previous spelling.

## Homonymous capture regression

The source experiment is
`examples/experiments/temporal_identity_formal.res`. Its essential case is:

```text
const t : PropositionTime
rule template(first, object, t) =
    A(object) @ first & B(object) @ t <= C(object) @ after t
rule partial = template(t)
rule fixed = partial(item, tau)
rule direct = template(t, item, tau)
```

After the first application, A is indexed by constant `At("t")`, B by the free
parameter `Var("t")`, and C by `After(Var("t"))`. After the second application,
A remains at constant t, while B and C specialize to tau and after(tau).
`fixed` and `direct` have identical clause bodies.

The analogous Program captures t across its RuleUse. Compiled shared parameters
retain their existing hygienic names. Partial and direct complete applications
have equivalent compiled clauses and computed consequences.

| Evidence / activation | Result |
|---|---|
| A@t, B@tau, activations@tau | C@after(tau), two justifications, one logical event |
| A@tau, B@tau, activations@tau | No C: captured constant t does not match tau |
| A@t, B@t, activations@tau | No C: activation binds the remaining parameter to tau |
| A@t, B@tau, no activations | No C: declarations do not activate themselves |

Both successful justifications preserve the activation witness, A@constant t,
B@constant tau and the temporal substitution to tau. Program origin identifies
CapturedTime, template and TemporalIdentity. Known inputs remain unchanged.

A residual query for C@after(tau), with only B@tau, returns A@constant t. Adding
A@tau does not satisfy that requirement. Adding A@constant t does. The residual
substitution retains the constant/parameter distinction without consulting the
original source context.

## Verification and scope

28 selected tests passed individually through `python3 scripts/test_safe.py`:

- Four new identity regressions: report
  `target/test-diagnostics/20261008T223749275899Z-3582208`.
- Ten CLI regressions covering temporal beta, Programs, module provenance,
  default Programs, 25 Peano sums, 25 Peano products and the polynomial boundary:
  `target/test-diagnostics/20261008T223831094853Z-3583186`.
- Fourteen library regressions covering literal temporal constants, temporal beta,
  nested matching, effect transitions, default blockers, EvidentialNot-related
  conclusions, residual substitution and residuation round trips:
  `target/test-diagnostics/20261008T224024709589Z-3584811`.

Compilation used approximately 444.56 MiB peak RSS for CLI tests and 647.98 MiB for
library tests. Selected test processes stayed below approximately 15.50 MiB peak
RSS. The runner retained 1 GiB total, no swap, one Cargo job and one harness thread.
No complete suite was run; nine paused CNL tests remain ignored.

These are regression results, not a proof for arbitrary programs. Historical tests
that inspect a lambda's typed syntax now explicitly expect `Var`; ground constant
expectations remain `At`. No parser changes were necessary.

The next stage remains separate: typed constructor terms and a structure-preserving
conversion between TimeExpr and Term. A computed temporal index still cannot be
passed as an ordinary coefficient/exponent argument. The existing polynomial
boundary regression verifies that this correction does not silently introduce
that conversion.

## Subsequent constructor stage

The identity-only boundary above was subsequently removed on the experimental
branch. See [temporal-constructors-experiment.md](temporal-constructors-experiment.md)
for the structural Term/TimeExpr correspondence and successful computed-value
transfer. The identity invariants and homonym regressions remain in force.
