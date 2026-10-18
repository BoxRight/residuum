module Pilot004Vocabulary

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// Pilot 004 descriptive vocabulary. Signature only: three general extensions
// over LegalGeneral (copresence of persons, a recorded condition of a person,
// a recorded kind of a thing), their constants, and the constants of the
// hypothetical norm. No rule, no Derived or Effect verb, no deontic or
// evaluative name. LegalGeneral is imported unchanged.

entity PersonCondition
entity ThingKind

const allergic : PersonCondition
const visuallyImpaired : PersonCondition
const animal : ThingKind
const food : ThingKind
const enterWith : Conduct

seeded verb copresent(first : Person, second : Person)
seeded verb condition(person : Person, condition : PersonCondition)
seeded verb kind(thing : Thing, kind : ThingKind)

// Identifier and declared validity of the hypothetical norm under evaluation;
// constant by construction, never the dossier index or an act date.
const hypNorm : Fact
const hypEnacted : PropositionTime
