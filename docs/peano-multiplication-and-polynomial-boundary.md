> Historical stage report: the TimeExpr-to-argument and source-constructor
> boundaries described below have since been removed on experiment-peano-after.
> See [temporal-constructors-experiment.md](temporal-constructors-experiment.md)
> for the current implementation. Polynomial addition itself is still pending.

# Peano multiplication and the polynomial-value boundary

Worktree: `/home/maitreya/residuum-peano`, branch `experiment-peano-after`.
Baseline: frozen `v1`, commit `8025bc6b4afee3415c652a5830576ef18aaf8cb5`;
the addition experiment is committed as `531544f` on this branch.
No production Rust code, parser, AST, type system, matcher, closure, runtime,
runner resource settings, frozen tag, or main branch is changed by this extension.

## Multiplication expressed in source

Fixture: `examples/experiments/peano_multiplication.res`.
It imports `peano_after.res` and reuses its operand spines, `add`, `Addition`,
`LeftWalk`, `RightWalk` and `sum` without changing those declarations or rules.
Imported experiments are not executed. Multiplication data are input operand
spines and requests only; neither sums nor products are supplied as seeds.

As before, this experiment interprets `At("zeroTime")` as Z and `After(t)` as S(t).
This is an experimental interpretation of structural indices, not a change to
PropositionTime generally or to DerivationTime.

The recurrence is implemented by a continuation over the multiplier's input spine:

1. `multiply(a,b,request) @ Z` activates `Multiply` and produces
   `productSoFar(a,b,request,firstCursor) @ Z`.
2. The explicit endpoint of a zero-length multiplier derives `product(request) @ Z`.
3. Each successor of b activates `MultiplyStep`. It constructs an
   `AdditionInvocation { operation = request, position = cursor }` using a normal
   record literal in a RuleUse binding, then derives an `awaitingSum` continuation.
4. `AdditionContinuation` requests the existing addition:
   `add(nZero,a,call) @ accumulator`.
   The existing addition constructs `sum(call) @ S^a(accumulator)`.
5. The continuation consumes that *derived* sum and produces the next
   `productSoFar` at precisely the returned index. The final multiplier endpoint
   derives the product; the ordinary `observeProduct` clause records completion.

Thus `a * 0 = 0` and the transition for the next multiplier successor is
`a * S(b) = (a * b) + a`. Addition already accepts a base index, so the accumulator
can be transported as an index without becoming a term-valued coefficient.
There is no arithmetic result table, no sum computed by Rust and inserted into
Known, and no numeric primitive. Rust's integer arithmetic is an independent
verification oracle only.

The original `add` declaration remains Seeded. v1 currently permits a positive
seeded-verb event as a rule consequent: `requestAddition` derives the request and
its association activates `Addition`. Those internal `add` events have derivation
records; they are not externally supplied seeds. This records existing v1 behavior,
without changing verb categories or claiming a new type-system law.

## Identity, partial application and provenance

Each addition invocation contains the outer request and the current multiplier
cursor. Different iterations and different multiplication requests therefore have
distinct ordinary record-valued identities. This is a constructor supplied by the
source Program binding, not an arithmetic lookup or a Rust routing table.

The constructor appears in a consequent. A later associated program receives the
already-ground record through a variable parameter and passes it to addition.
No recursive record-pattern matching is assumed. `AdditionInvocation` nominally
extends the existing `Request` entity, so it fits the unchanged addition signature.

`ProductStepForFour`, `ProductStepPartial`, `ProductStepGround` and
`ProductStepDirect` exercise partial application and beta reduction. Substitution
reaches both member rules, the request captured inside the invocation record, and
the temporal binding. The unapplied cursor/time parameters remain in the partial
program. `MultiplyByFour` also compiles to a clause whose residual parameters match
all five source requests for that operand.

Every arithmetic justification is replayed against the compiled producer clause,
comparing its complete derivation record, substitutions, ordered witnesses, event
and proposition type/time. Origins distinguish multiplication programs/rules in
`PeanoMultiplication` from the unchanged addition programs/rules in `PeanoAfter`.
Both run in the same `close_program` fixpoint. No Effect or State execution occurs.

A concurrent scenario computes 4 * 4 and 2 * 3 in one closure. Its seven distinct
addition calls preserve their separate root requests and produce 16 and 6 without
cross-contamination. Requests must identify one operand pair, and operand spines
must be finite, acyclic, unbranched, and have their unique endpoint at their length.
The number of arithmetic successor steps is a*b; this is not a claim that the
current general-purpose matcher has O(a*b) running time.

## Verified finite results

Each of the 25 cases below is run using source-defined inputs in its own closure.
All cells are **computed outputs**, checked by the independent oracle.

| a \ b | 0 | 1 | 2 | 3 | 4 |
|---|---:|---:|---:|---:|---:|
| 0 | 0 | 0 | 0 | 0 | 0 |
| 1 | 0 | 1 | 2 | 3 | 4 |
| 2 | 0 | 2 | 4 | 6 | 8 |
| 3 | 0 | 3 | 6 | 9 | 12 |
| 4 | 0 | 4 | 8 | 12 | 16 |

The verifier also checks the zero/one laws, the recursive equation on successive
multiplier prefixes, and commutativity for this finite domain. The largest case
produces a TimeExpr with sixteen nested `After` constructors, four independently
justified sums, and 47 total derivation records. No sixteen-element operand spine
or numeral constant is supplied.

**EXPERIMENTAL OBSERVATION:** the v1 core computes these products using recursive
associated programs and the existing addition.

**Restricted algorithm argument:** the initial accumulator is Z. Each of the b
successors produces exactly one uniquely identified addition of a to that index.
Induction on the finite multiplier spine gives S^(a*b)(Z). A cursor advances once
per completed call and never moves backwards, so well-formed finite spines yield
a finite closure. This is not a theorem about arbitrary programs or malformed data.

The CLI/forward fixture runner still rejects these recursive dependencies using
its existing conservative acyclicity guard. Core API execution is tested under
`test_safe.py`; no CLI restriction is removed or bypassed in the CLI.

## Polynomial representation: stop before implementation

The initial candidate is a coefficient relation indexed by polynomial and
exponent, with an explicit coefficient for every exponent in a bounded domain.
For the proposed p and q, exponents would be 0 through 3 and q would explicitly
supply zero coefficients at exponents 1 and 3. This avoids treating absent evidence
as a zero coefficient or introducing negation-as-failure.

Before writing coefficient-wise addition, the probes test whether computed Peano
indices can become exponent/coefficient *arguments*. Three distinct boundaries
must not be conflated:

| Probe | Existing v1 behavior |
|---|---|
| Transfer computed index to an argument of nominal `Natural` type | Elaboration rejects `PropositionTime` as not assignable to `Natural`. |
| Use `PropositionTime` for the exponent/coefficient argument type | Elaboration accepts the rule, but forward instantiation yields no coefficient event. |
| Write `after(t)` directly as an event argument | Formal parsing rejects it; event arguments accept identifiers, not TimeExpr constructors. |

The valid probe is `peano_polynomial_time_boundary.res`. It computes
`sum(r23) @ S^5(Z)` with the original addition and tries these ordinary rules:

```text
rule exportCoefficient(request, t) =
    sum(request) @ t <= coefficient(p, zeroTime, t) @ zeroTime
rule exportExponent(request, t) =
    sum(request) @ t <= coefficient(p, t, zeroTime) @ zeroTime
```

Their parameter t has type PropositionTime. Nevertheless, neither derives a
coefficient. In `src/derive.rs`, `bind_time` binds t as
`SubstitutionValue::Time(TimeExpr)`. `substitute_term` accepts only a
`SubstitutionValue::Term` for an argument variable. `has_unbound_rule_parameter`
therefore treats the consequent argument as unresolved and suppresses that
instantiation. The temporal expression remains usable in an `@ t` position.
There is no TimeExpr variant in `Term`, nor an existing operation that materializes
an arbitrary constructed TimeExpr into a term-valued numeral.

This is **not** solely a static type rejection: the PropositionTime-typed version
passes elaboration and stops at the matching/substitution boundary. Declaring
`PropositionTime` as an argument type does not add the missing term representation.
A positive control copies a supplied named `zeroTime` argument from
`suppliedTimeValue` to `copiedTimeValue`; it is explicitly supplied evidence, not a
computed coefficient. The two `.invalid.res` files are intentional negative probes
of nominal typing and source grammar, respectively.

**STOP:** coefficient-wise polynomial addition is not implemented, and no claim is
made that the requested 4x^3 + 6x^2 + 2x + 6 was computed. The missing operation for
this representation is passing a *constructed* numeric index as an ordinary typed
argument. No AST, parser, matcher or coercion was added to bypass it.

This does not prove that every possible polynomial encoding is impossible in v1.
An encoding with opaque value identifiers and separately indexed numeric facts
might avoid direct TimeExpr arguments, but it has not been implemented or verified
here: the requested investigation stops at the value-transfer boundary.

## Bounded verification

Only individual tests were selected; no full suite was run.

- Largest-case report: `target/test-diagnostics/20261008T215600198548Z-3545792`.
  4 * 4 passed in 0.713 seconds with about 15.50 MiB reported peak RSS.
- Remaining tests and addition regressions:
  `target/test-diagnostics/20261008T215634588773Z-3546432`.
  Nine selected tests passed individually: six new multiplication/boundary tests
  and all three original addition tests. The all-25-products test took 2.115 seconds;
  the concurrent/partial-application test took 1.180 seconds. Peak test RSS remained
  at or below approximately 15.50 MiB.
- Initial test compilation passed at approximately 281.42 MiB reported peak RSS.
- The runner retains 1 GiB total memory, no swap, one Cargo job, one harness thread,
  timeouts and preventive stopping. No limits or controls were changed.

Together these are seven new selected tests and three unchanged addition tests.

## Subsequent temporal identity correction

The later experimental identity correction distinguishes TimeExpr::Var from
TimeExpr::At; see [temporal-identity-experiment.md](temporal-identity-experiment.md).
It preserves the value-transfer boundary documented above. Constructor terms and
TimeExpr-to-Term conversions are still pending.
