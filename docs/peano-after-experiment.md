# Peano addition using v1 `after`

Experimental branch: `experiment-peano-after`, based on frozen tag `v1`
(`8025bc6b4afee3415c652a5830576ef18aaf8cb5`). No parser, elaborator,
TypedAST, matcher, closure, runtime, CLI implementation, or resource limits change.
The only Rust additions are test harness code and its test-module registration.

## Interpretation and representation

For this experiment, choose `Z = At("zeroTime")` and `S(t) = After(t)`.
This interprets the existing structural time constructor as a Peano successor;
it does not redefine PropositionTime generally as discrete numerical time.
`derivedAt` is not the number being calculated.

`examples/experiments/peano_after.res` supplies unary operands as finite relations:

- `start(number, cursor)` identifies the initial cursor;
- `successor(number, cursor, next)` describes one input successor;
- `end(number, cursor)` explicitly identifies termination.

The `Natural` and `Cursor` constants are opaque identifiers. These input spines
encode the operands, not a table of sums. The fixture contains spines of lengths
zero through four and 25 addition requests, but no seeded sum results.

## Algorithm expressed in Residuum

`add(left, right, request) @ zeroTime` activates `Addition`.
Its first clause derives a `leftWalk` instance. Each input edge produces a new
`leftWalk` at `after t`, activating `LeftWalk` again. The explicit left endpoint
switches to `rightWalk` at the *same* `t`. `RightWalk` recursively adds one `after`
per edge of the right operand. The final endpoint derives `sum(request) @ t`.
All programs and rules participate in one `close_program` fixpoint.

The result of `2 + 3` is therefore:

```text
sum(r23) @ after(after(after(after(after(zeroTime)))))
```

The fixture itself only writes the existing source syntax `@ t` and `@ after t`.
Nested successors are constructed by inference, rather than parsed from a new
syntax or supplied by Rust. A separate `completed(request) @ zeroTime` observation
avoids pretending that the source parser already accepts nested temporal syntax.
No residual query is executed by this verifier; its experiment goal is an existing
source-defined observation used to inspect the forward closure.

## Tests and results

Three selected CLI-target Rust tests invoke existing APIs under `test_safe.py`:

1. All 25 pairs of operands zero through four produce the independently expected
   Peano terms. The checks include `sum(0,n) = n`,
   `sum(S(m),n) = S(sum(m,n))`, commutativity on these inputs, and `4 + 4 = 8`.
   Eight output successors are constructed despite input spines ending at four.
   There are 245 known events (including 45 seeds) and 200 justifications.
   Source-rule replay verifies witnesses and resulting proposition types/times;
   activation substitutions retain the complete nested time term.
   Provenance identifies Program, internal Rule, and Module. Changing the
   derivation clock preserves events and derivation records, changing only stamps.
2. Partial application of `LeftWalk(nTwo)` substitutes both internal RuleUses and
   leaves right operand, request, cursor and time as parameters. Further
   application equals direct application. Temporal beta reduction substitutes
   `zeroTime` beneath `after`. Local cursor binders remain local. Neither an absent
   request nor EvidentialNot of a request activates the programs.
3. The ordinary forward fixture runner rejects the recursive dependency graph.
   This expected rejection is verified without changing or bypassing its guard
   in the CLI. The computation above uses the existing core API in a bounded test
   process, not `residuum run`.

The runner retains 1 GiB total, no swap, one Cargo job, one harness thread,
timeout and preventive memory stops. Only these selected tests are authorized;
no full-suite run was made for this experiment. Rust supplies the independent
expected-number oracle and verifies results; it supplies no calculation rules,
addition facts, or synthesized arithmetic output to Residuum.

## Scope and termination

**EXPERIMENTAL OBSERVATION:** v1's core computes unary addition using associated
program activation and structural `after`, without integer primitives.
The finite cases do not establish a universal completeness or computability claim.

**Algorithm argument (restricted inputs):** when each operand has one finite,
acyclic, unbranched spine with its endpoint at its length, the left phase takes
exactly its length in successor steps, the phase switch takes none, and the right
phase takes exactly its length. Thus the terminal index is `S^(m+n)(Z)`.
No rule re-enters the left phase. This explains termination for this representation;
it is not a termination theorem for arbitrary Residuum programs. Malformed cycles,
branching spines or multiple endpoints are outside the claim.

The algorithm's clauses do not depend on particular operand constants or their
lengths. Larger inputs require supplying larger unary spines, not adding rules or
precalculating sums. Computational resource and source-size limits still apply.

**Language boundary:** the result is a constructed `TimeExpr` attached to an
Event, not an integer/`Natural` value usable directly as a coefficient argument.
This experiment does not provide signed integers, subtraction/cancellation,
comparisons, or a complete polynomial-addition implementation. It also does not
show addition taking two arbitrary nested `TimeExpr` values as term arguments:
the operands here are unary relational spines. Future numerical primitives may
be useful for values and efficiency, but their absence alone does not prove that
symbolic arithmetic computation is impossible in v1.

## Recorded verification

Final bounded run: `target/test-diagnostics/20261008T213603970188Z-3526298`.
All three selected tests passed individually. Arithmetic took 2.869 seconds,
partial application/activation 0.082 seconds, and the CLI boundary 0.082 seconds.
Peak reported test RSS was about 15.50 MiB. The initial compilation completed
under the unchanged controls (498.08 MiB reported peak RSS). No full suite ran.
The fixture and harness are experimental worktree changes, not changes to `main`
or the existing release tags.
