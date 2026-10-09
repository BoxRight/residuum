# Dense polynomial addition with symbolic Peano coefficients

This experiment lives on `experiment-peano-after`, after commit `4ac81a4`.
It uses the branch's existing constructors, typed records, imports, Programs,
partial application, beta reduction and `close_program`. It adds no language
feature or engine operation. Frozen `main` and `v1` remain unchanged.

## Source and execution

- `examples/experiments/peano_polynomial_addition.res`: declarations, polynomial
  Programs and the structural operand adapter; imports `peano_after.res`.
- `examples/experiments/peano_polynomial_scenario.res`: twelve source-defined
  experiments, supplying only domains, input coefficients and requests.
- `src/bin/residuum/cli/peano_polynomial_tests.rs`: independent result verifier
  and provenance audit. Rust does not implement polynomial addition.

Run the principal experiment's verifier under the existing resource limits:

```sh
python3 scripts/test_safe.py --test-target cli \
  --test cli::tests::peano_polynomials::polynomial_principal_computes_four_coefficients_from_requests_and_input_only
```

The generic `--fixture ... --forward` runner and CLI forward adapter currently
require an acyclic dependency graph. They reject this recursive experiment
before computing its closure. They are not changed here. As in the earlier
Peano experiments, individual tests load the `.res` module and invoke the
existing `close_program`, inside `test_safe.py`'s bounded process. This is not
an unbounded alternative runner. The source's `query residual` declarations
provide observable goals; this experiment checks forward membership, not
residual search over these recursive Programs.

## Representation and preconditions

`Polynomial` is an identity. Its coefficients are relations:

```text
inputCoefficient(p, exponent, value) @ zeroTime
coefficient(p, exponent, value) @ zeroTime
```

The first is Seeded; the second is Derived. `receiveCoefficient` publishes an
input into the reusable coefficient relation. Outputs can subsequently be
operands without turning derived coefficients into new seeds.

Both exponent and coefficient have type `PropositionTime`, encoding unary
naturals by `zeroTime` and nested `after(...)`. This is an experimental encoding,
not a claim that all proposition times are natural numbers. No numeric type,
arithmetic primitive, list or array is introduced.

An explicit finite set of `exponent(e) @ zeroTime` facts supplies the processing
domain. The scenarios use 0 through 3. Dense input requires exactly one
coefficient per polynomial and declared exponent, including explicit zeros.
These are data preconditions, not newly enforced schema constraints. Missing
coefficients do not mean zero: that exponent produces no result. The algorithms
expect ground unary values rooted at `zeroTime`, not arbitrary temporal roots.

The fixtures contain input coefficients and goal observations, never sum facts,
output coefficient seeds, lookup tables or expected answers as rule premises.
The Rust decoder only counts constructors to check results after the closure.

## One shared forward computation

```text
addPolynomial(p, q, r) @ requestedAt
    -> PolynomialAddition
    -> polynomialRequest(p, q, r) @ requestedAt
    -> coefficientAddition(p, q, r, e, a, b) @ requestedAt
    -> CoefficientAddition
    -> add(nZero, PeanoOperand { value = b }, invocation) @ a
    -> imported Addition / LeftWalk / RightWalk
    -> sum(invocation) @ total
    -> coefficient(r, e, total) @ zeroTime
```

`PolynomialAddition` groups five rule uses: publish the canonical request,
provide the zero operand's start/end, start each coefficient computation and
collect each result. Publication has body `I`, but the known associated verb
instance remains its activation guard and provenance witness. A declaration
alone never activates it.

`CoefficientAddition` groups three rule uses. They create the right operand's
start cursor, request its unfolding, and invoke the existing addition relation.
The adapter structurally peels `after(tail)` into an `operandStep`. Its associated
`OperandSuccessor` emits the spine edge and recursively unfolds `tail`; at
`zeroTime`, `finishOperand` emits the end cursor. This builds operand structure,
not a precomputed result. The imported addition then adds the right spine to
the left coefficient carried as its initial index.

`after` only constructs terms. The source rules and the shared fixpoint generate
all walks, continuations and results. There is no imperative successor action.

An invocation is a typed `PolynomialSumInvocation <: Request` record containing
left/right/result identities, exponent and the request's proposition time.
`collectCoefficient` requires the exact same invocation as the completed sum.
Requests with different operands, destinations, exponents or times therefore
do not accidentally consume one another's results. Identical requests are the
same logical fact under the existing set semantics.

The request's proposition time is distinct from the computed coefficient.
The old Peano addition uses its result index to carry the latter; collection
transports that structure back into an ordinary coefficient argument. Output
coefficient observations remain indexed at `zeroTime` by explicit source rules.

## Partial application

```text
program AddFromP = PolynomialAddition(p)
program AddPAndQ = AddFromP(q)
program AddPQGround = AddPAndQ(r, zeroTime)
program AddPQDirect = PolynomialAddition(p, q, r, zeroTime)
seeded verb addPolynomialFromP(right : Polynomial, result : Polynomial) = program AddFromP
```

Applying `p` substitutes it into all five members, leaving `right`, `result`
and `t` free. The rules' exponent, coefficient and total binders remain local.
The ground and direct Programs have identical rule uses. The partial verb
activates the residual Program and produces the same polynomial as the ordinary
three-argument request. Both routes publish the same internal request, with the
appropriate original activation retained as evidence.

## Observed results

Coefficient vectors in this table are displayed in ascending exponent order
only to summarize the verifier's results; no such vector is an algorithm input.

| Scenario | Derived output, exponents 0, 1, 2, 3 |
|---|---|
| `principal`: p + q | 6, 2, 6, 4 |
| `zeroRight`: p + 0 | 1, 2, 3, 4 |
| `zeroLeft`: 0 + p | 1, 2, 3, 4 |
| `bothZero`: 0 + 0 | 0, 0, 0, 0 |
| `intermediateZeros` | 2, 0, 2, 1 |
| `concurrent` | Both principal and intermediate results, independently |
| `partial` | 6, 2, 6, 4 |
| `absent` | No result, despite all operand coefficients being available |
| `missingCoefficient` | Exponents 0, 2, 3 only; no implicit zero for exponent 1 |
| `distinctRequestTimes` | Both result polynomials, with distinct invocation times |
| `chained`: (p + q) + 0 | 6, 2, 6, 4, consuming the computed first output |
| `largerCoefficients`: 9 + 11 at exponent 0 | 20, 0, 0, 0 |

In ordinary notation, the principal output is **4x³ + 6x² + 2x + 6**.
Its closure has 93 logical facts and 82 justifications. Four imported additions
compute its four output coefficients.

These are experimental observations, not a universal soundness, completeness
or termination proof. The displayed finite scenarios have finite unary operand
spines and no cyclic polynomial data dependencies. Arbitrary requests that feed
growing outputs back into their own inputs may fail to reach a finite closure;
no engine termination policy is changed.

## Provenance and verification

The audit replays every principal/concurrent derivation with the existing
matcher, making its existing implicit Program activation guard explicit only in
the verifier. It checks exact substitutions, proposition types/times, ordered
witnesses and producer identity, and rejects duplicate event/record proofs.
It decodes `Program + Rule + Module` origins, including the imported Peano rules.
Each principal coefficient has a support path to its request, both polynomial
inputs and the coefficient/addition Programs.

The concurrent closure equals the union of the two separate logical closures.
Their event/derivation-record pairs also coincide with the joint execution;
`derivedAt` remains the clock stamp of each execution and is not compared across
them. Shared operand facts do not create cross-request arithmetic results.

Eight new tests and three selected constructor/multiplication regressions passed
individually, with no complete-suite execution:

- Principal: `target/test-diagnostics/20261009T001749658969Z-3661051`.
- Provenance: `target/test-diagnostics/20261009T001817073557Z-3661636`.
- Six remaining tests plus three regressions:
  `target/test-diagnostics/20261009T001916502301Z-3662322`.

The new test target compiled at approximately 345 MiB peak RSS; selected test
processes used at most approximately 16 MiB. The runner's 1 GiB, no-swap,
one-job/one-thread limits and preventive stop threshold remain intact.

Only the two new `.res` files, the new verifier, its test registration and this
document are added/changed. `Rule`, `Program`, defaults, `close`, the parser,
the elaborator, runtime and imported `peano_after.res` are unchanged.
