module Pilot004Dossiers

import "./vocabulary.res"

// Contract v2. Four retrospective dossiers designed by
// tools/pilot004/design_dossiers.py with a declared label-free criterion:
// admissibility, a relational contrast (dossier1 and dossier2 differ only
// in copresence), literal coverage, then diversity of catalogue
// predictions. Proposition indices t1..t4 are dossier moments; date1..date4
// are act dates used only as arguments. No valuation is attached.
// Search objective (proposal, certified by the Rust tests): (True, 15, -22, -37)

const ana : Person
const bea : Person
const cal : Person
const dan : Person
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
        condition(ana, allergic) @ t1,
        ~condition(ana, visuallyImpaired) @ t1,
        ~condition(bea, allergic) @ t1,
        condition(bea, visuallyImpaired) @ t1,
        kind(obj, animal) @ t1,
        ~kind(obj, food) @ t1,
        copresent(ana, bea) @ t1,
        copresent(bea, ana) @ t1,
    }
    query residual
    prohibition(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ t1
}

experiment dossier2 {
    seeds {
        Act(act2, ana, enterWith, obj, bea, private, date2) @ t2,
        condition(ana, allergic) @ t2,
        ~condition(ana, visuallyImpaired) @ t2,
        ~condition(bea, allergic) @ t2,
        condition(bea, visuallyImpaired) @ t2,
        kind(obj, animal) @ t2,
        ~kind(obj, food) @ t2,
        ~copresent(ana, bea) @ t2,
        ~copresent(bea, ana) @ t2,
    }
    query residual
    prohibition(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ t2
}

experiment dossier3 {
    seeds {
        Act(act3, ana, enterWith, obj, bea, private, date3) @ t3,
        ~condition(ana, allergic) @ t3,
        condition(ana, visuallyImpaired) @ t3,
        ~condition(bea, allergic) @ t3,
        ~condition(bea, visuallyImpaired) @ t3,
        kind(obj, animal) @ t3,
        kind(obj, food) @ t3,
        copresent(ana, bea) @ t3,
        copresent(bea, ana) @ t3,
        copresent(ana, cal) @ t3,
        copresent(cal, ana) @ t3,
        ~copresent(ana, dan) @ t3,
        ~copresent(dan, ana) @ t3,
        ~copresent(bea, cal) @ t3,
        ~copresent(cal, bea) @ t3,
        copresent(bea, dan) @ t3,
        copresent(dan, bea) @ t3,
    }
    query residual
    prohibition(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ t3
}

experiment dossier4 {
    seeds {
        Act(act4, ana, enterWith, obj, bea, private, date4) @ t4,
        ~condition(ana, allergic) @ t4,
        condition(ana, visuallyImpaired) @ t4,
        condition(bea, allergic) @ t4,
        ~condition(bea, visuallyImpaired) @ t4,
        condition(cal, allergic) @ t4,
        ~kind(obj, animal) @ t4,
        ~kind(obj, food) @ t4,
        ~copresent(ana, bea) @ t4,
        ~copresent(bea, ana) @ t4,
        copresent(ana, cal) @ t4,
        copresent(cal, ana) @ t4,
        ~copresent(ana, dan) @ t4,
        ~copresent(dan, ana) @ t4,
        copresent(bea, cal) @ t4,
        copresent(cal, bea) @ t4,
        ~copresent(bea, dan) @ t4,
        ~copresent(dan, bea) @ t4,
    }
    query residual
    prohibition(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ t4
}
