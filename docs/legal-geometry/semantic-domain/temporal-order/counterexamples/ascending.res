module TemporalOrderAscending

import "../temporal_order.res"

// Counterexample: an ascending rule creates after(x) from x. The set of date
// points is no longer bounded by the subterms of mentioned dates, and the
// closure does not terminate.
rule ascend(x, t) = datePoint(x) @ t <= datePoint(after(x)) @ t
