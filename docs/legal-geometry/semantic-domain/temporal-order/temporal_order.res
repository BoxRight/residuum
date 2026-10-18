module TemporalOrder

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// Definitional program for dates represented as arguments. A date is a
// PropositionTime term after^n(r) over a root constant r; in these fixtures
// one application of after is one calendar year. The proposition index @ t of
// every atom is the dossier index and is never a date: dates occur only as
// arguments. No rule produces a normative position.

// Seeded description of a dossier: the dates it mentions.
seeded verb datePoint(date : PropositionTime)
seeded verb born(person : Person, date : PropositionTime)
seeded verb dossierDate(date : PropositionTime)

derived verb before(earlier : PropositionTime, later : PropositionTime)
derived verb notAfter(earlier : PropositionTime, later : PropositionTime)
derived verb majority(person : Person, date : PropositionTime)
derived verb minorAt(person : Person, date : PropositionTime)
derived verb adultAt(person : Person, date : PropositionTime)
derived verb bornAfterDossier(person : Person, date : PropositionTime)

// Every mentioned date and each of its predecessors is a date point. The
// closure only descends into subterms of mentioned dates.
rule bornPoint(p, b, t) = born(p, b) @ t <= datePoint(b) @ t
rule dossierPoint(d, t) = dossierDate(d) @ t <= datePoint(d) @ t
rule majorityPoint(p, m, t) = majority(p, m) @ t <= datePoint(m) @ t
rule descend(x, t) = datePoint(after(x)) @ t <= datePoint(x) @ t

// Strict order: y is reached from x by one or more applications of after,
// within the finite set of date points.
rule successor(x, t) = datePoint(after(x)) @ t <= before(x, after(x)) @ t
rule extend(x, y, t) = before(x, y) @ t & datePoint(after(y)) @ t <= before(x, after(y)) @ t

rule reflexive(x, t) = datePoint(x) @ t <= notAfter(x, x) @ t
rule strict(x, y, t) = before(x, y) @ t <= notAfter(x, y) @ t

// Majority date: eighteen years after birth (non-recursive; one term per birth).
rule majorityDate(p, b, t) = born(p, b) @ t <= majority(p, after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(b))))))))))))))))))) @ t

// Descriptive conditions at the dossier date d.
rule minor(p, b, m, d, t) =
    born(p, b) @ t & dossierDate(d) @ t & notAfter(b, d) @ t
    & majority(p, m) @ t & before(d, m) @ t
    <= minorAt(p, d) @ t
rule adult(p, m, d, t) =
    majority(p, m) @ t & dossierDate(d) @ t & notAfter(m, d) @ t
    <= adultAt(p, d) @ t
rule unborn(p, b, d, t) =
    born(p, b) @ t & dossierDate(d) @ t & before(d, b) @ t
    <= bornAfterDossier(p, d) @ t
