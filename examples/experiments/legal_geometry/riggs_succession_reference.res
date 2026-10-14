module RiggsSuccessionReference

import "./ontology_reference/legal_general.res"

// Delimited literal-reference theory for the historical experiment.
// operativeDisposition positively records an admitted, operative disposition.
// It bundles validity/revocation questions as an explicit external assumption.
// No inference from absence of revocation. Each rule preserves the observation
// index; effective is the instrument's symbolic date, not the death date.
seeded verb operativeDisposition(will : Fact, testator : Person, beneficiary : Person, object : Thing, authority : Authority, effective : PropositionTime)
seeded verb deathRecorded(death : Fact, testator : Person, authority : Authority, effective : PropositionTime)
seeded verb survivorRecorded(beneficiary : Person, testator : Person)

// Separate, externally established relations for evaluating arguments.
// A victim is explicitly a victim, not the beneficiary argument of Act.
seeded verb intentionalKilling(act : Fact, killer : Person, victim : Person, authority : Authority, effective : PropositionTime)
seeded verb deathCausedBy(act : Fact, death : Fact)
seeded verb acquisitionPurposeRecorded(act : Fact, will : Fact, beneficiary : Person)

derived verb testamentaryEntitlement(will : Fact, testator : Person, beneficiary : Person, object : Thing, authority : Authority, effective : PropositionTime)
derived verb acquisitionLinkedToWrong(will : Fact, act : Fact, death : Fact, beneficiary : Person, object : Thing, authority : Authority, effective : PropositionTime)
derived verb purposefulAcquisition(will : Fact, act : Fact, death : Fact, beneficiary : Person, object : Thing, authority : Authority, effective : PropositionTime)

rule literalTestamentaryEntitlement(will, testator, beneficiary, object, authority, effective, death, deathAuthority, deathEffective, t) =
    operativeDisposition(will, testator, beneficiary, object, authority, effective) @ t
    & deathRecorded(death, testator, deathAuthority, deathEffective) @ t
    & survivorRecorded(beneficiary, testator) @ t
    <= testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t

// The attribution is under the literal-reference assumptions, not the judgment.
// Reuse the general patrimonial/title/capacity chain without rewriting it.
rule literalEntitlementAsset(will, testator, beneficiary, object, authority, effective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    <= patrimonialAsset(beneficiary, object, will, authority, effective) @ t

rule acquisitionCausalLink(will, testator, beneficiary, object, authority, effective, death, deathAuthority, deathEffective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & deathRecorded(death, testator, deathAuthority, deathEffective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & deathCausedBy(act, death) @ t
    <= acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t

rule acquisitionPurposeLink(will, act, death, beneficiary, object, authority, effective, t) =
    acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= purposefulAcquisition(will, act, death, beneficiary, object, authority, effective) @ t
