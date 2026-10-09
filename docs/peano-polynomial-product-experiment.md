# Complete polynomial products by recursive Program composition

This experiment continues `experiment-peano-after` after `175e43b`.
It adds only source fixtures, a Rust verifier and test registration. It does not
modify any parser, AST, type-system, arithmetic primitive, `Rule`, `Program`,
default policy, `close` or runtime. Frozen `main` and `v1` remain unchanged.

## Sources and representation

- `examples/experiments/peano_polynomial_product.res` defines the convolution.
  It imports the existing polynomial-addition and Peano-multiplication modules.
- `examples/experiments/peano_polynomial_product_scenario.res` contains eight
  source-defined scenarios. Importing the arithmetic libraries does not execute
  their experiments.
- `src/bin/residuum/cli/peano_polynomial_product_tests.rs` loads those scenarios,
  calls the existing `close_program`, decodes results and checks provenance.

Inputs reuse `inputCoefficient` and `coefficient`, with exponent/coefficient
values encoded as ground `PropositionTime` constructor terms rooted at
`zeroTime`. No list, array, lookup table or precomputed arithmetic relation is
supplied to Residuum. Expected coefficients occur only in observation goals
and the independent verifier; they are not premises.

The factor domain is an explicit finite, contiguous chain, starting at zero:

```text
firstFactorExponent(zeroTime)
nextFactorExponent(zeroTime, after(zeroTime))
...
lastFactorExponent(after(...zeroTime...))
```

Both factors are dense on the same domain 0 through n, including explicit zeros.
The output domain is declared using the existing `exponent` facts, normally
0 through 2n. Unique coefficients, a unique finite successor chain and adequate
output bounds are data preconditions, not new language validation features.
Distinct result identities avoid feedback into the factors. Missing coefficients
never become zero evidence automatically.

## Algorithm in Residuum

For each known `multiplyPolynomial(p,q,r) @ t`, its associated
`PolynomialProduct` Program publishes a canonical request and starts the work.
Its fourteen members jointly enter the same forward fixpoint. A partial alias
can capture p while leaving q, r and t free.

Each pair of factor exponents (i,j) activates `PairWork`. It:

1. Reuses the previous structural operand adapter to build unary operand spines.
2. Calls the imported Peano addition to compute the output degree i+j.
3. Calls the imported Peano multiplication to compute the coefficient a_i*b_j.
4. Joins the actual degree and product results into `pairValue`.

Multiplication recursively activates its addition continuations; the product
index becomes an ordinary value in `pairValue`. The degree index undergoes
the same structural transport. `after` remains a constructor: the clauses and
the fixpoint perform the computation.

For each output exponent k, there is one contribution per factor row i.
An actual pair with computed degree k supplies its product. If k<i or k>i+n,
the row contributes zero, justified by positive Peano comparison evidence.
The upper bound i+n comes from the computed degree of the row's last pair,
not from a precomputed exponent-sum table.

Strict comparison is defined entirely by ordinary Horn clauses:

```text
compare(0,S(b)) -> less(0,S(b))
compare(S(a),S(b)) -> compare(a,b)
compare(S(a),S(b)) & less(a,b) -> less(S(a),S(b))
```

Neither failure to compare nor absence of a coefficient creates a zero.

A recursive row fold then computes:

```text
productFold(job,k,i,total)
    -> known zero contribution: advance with the same total
    -> positive contribution: FoldAddition -> imported Addition -> new total
    -> next row via the declared successor relation
    -> final row: productFinished -> coefficient(r,k,total)
```

Only the finished fold publishes a coefficient. Intermediate products are not
mistaken for the full coefficient. All requested output exponents, including
zero coefficients, complete independently in the shared closure.

Jobs include operand/result identities and the request time. Distinct record
types identify degree sums, pair products and row sums; their fields include
the appropriate row/column/output exponents. This prevents accidental reuse of
results between different requests or stages. The request time is metadata of
the source instance, separate from constructor values used as numbers.

## Verified complete products

| Scenario | Computed result |
|---|---|
| `partialProduct` | (1+2x)(2+x) = 2+5x+2x² |
| `zeroProduct` | (1+2x)*0 = 0, with three explicit zero coefficients |
| `unitProduct` | (1+2x)*1 = 1+2x, including zero at exponent 2 |
| `gapsProduct` | (1+x²)² = 1+2x²+x⁴, with explicit zeros at exponents 1 and 3 |
| `independentProducts` | One request yields 1+x; another yields 1+2x+x² |
| `computedProductThenSum` | The calculated product becomes an operand of the existing polynomial-addition Program |
| `absentProduct` | No products or row folds without a request |

The complete degree-one product has 223 facts and 244 justifications. It computes
four term pairs, six row contributions and all three finished coefficients.
No result was supplied as evidence. A deliberately missing factor coefficient
leaves the unsupported output exponents unfinished rather than assuming zero.

Partial application specializes all fourteen rule uses, including job captures
with temporal fields. Unapplied parameters and local clause parameters remain
free. The specialized and direct requests produce the same logical results.
Independent requests compute both outputs in one closure; their published
coefficients retain the correct job identity and request time. A stronger test
that also recomputed the two separate closures reached its 15-second timeout
after the joint computation had produced its outputs. That comparison was
stopped; equality of joint/separate proof sets is not established here. No clock
stamp is used to decide a result.

The provenance audit replays every derivation of the small complete product
using the existing matcher, with the existing implicit activation guard made
explicit only in the verifier. It checks substitutions, witness order,
proposition types/times and producer identity. Each final coefficient has a
support path to the original request, both factors, and the imported recursive
multiplication/addition Programs. Program, member rule and module origins remain
available through the existing provenance API.

These observations establish that a complete polynomial product is expressible
and computed on these finite cases. They are not a general complexity,
termination, correctness or completeness theorem.

## Larger case: unresolved execution-time limit

`principalProduct` retains the earlier degree-three inputs:

```text
p = 1 + 2x + 3x² + 4x³
q = 5 + 3x²
```

Their mathematical convolution would be:

```text
5 + 10x + 18x² + 26x³ + 9x⁴ + 12x⁵ + 0x⁶
```

This is an expected result, **not a verified output of Residuum**. Two controlled
attempts reached the existing 15-second test timeout, at approximately 22 MiB
RSS. The runner killed each process and stopped the remaining work. No further
attempt at that case was made and no resource limit was raised.

The second attempt grouped bounds and row traversal as recursive members of the
single request Program instead of activating a Program for each row/stage; it
still timed out. That organization is retained and verified on the smaller
cases. A timeout alone does not establish that the larger closure is infinite,
nor demonstrate a missing representational or arithmetic capability. Its
performance remains unresolved. The large scenario is a diagnostic fixture,
not an active test asserting that its closure currently completes in 15 seconds.

Reports for the two timed-out attempts:

- `target/test-diagnostics/20261009T011630350979Z-3713110`.
- `target/test-diagnostics/20261009T011900229702Z-3715696`.

The generic `--forward`/CLI adapter still rejects recursive dependency graphs
conservatively. As in the preceding Peano experiments, verification invokes
`close_program` from individual tests inside `test_safe.py`'s cgroup; it is not
an unbounded execution alternative. Source `query residual` declarations supply
observation goals, not a claim of recursive residual-query support.

## Verification and resource scope

The final unique import graph is 63,598 bytes, within the existing 65,536-byte
source budget. Only indentation of new files was reduced; the original loader
limit, original fixtures and module-count limit were preserved.

The final tests use `python3 scripts/test_safe.py --test-target cli --test NAME`,
with a separate process per test. They preserve 1 GiB total, no swap, one Cargo
job, one harness thread and the existing timeouts/preventive stop threshold.
No full suite was requested or executed.

Final full-coefficient and provenance checks:
`target/test-diagnostics/20261009T012157516872Z-3718532`.

Remaining case checks and selected addition/multiplication regressions:
`target/test-diagnostics/20261009T012242333455Z-3719662`.

The joint/separate recomputation timeout is recorded in
`target/test-diagnostics/20261009T012242333455Z-3719662`. The final standalone
joint-case, chaining and missing-input checks, plus two regressions, passed in
`target/test-diagnostics/20261009T012747089723Z-3723662`.

Across the final source/verifier versions, ten new tests and two regressions
passed individually. Earlier controlled timeouts remain recorded failures; they
are not included as passing tests. The largest verified product case,
`gapsProduct`, took approximately 12 seconds; the joint independent-request case
took approximately 10.4 seconds. Selected passing processes used less than
17 MiB RSS; final compilation used approximately 305 MiB. These finite results
do not establish scalability to the larger diagnostic case.
