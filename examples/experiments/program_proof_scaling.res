// Fixed event count, different numbers of distinct RuleUse origins.
// All outputs are derived from a request; there are no expected answers in seeds.
module ProgramProofScaling
entity Token
const alice : Token
const tau : PropositionTime
derived verb supported(token : Token)
rule produce(token, t) = I <= supported(token) @ t

seeded verb request1(token : Token) = program Package1
program Package1(token : Token, t : PropositionTime) {
    use produce { token = token, t = t }
}
experiment proofs1 {
    seeds { request1(alice) @ tau, }
    query residual supported(alice) @ tau
}

seeded verb request4(token : Token) = program Package4
program Package4(token : Token, t : PropositionTime) {
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
}
experiment proofs4 {
    seeds { request4(alice) @ tau, }
    query residual supported(alice) @ tau
}

seeded verb request16(token : Token) = program Package16
program Package16(token : Token, t : PropositionTime) {
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
    use produce { token = token, t = t }
}
experiment proofs16 {
    seeds { request16(alice) @ tau, }
    query residual supported(alice) @ tau
}
