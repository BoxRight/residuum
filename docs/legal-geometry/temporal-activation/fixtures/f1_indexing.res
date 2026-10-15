module TemporalIndexing

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// F1. What the existing temporal operators do to a rule with no temporal scope.
// Fixture-local vocabulary: one seeded property of a dossier (accompanies) and
// probes for after. LegalGeneral is imported unchanged.

const enterWith : Conduct
const ana : Person
const host : Person
const rex : Thing
const dogBan : Fact
const visitAna : Fact
const enact0 : PropositionTime
const t0 : PropositionTime
const t1 : PropositionTime

seeded verb accompanies(owner : Person, dog : Thing, venue : Person)
seeded verb incidentReport(owner : Person, dog : Thing)
derived verb notified(owner : Person, dog : Thing)
derived verb reminded(owner : Person, dog : Thing)
derived verb oneStepProbe(owner : Person, dog : Thing)
derived verb twoStepProbe(owner : Person, dog : Thing)
derived verb incidentAtVisit(owner : Person, dog : Thing)

// G0: the general prohibition. Its lambda quantifies over t; no vigencia.
rule generalDogBan(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, enact0) @ t

// after probes: one step, then a second step built from the first.
rule notifyNext(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t
    <= notified(owner, dog) @ after t
rule remindNext(owner, dog, t) =
    notified(owner, dog) @ t
    <= reminded(owner, dog) @ after t
rule oneStep(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t & notified(owner, dog) @ after t
    <= oneStepProbe(owner, dog) @ t
rule twoSteps(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t & reminded(owner, dog) @ after t
    <= twoStepProbe(owner, dog) @ t

// Evidence at a later index adds a consequence at an earlier index.
rule retroFinding(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t & incidentReport(owner, dog) @ after t
    <= incidentAtVisit(owner, dog) @ t

// The same dossier property at three indices, and an entry at t1.
experiment indexing {
    seeds {
        accompanies(ana, rex, host) @ t0,
        accompanies(ana, rex, host) @ t1,
        accompanies(ana, rex, host) @ after t0,
        Act(visitAna, ana, enterWith, rex, host, private, t1) @ t1,
    }
    query residual violation(dogBan, visitAna, ana, rex, host, administrative, enact0) @ t1
}

// after is one constructor step: reminded @ after(after(t0)) is not matched
// by a pattern @ after t with t = t0.
experiment afterSteps {
    seeds {
        accompanies(ana, rex, host) @ t0,
    }
    query residual twoStepProbe(ana, rex) @ t0
}

experiment retroBase {
    seeds {
        accompanies(ana, rex, host) @ t0,
    }
    query residual incidentAtVisit(ana, rex) @ t0
}

experiment retroLate {
    seeds {
        accompanies(ana, rex, host) @ t0,
        incidentReport(ana, rex) @ after t0,
    }
    query residual incidentAtVisit(ana, rex) @ t0
}
