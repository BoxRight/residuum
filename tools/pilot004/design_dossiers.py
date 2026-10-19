"""Deterministic structural design of the Pilot 004 dossiers (contract v2).

The four dossiers are chosen by a declared, label-free criterion over the
frozen candidate catalogue (docs/pilot-004-contract.md section 3):

  hard  K1-K5 admissibility; one Act anchor per dossier; copresence recorded
        in both orientations; dossier1 and dossier2 have identical individual
        attributes and differ only in copresence (relational contrast);
        every grammar literal that can stand alone is satisfied in at least
        one dossier and unsatisfied in at least one;
  then  maximise the number of distinct prediction patterns of the catalogue,
        minimise the number of candidates deriving nothing, minimise the
        number of seeded facts.

The search is a seeded local search; its matcher only proposes a design.
The frozen numbers are certified with close_program by the Rust tests.
No valuation is read. Run: python3 -B tools/pilot004/design_dossiers.py
"""
import itertools
import random
from pathlib import Path

import generate_catalog as g

FROZEN = Path(__file__).resolve().parents[2] / "docs/legal-geometry/pilot-004/frozen"
PERSONS = ["ana", "bea", "cal", "dan"]
SUBJECT, BENEFICIARY, THING = "ana", "bea", "obj"
SEED, RESTARTS, ITERATIONS = 4004, 60, 6000

UNITS = ([("condition", (p, k)) for p in PERSONS for k in g.CONDITIONS]
         + [("kind", (THING, k)) for k in g.KINDS]
         + [("copresent", pq) for pq in itertools.combinations(PERSONS, 2)])
COPRESENCE = [i for i, u in enumerate(UNITS) if u[0] == "copresent"]


def holds(dossier, verb, args, negative):
    if verb == "copresent":
        if args[0] == args[1]:
            return False
        key = ("copresent", tuple(sorted(args, key=PERSONS.index)))
    else:
        key = (verb, args)
    return dossier.get(key, "?") == ("~" if negative else "+")


def fires(body, dossier):
    for x in PERSONS:  # Formal has no inequality: x ranges over every person
        env = {"s": SUBJECT, "b": BENEFICIARY, "x": x, "o": THING}
        if all(holds(dossier, v, tuple(env.get(a, a) for a in args), n) for v, args, n in body):
            return True
    return False


def dossiers_of(state):
    d1, d2_copresence, d3, d4 = state
    d2 = list(d1)
    for j, i in enumerate(COPRESENCE):
        d2[i] = d2_copresence[j]
    return [d1, d2, d3, d4]


def score(designs, bodies, lits):
    ds = [{UNITS[i]: s for i, s in enumerate(d) if s != "?"} for d in designs]
    patterns = [tuple(fires(b, d) for d in ds) for b in bodies]
    standalone = [l for l in lits if not ("x" in l[1] and l[0] != "copresent")]
    covered = all(any(fires((l,), d) for d in ds) and not all(fires((l,), d) for d in ds)
                  for l in standalone)
    contrast = designs[0] != designs[1]
    inert = sum(1 for p in patterns if not any(p))
    facts = sum(1 for d in designs for s in d if s != "?")
    return (covered and contrast, len(set(patterns)), -inert, -facts)


def search():
    lits = g.literals()
    bodies = [b for n in (1, 2) for b in itertools.combinations(lits, n) if g.admissible(b)]
    rng = random.Random(SEED)
    best = None
    for _ in range(RESTARTS):
        state = [[rng.choice("+~?") for _ in UNITS], [rng.choice("+~?") for _ in COPRESENCE],
                 [rng.choice("+~?") for _ in UNITS], [rng.choice("+~?") for _ in UNITS]]
        current = score(dossiers_of(state), bodies, lits)
        for _ in range(ITERATIONS):
            vector = state[rng.randrange(4)]
            i = rng.randrange(len(vector))
            old = vector[i]
            vector[i] = rng.choice([c for c in "+~?" if c != old])
            candidate = score(dossiers_of(state), bodies, lits)
            if candidate >= current:
                current = candidate
            else:
                vector[i] = old
        if best is None or current > best[0]:
            best = (current, [list(v) for v in state])
    return best


def render(score_value, designs):
    out = ["module Pilot004Dossiers", "", 'import "./vocabulary.res"', "",
           "// Contract v2. Four retrospective dossiers designed by",
           "// tools/pilot004/design_dossiers.py with a declared label-free criterion:",
           "// admissibility, a relational contrast (dossier1 and dossier2 differ only",
           "// in copresence), literal coverage, then diversity of catalogue",
           "// predictions. Proposition indices t1..t4 are dossier moments; date1..date4",
           "// are act dates used only as arguments. No valuation is attached.",
           f"// Search objective (proposal, certified by the Rust tests): {score_value}", ""]
    out += [f"const {p} : Person" for p in PERSONS] + [f"const {THING} : Thing"]
    for n in range(1, 5):
        out += [f"const act{n} : Fact", f"const date{n} : PropositionTime", f"const t{n} : PropositionTime"]
    out.append("")
    for n, design in enumerate(designs, start=1):
        out += [f"experiment dossier{n} {{", "    seeds {",
                f"        Act(act{n}, {SUBJECT}, enterWith, {THING}, {BENEFICIARY}, private, date{n}) @ t{n},"]
        for (verb, args), state in zip(UNITS, design):
            if state == "?":
                continue
            neg = "~" if state == "~" else ""
            if verb == "copresent":
                out.append(f"        {neg}copresent({args[0]}, {args[1]}) @ t{n},")
                out.append(f"        {neg}copresent({args[1]}, {args[0]}) @ t{n},")
            else:
                out.append(f"        {neg}{verb}({args[0]}, {args[1]}) @ t{n},")
        out += ["    }", "    query residual",
                f"    prohibition(hypNorm, {SUBJECT}, enterWith, {THING}, {BENEFICIARY}, administrative, hypEnacted) @ t{n}",
                "}", ""]
    return "\n".join(out).rstrip() + "\n"


def main():
    value, state = search()
    designs = dossiers_of(state)
    (FROZEN / "dossiers.res").write_text(render(value, designs))
    print(value)


if __name__ == "__main__":
    main()
