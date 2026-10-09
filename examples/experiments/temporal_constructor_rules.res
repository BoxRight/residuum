module TemporalConstructorRules
import "./peano_after.res"

entity Polynomial
entity TimeBox { value : PropositionTime }
entity TimeEnvelope { payload : TimeBox }
entity TimeShelf { entries : Set TimeEnvelope }
const shelf : TimeShelf
const polynomial : Polynomial

derived verb coefficient(polynomial : Polynomial, exponent : PropositionTime, value : PropositionTime)
derived verb verified(polynomial : Polynomial, value : PropositionTime)
derived verb packed(value : TimeEnvelope)
derived verb unpacked(value : PropositionTime)
derived verb predecessor(value : PropositionTime)
derived verb observed(value : PropositionTime)
derived verb indexed(value : PropositionTime)
seeded verb inspect(value : PropositionTime) = program Echo

// The only coefficient here comes from an actually computed sum, never a seed.
rule publish(t) =
    sum(r23) @ t <= coefficient(polynomial, after(after(zeroTime)), t) @ zeroTime
rule resume(value) =
    coefficient(polynomial, after(after(zeroTime)), value) @ zeroTime
    <= add(nZero, nTwo, r02) @ value
rule verify(t) = sum(r02) @ t <= verified(polynomial, t) @ zeroTime
rule pack(value) =
    coefficient(polynomial, after(after(zeroTime)), value) @ zeroTime
    <= packed(TimeEnvelope { payload = TimeBox { value = after(after(value)), }, }) @ zeroTime
rule unpack(value) =
    packed(TimeEnvelope { payload = TimeBox { value = value, }, }) @ zeroTime
    <= unpacked(value) @ value
rule peel(value) =
    coefficient(polynomial, after(after(zeroTime)), after(value)) @ zeroTime
    <= predecessor(value) @ value

rule emit(value, t) = I <= observed(after(after(value))) @ t
rule emitRecord(value, t) =
    I <= packed(TimeEnvelope { payload = TimeBox { value = after(after(value)), }, }) @ t
rule echoIndex(value, t) = observed(value) @ t <= indexed(value) @ value
rule partialEmit = emit(zeroTime)
rule groundEmit = partialEmit(zeroTime)
rule directEmit = emit(zeroTime, zeroTime)
program Echo(value : PropositionTime, t : PropositionTime) {
    use emit { value = value, t = t }
    use emitRecord { value = value, t = t }
}
program PartialEcho = Echo(zeroTime)
program GroundEcho = PartialEcho(zeroTime)
program DirectEcho = Echo(zeroTime, zeroTime)
// Keep comparison lambdas passive; there are no dormant seeds.
program Inactive(value : PropositionTime, t : PropositionTime) {
    use partialEmit { t = t }
    use groundEmit {}
    use directEmit {}
}
seeded verb dormant(value : PropositionTime) = program Inactive

// Execution is explicit; this Effect only becomes known during forward closure.
effect verb retain(subject : TimeShelf, value : PropositionTime) =>
    subject.entries += TimeEnvelope { payload = TimeBox { value = after(value), }, }
rule retainComputed(value) =
    coefficient(polynomial, after(after(zeroTime)), value) @ zeroTime
    <= retain(shelf, value) @ after zeroTime

// Constructors inside RuleUse bindings, including a temporal binding.
program ConstructedTime(value : PropositionTime, t : PropositionTime) {
    use emit { value = after(value), t = after(t) }
    use emitRecord { value = after(value), t = after(t) }
}
program PartialConstructed = ConstructedTime(zeroTime)
program GroundConstructed = PartialConstructed(zeroTime)
program DirectConstructed = ConstructedTime(zeroTime, zeroTime)
seeded verb aligned(value : PropositionTime)
derived verb alignedResult(value : PropositionTime)
rule alignment(value) = aligned(value) @ value <= alignedResult(value) @ value
