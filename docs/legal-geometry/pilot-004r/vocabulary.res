module Pilot004RVocabulary

import "../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// Pilot 004-R instance constants (docs/pilot-004r-generative-contract.md §3).
// Constants of existing LegalGeneral sorts only: no verb, entity or rule.
// LegalGeneral is imported unchanged.

// Brookside (DJ 202-14-282)
const jant : Person
const usa : Person
const premisesB : Thing
const saDocB : Thing
const policyA : Thing
const signB : Thing
const saB_j : Fact
const saB_u : Fact
const d20210216 : PropositionTime
const tB : PropositionTime

// Casa Fiesta (USAO EDKY release, 2020-08-06)
const casaFiesta : Person
const saDocCF : Thing
const policyCF : Thing
const signCF : Thing
const saCF_c : Fact
const saCF_u : Fact
const cfA1 : Fact
const cfA2 : Fact
const dCF : PropositionTime
const tCF : PropositionTime

// Flowers v. Prasad (B260140): no established representable atom.
const john : Person
const seth : Person
const bPrasad : Person
const vic : Person
const rPrasad : Person
const sappaniapillai : Person
const abraham : Person
const feesF : Thing

// Conducts named in the documents.
const agree : Conduct
const adoptPolicy : Conduct
const distribute : Conduct
const post : Conduct
const keepAvailable : Conduct
const sue : Conduct
