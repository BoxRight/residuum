module Pilot004Dossiers

import "./vocabulary.res"

// Four retrospective dossiers. Each records one act of entering with a thing
// into the sphere of the beneficiary, and descriptive facts about the
// participants. dossier1 is the base; each other dossier differs from it in
// one factor: the copresence relation between a third party and the
// beneficiary (dossier2, explicit evidence of non-copresence), the kind of
// the thing (dossier3), or a condition of the subject (dossier4).
// Proposition indices t1..t4 are dossier moments; date1..date4 are act dates
// used only as arguments. No valuation is attached.

const ana : Person
const bea : Person
const cal : Person
const obj : Thing
const act1 : Fact
const date1 : PropositionTime
const t1 : PropositionTime
const act2 : Fact
const date2 : PropositionTime
const t2 : PropositionTime
const act3 : Fact
const date3 : PropositionTime
const t3 : PropositionTime
const act4 : Fact
const date4 : PropositionTime
const t4 : PropositionTime

experiment dossier1 {
    seeds {
        Act(act1, ana, enterWith, obj, bea, private, date1) @ t1,
        kind(obj, animal) @ t1,
        condition(cal, allergic) @ t1,
        copresent(cal, bea) @ t1,
        copresent(bea, cal) @ t1,
    }
    query residual
    prohibition(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ t1
}

experiment dossier2 {
    seeds {
        Act(act2, ana, enterWith, obj, bea, private, date2) @ t2,
        kind(obj, animal) @ t2,
        condition(cal, allergic) @ t2,
        ~copresent(cal, bea) @ t2,
        ~copresent(bea, cal) @ t2,
    }
    query residual
    prohibition(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ t2
}

experiment dossier3 {
    seeds {
        Act(act3, ana, enterWith, obj, bea, private, date3) @ t3,
        kind(obj, food) @ t3,
        condition(cal, allergic) @ t3,
        copresent(cal, bea) @ t3,
        copresent(bea, cal) @ t3,
    }
    query residual
    prohibition(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ t3
}

experiment dossier4 {
    seeds {
        Act(act4, ana, enterWith, obj, bea, private, date4) @ t4,
        kind(obj, animal) @ t4,
        condition(cal, allergic) @ t4,
        copresent(cal, bea) @ t4,
        copresent(bea, cal) @ t4,
        condition(ana, visuallyImpaired) @ t4,
    }
    query residual
    prohibition(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ t4
}
