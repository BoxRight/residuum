# Active verb programs: forward experiment

`close_program(seeds, module, clock)` reads the existing typed `Verb.program`
association. It uses the same Horn fixpoint, matching, substitution, logical
event identity, proof deduplication, and external derivation clock as `close`.
The rules-only `close(seeds, rules, clock)` retains its existing behavior.

In the module-aware entry, a rule referenced by any `Verb.program` is a program
definition rather than an ordinary globally active rule. Explicit rule aliases
remain ordinary rules. Positive known instances of Seeded and Derived verbs
activate their associated definition once per logical event. Effect associations
remain inactive in this experiment.

Activation binds the lambda's parameters positionally to the instance arguments
and binds its final PropositionTime parameter to the instance's time. Its body
must still match Known with those bindings. Activated calls persist while new
evidence arrives, so later compatible premises can enable a previously waiting
call. A body `I` needs no additional evidence, but still requires the triggering
instance; a declaration alone cannot activate it. `~sells(...)` does not activate
the positive program.

Provenance retains the program rule name, complete substitution, and body
witnesses. If the body does not already cite the triggering instance, activation
prepends that instance to its witnesses. For the original self-anchored
`sellsProgram`, proof records match the ordinary rule baseline exactly. For a
derived `sells`, the upstream proof and incoming witness category differ from a
real seed; downstream events, substitutions, proposition times, and logical
proof records agree. DerivationTime comes from the external clock and its stamp
ordering can differ when activation schedules a rule in a later round.

The fixture chain preserves the baseline's buyer as owner:

```text
sells(juan, car, pedro) @ tau
  -> give(juan, car, pedro) @ after(tau)
  -> owns(pedro, car) @ after(tau)
  -> notDo(charlie, molest, car, pedro) @ after(after(tau))
```

The new forward observation mode uses existing experiment seed blocks and goals:

```sh
python3 scripts/test_safe.py --forward --fixture examples/experiments/sells_associated_program.res
python3 scripts/test_safe.py --forward --fixture examples/experiments/sells_derived_associated_program.res
python3 scripts/test_safe.py --forward --fixture examples/experiments/sells_unit_associated_program.res
```

Here the goal is an observation of forward Known; the mode does not run the
block's residual query. No syntax has been added. The runner conservatively
requires an acyclic dependency graph including implicit activation guards.
The same cgroup, timeout, memory, swap, and job/thread limits apply.

Controls in `associated_program_waits.res` exercise evidence arriving after
activation and reject a witness at a different time. `effect_program_inactive.res`
derives an Effect instance without activating its associated program.

No state transformation is executed. Residual search, abduction, defaults,
FOUR semantics, and incremental inference keep their existing rules-only paths.
Forward support for associations does not yet extend those paths.

## Composition of associated programs

The follow-up [composition structure experiment](composition-structure-experiment.md)
compares staged, associated, inline, and incremental behavior; defines a finite
operational order; and tests precisely scoped observation adjunctions. It adds
no production mechanism.

`sells_composed_programs.res` isolates two active associations using only the
existing declarations and rules:

```text
sells.program = sellsProgram
owns.program = protectionFromCharlie
protectionFromCharlie = ownsProgram(charlie, molest)
```

The first program derives `owns` directly; the new Derived instance enters Known
and activates the second, beta-specialized program in a later round:

```text
sells(juan, car, pedro) @ tau
  -> owns(pedro, car) @ after(tau)
  -> notDo(charlie, molest, car, pedro) @ after(after(tau))
```

This small fixture omits `give` to isolate a program producing the verb that
activates the next program. It does not change the existing sells/give experiment
or Legal. No forward engine changes or new activation mechanisms are needed.

`sells_composed_inline.res` expresses the same behavior with two ordinary rules,
without associations and with the specialized protection rule written inline.
Tests compare events including operational metadata, rule names, substitutions,
provenance witnesses, and the whole beta-reduced protection rule. DerivationTime
is supplied externally; the associated path must stamp ownership before its
protection consequence, while exact stamp labels need not match the inline
schedule.

The controls include absent and negative-only inputs, two sales with different
buyers and proposition times, and repeated copies of the same input seeds.
Without activation and its program definitions, the remaining ordinary rule
cannot derive either ownership or protection.

```sh
python3 scripts/test_safe.py --forward --fixture examples/experiments/sells_composed_programs.res
```

The two composition tests and two selected Seeded/Derived regressions passed
individually. Compilation peaked at 476 MiB RSS and tests at roughly 15 MiB.
The composed fixture also passed through the external forward runner, including
the absent, negative-only, and two-instance controls. The full suite was not run.
Reports: `target/test-diagnostics/20261001T043559050808Z-1058484/` (tests) and
`target/test-diagnostics/20261001T043649815965Z-1059281/` (fixture).

## Verification

Four new tests and five selected regressions passed individually through
`scripts/test_safe.py`; the full suite was not run. The three sells fixtures above
also passed through the external forward runner. Test compilation peaked at
536 MiB RSS, example compilation at 442 MiB, and execution at roughly 15 MiB.
All checks retained 1 GiB total, no swap, one Cargo job, one harness thread, and
the existing timeouts.

Reports:
- New tests: `target/test-diagnostics/20261001T042741896698Z-1050960/`
- Regressions: `target/test-diagnostics/20261001T042828532651Z-1051744/`
- Seed fixture: `target/test-diagnostics/20261001T042846362745Z-1052104/`
- Derived fixture: `target/test-diagnostics/20261001T042927811116Z-1052797/`
- Unit control: `target/test-diagnostics/20261001T042940733579Z-1052896/`
