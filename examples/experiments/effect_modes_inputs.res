module EffectModesInputs

// Consumer of the declarations in legal/do_notdo_formal.res.
// The verifier supplies external object identities and a preinitialized Store.
seeded verb doingRequired(subject : Person, conduct : Conduct, object : Thing, beneficiary : Person)
seeded verb abstentionRequired(subject : Person, conduct : Conduct, object : Thing, beneficiary : Person)

rule constitutiveDoing(subject, conduct, object, beneficiary, t) =
    doingRequired(subject, conduct, object, beneficiary) @ t
    <= do(subject, conduct, object, beneficiary) @ after t

rule constitutiveAbstention(subject, conduct, object, beneficiary, t) =
    abstentionRequired(subject, conduct, object, beneficiary) @ t
    <= notDo(subject, conduct, object, beneficiary) @ after t
