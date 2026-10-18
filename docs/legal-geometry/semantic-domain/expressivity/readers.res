module ContentReaders

import "./content.res"

// Probe readers, not hypotheses of the catalogue. They show that rules can
// distinguish an act with its content from either component alone, and can
// bind variables inside a Relation pattern, including the optional thing.

derived verb certifiedContent(act : Fact, thing : Thing, conduct : Conduct, beneficiary : Person)
derived verb relationalDebtor(act : Fact, debtor : Person, thing : Thing, creditor : Person)

rule sourcedCertification(k, i, o, c, b, e, t) =
    Act(k, i, certify, o, b, administrative, e) @ t & certifies(k, o, c, b) @ t
    <= certifiedContent(k, o, c, b) @ t

rule relationWithThing(k, d, x, cr, t) =
    constitutes(k, Relation { debtor = d, thing = x, conduct = care, creditor = cr, mode = Doing }) @ t
    <= relationalDebtor(k, d, x, cr) @ t
