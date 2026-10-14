module RiggsCandidateRules

import "./riggs_succession_reference.res"

// Generated bounded hypotheses. Never load all candidate rules as one theory.
// Impediment is a proposed legal conclusion, not an automatic removal of title.
derived verb hypotheticalSuccessionImpediment(will : Fact, testator : Person, beneficiary : Person, object : Thing, authority : Authority, effective : PropositionTime)

rule candidateImpediment00(will, testator, beneficiary, object, authority, effective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment01(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment02(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment03(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment04(will, testator, beneficiary, object, authority, effective, act, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment05(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment06(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment07(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment08(will, testator, beneficiary, object, authority, effective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment09(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment10(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment11(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment12(will, testator, beneficiary, object, authority, effective, act, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment13(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment14(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment15(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment16(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment17(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment18(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment19(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment20(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment21(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment22(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment23(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment24(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment25(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment26(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment27(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment28(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment29(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment30(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment31(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment32(will, testator, beneficiary, object, authority, effective, act, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment33(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment34(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment35(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment36(will, testator, beneficiary, object, authority, effective, act, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment37(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment38(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment39(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment40(will, testator, beneficiary, object, authority, effective, act, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment41(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment42(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment43(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment44(will, testator, beneficiary, object, authority, effective, act, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment45(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment46(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment47(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment48(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment49(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment50(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment51(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment52(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment53(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment54(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment55(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment56(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment57(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment58(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment59(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment60(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment61(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment62(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t

rule candidateImpediment63(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & patrimonialAsset(beneficiary, object, will, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & ~acquisitionPurposeRecorded(act, will, beneficiary) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
