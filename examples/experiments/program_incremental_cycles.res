// Finite event cycle: a new proof of echo is not a new echo event.
// Imported programs retain their original partial application and module origins.
module ProgramIncrementalCycles
import "./dynamic_incremental_programs.res"

derived verb cycleEcho(token : Token)
rule enterCycle(token, t) = echo(token) @ t <= cycleEcho(token) @ t
rule returnCycle(token, t) = cycleEcho(token) @ t <= echo(token) @ t

experiment concurrent {
    seeds {
        oldSupport(alice) @ tau, oldSupport(bob) @ tau,
        request(alice) @ tau, request(bob) @ tau,
        alternativeSupport(alice) @ tau, alternativeSupport(bob) @ tau,
        lateSupport(alice) @ tau, lateSupport(bob) @ tau,
        ~request(alice) @ tau,
    }
    query residual complemented(bob) @ tau
}
