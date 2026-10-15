"""Generate the F4 epoch-regime fixtures: T0 ⊆ T1 ⊆ T2 ⊆ T3 by import, one shared panel."""
from pathlib import Path
OUT = Path(__file__).resolve().parent / "fixtures"

HEADER = """module {module}

import "{imp}"

"""

T0_DECLS = """// F4, theory T0. Learning epochs e0..e3 are PropositionTime constants used as
// ARGUMENTS; legal dossier times sA..sF are proposition indices. They are kept
// distinct. A dossier is evaluated under an epoch by an explicit seed
// evaluatedAt(e). Every generalization is written, from T0 on, with the same
// guard unrevisedThrough(itself, e): the theory states for which epochs it
// keeps that generalization. A later theory only ADDS rules and certificates.
// The epoch is carried in the effective argument of the consequence, so the
// assessments under different epochs are distinct logical facts.

const enterWith : Conduct
const ana : Person
const bea : Person
const cal : Person
const dan : Person
const eva : Person
const host : Person
const rex : Thing
const fido : Thing
const kim : Thing
const max : Thing
const pip : Thing
const g0 : Fact
const g0a : Fact
const g3 : Fact
const e0 : PropositionTime
const e1 : PropositionTime
const e2 : PropositionTime
const e3 : PropositionTime
const sA : PropositionTime
const sB : PropositionTime
const sC : PropositionTime
const sD : PropositionTime
const sE : PropositionTime
const sF : PropositionTime

seeded verb accompanies(owner : Person, dog : Thing, venue : Person)
seeded verb propertyA(dog : Thing)
seeded verb allergicPresent(person : Person, venue : Person)
seeded verb serviceDog(dog : Thing, owner : Person)
seeded verb evaluatedAt(epoch : PropositionTime)
derived verb unrevisedThrough(generalization : Fact, epoch : PropositionTime)

// G0: the general prohibition, with the epoch guard.
rule g0Ban(owner, dog, venue, e, t) =
    accompanies(owner, dog, venue) @ t & evaluatedAt(e) @ t & unrevisedThrough(g0, e) @ t
    <= prohibition(g0, owner, enterWith, dog, venue, administrative, e) @ t

rule certG0E0(t) = evaluatedAt(e0) @ t <= unrevisedThrough(g0, e0) @ t
"""

T1_DECLS = """// F4, theory T1 = T0 + after experience 1 (a dog with A; Prohibition not in N).
// G0a refines G0 on explicit evidence of not-A. No certificate for g0 at e1
// is added: that withholding is a decision of the learner, not of a rule.
rule g0aBan(owner, dog, venue, e, t) =
    accompanies(owner, dog, venue) @ t & ~propertyA(dog) @ t
    & evaluatedAt(e) @ t & unrevisedThrough(g0a, e) @ t
    <= prohibition(g0a, owner, enterWith, dog, venue, administrative, e) @ t

rule certG0aE1(t) = evaluatedAt(e1) @ t <= unrevisedThrough(g0a, e1) @ t
"""

T2_DECLS = """// F4, theory T2 = T1 + after experience 2 (allergic person present; Right not
// in N). No rule of T1 derives right, so the valuation already holds: only the
// certificate of g0a is extended. The valuation stays as a constraint.
rule certG0aE2(t) = evaluatedAt(e2) @ t <= unrevisedThrough(g0a, e2) @ t
"""

T3_DECLS = """// F4, theory T3 = T2 + after experience 3 (service dog; Right in N).
rule g3ServiceRight(owner, dog, venue, e, t) =
    accompanies(owner, dog, venue) @ t & serviceDog(dog, owner) @ t
    & evaluatedAt(e) @ t & unrevisedThrough(g3, e) @ t
    <= right(g3, owner, enterWith, dog, venue, legislative, e) @ t

rule certG0aE3(t) = evaluatedAt(e3) @ t <= unrevisedThrough(g0a, e3) @ t
rule certG3E3(t) = evaluatedAt(e3) @ t <= unrevisedThrough(g3, e3) @ t
"""

# name, seeds, goal (label only)
PANEL = [
    ("s1Dossier", ["accompanies(ana, rex, host) @ sA", "propertyA(rex) @ sA",
                   "evaluatedAt(e0) @ sA", "evaluatedAt(e1) @ sA", "evaluatedAt(e3) @ sA"],
     "prohibition(g0, ana, enterWith, rex, host, administrative, e0) @ sA"),
    ("plainDog", ["accompanies(bea, fido, host) @ sB", "~propertyA(fido) @ sB",
                  "evaluatedAt(e0) @ sB", "evaluatedAt(e1) @ sB", "evaluatedAt(e2) @ sB", "evaluatedAt(e3) @ sB"],
     "prohibition(g0a, bea, enterWith, fido, host, administrative, e1) @ sB"),
    ("unknownA", ["accompanies(bea, fido, host) @ sC", "evaluatedAt(e0) @ sC", "evaluatedAt(e1) @ sC"],
     "prohibition(g0a, bea, enterWith, fido, host, administrative, e1) @ sC"),
    ("s2Dossier", ["accompanies(cal, kim, host) @ sD", "~propertyA(kim) @ sD", "allergicPresent(dan, host) @ sD",
                   "evaluatedAt(e2) @ sD", "evaluatedAt(e3) @ sD"],
     "right(g3, cal, enterWith, kim, host, legislative, e3) @ sD"),
    ("s3Dossier", ["accompanies(eva, max, host) @ sE", "~propertyA(max) @ sE", "serviceDog(max, eva) @ sE",
                   "evaluatedAt(e2) @ sE", "evaluatedAt(e3) @ sE"],
     "right(g3, eva, enterWith, max, host, legislative, e3) @ sE"),
    ("s4Question", ["accompanies(eva, pip, host) @ sF", "~propertyA(pip) @ sF", "serviceDog(pip, eva) @ sF",
                    "allergicPresent(dan, host) @ sF", "evaluatedAt(e3) @ sF"],
     "right(g3, eva, enterWith, pip, host, legislative, e3) @ sF"),
    ("twoDogs", ["accompanies(ana, rex, host) @ sA", "propertyA(rex) @ sA",
                 "accompanies(bea, fido, host) @ sA", "~propertyA(fido) @ sA", "evaluatedAt(e1) @ sA"],
     "prohibition(g0a, bea, enterWith, fido, host, administrative, e1) @ sA"),
]


def panel():
    out = ["\n// Shared panel: identical seeds in every theory T0..T3.\n"]
    for name, seeds, goal in PANEL:
        body = "".join(f"        {s},\n" for s in seeds)
        out.append(f"experiment {name} {{\n    seeds {{\n{body}    }}\n    query residual {goal}\n}}\n\n")
    return "".join(out)


LEGAL = "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"
files = [("f4_t0", "EpochT0", LEGAL, T0_DECLS), ("f4_t1", "EpochT1", "f4_t0.res", T1_DECLS),
         ("f4_t2", "EpochT2", "f4_t1.res", T2_DECLS), ("f4_t3", "EpochT3", "f4_t2.res", T3_DECLS)]
for stem, module, imp, decls in files:
    (OUT / f"{stem}.res").write_text(HEADER.format(module=module, imp=imp) + decls + panel())
print("written")
