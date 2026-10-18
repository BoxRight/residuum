module TemporalOrderIndexAsDate

import "../temporal_order.res"

// Counterexample to the separation contract: the dossier date is used as the
// proposition index. It elaborates, so the separation is checked structurally.
derived verb dossierAt(date : PropositionTime)
rule indexAsDate(d) = dossierDate(d) @ d <= dossierAt(d) @ d
