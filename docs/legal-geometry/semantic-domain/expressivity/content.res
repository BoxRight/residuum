module ContentExpressivity

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// Expressivity probe: content of acts, separate from the acts themselves.
// constitutes(act, r): the act identified by `act` constitutes or attests the
// relational content r. certifies(act, thing, conduct, beneficiary): the act
// identified by `act` certifies that `thing` is apt for `conduct` in favour of
// `beneficiary`. Neither verb is read by any rule of LegalGeneral; no
// normative effect is attributed to them.

seeded verb constitutes(act : Fact, content : Relation)
seeded verb certifies(act : Fact, thing : Thing, conduct : Conduct, beneficiary : Person)

const ana : Person
const bea : Person
const cal : Person
const issuer : Person
const obj : Thing
const care : Conduct
const certify : Conduct
const guide : Conduct
const appointment : Fact
const service : Fact
const cert : Fact
const certDate : PropositionTime
const t0 : PropositionTime

experiment relationalContent {
    seeds {
        constitutes(appointment, Relation { debtor = cal, thing = none, conduct = care, creditor = bea, mode = Doing }) @ t0,
        constitutes(service, Relation { debtor = ana, thing = obj, conduct = care, creditor = bea, mode = Doing }) @ t0,
    }
    query residual
    obligation(appointment, cal, care, obj, bea, private, certDate) @ t0
}

experiment actWithContent {
    seeds {
        Act(cert, issuer, certify, obj, ana, administrative, certDate) @ t0,
        certifies(cert, obj, guide, ana) @ t0,
    }
    query residual
    obligation(cert, issuer, certify, obj, ana, administrative, certDate) @ t0
}

experiment actWithoutContent {
    seeds {
        Act(cert, issuer, certify, obj, ana, administrative, certDate) @ t0,
    }
    query residual
    obligation(cert, issuer, certify, obj, ana, administrative, certDate) @ t0
}

experiment contentWithoutAct {
    seeds {
        certifies(cert, obj, guide, ana) @ t0,
    }
    query residual
    obligation(cert, issuer, certify, obj, ana, administrative, certDate) @ t0
}
