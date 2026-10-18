module SharedSpacesVocabulary

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// Descriptive vocabulary of situations about entering and using another
// person's space in the presence of third parties. Signature only: entities,
// constants and Seeded verbs. No rule, no Derived or Effect verb, no
// deontic or evaluative name. LegalGeneral is imported unchanged.

entity PersonTrait
entity ThingTrait
entity Bond

const minor : PersonTrait
const elderly : PersonTrait
const visuallyImpaired : PersonTrait
const mobilityImpaired : PersonTrait
const pregnant : PersonTrait

const animal : ThingTrait
const certifiedAssistance : ThingTrait
const vaccinated : ThingTrait
const food : ThingTrait
const electronic : ThingTrait
const motorized : ThingTrait
const large : ThingTrait
const registered : ThingTrait

const guardianOf : Bond
const employs : Bond
const livesWith : Bond
const clientOf : Bond
const presentWith : Bond

const enterWith : Conduct
const useInside : Conduct
const leaveUnattended : Conduct

// The conduct under examination in a dossier: the question, not a legal fact.
seeded verb considered(subject : Person, conduct : Conduct, object : Thing, beneficiary : Person)
seeded verb personTrait(person : Person, trait : PersonTrait)
seeded verb thingTrait(thing : Thing, trait : ThingTrait)
seeded verb sensitiveTo(person : Person, trait : ThingTrait)
seeded verb bond(from : Person, to : Person, kind : Bond)
seeded verb custody(person : Person, thing : Thing)

// Identifier and declared validity of the hypothetical norm under evaluation.
// Constant by construction during the ablation; never the dossier moment.
const hypNorm : Fact
const hypEnacted : PropositionTime
