module TemporalHistoryVocabulary

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// Shared dossier vocabulary of the temporal history experiment. LegalGeneral
// is imported unchanged. Dossier moments t0, t1, t2 are proposition indices;
// enact0, enact1, visitDate0 and saleDate1 are legal dates, used only as
// arguments (effective, notEarlier), never as proposition indices.

const enterWith : Conduct
const register : Conduct
const ana : Person
const bea : Person
const cal : Person
const host : Person
const rex : Thing
const fido : Thing
const dogBan : Fact
const regDuty : Fact
const permitFido : Fact
const visitAna0 : Fact
const sale1 : Fact
const t0 : PropositionTime
const t1 : PropositionTime
const t2 : PropositionTime
const enact0 : PropositionTime
const enact1 : PropositionTime
const visitDate0 : PropositionTime
const saleDate1 : PropositionTime

seeded verb accompanies(owner : Person, dog : Thing, venue : Person)
seeded verb propertyA(dog : Thing)
// Explicit legal datum: the norm identified by `norm` is in force with date
// `effective`. A generalization reads its vigencia from here; no position is
// produced with an invented date when the datum is absent.
seeded verb normEffective(norm : Fact, effective : PropositionTime)
