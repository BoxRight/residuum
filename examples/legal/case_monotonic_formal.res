module LegalMonotonicCase

// Expediente monotónico: todos los hechos y todas las reglas están aquí.
// Ejecutar:
// python3 scripts/test_safe.py --fixture examples/legal/case_monotonic_formal.res --forward
// Las queries existentes sólo nombran observaciones en modo --forward:
// no se hace búsqueda residual, evaluación de defaults ni ejecución de State.

entity Person
entity Thing
entity Conduct
entity Authority
entity Fact

const juan : Person
const pedro : Person
const legislator : Person
const car : Thing
const boat : Thing
const house : Thing
const book : Thing
const bicycle : Thing
const legislativeSeat : Thing
const transfer : Conduct
const deliver : Conduct
const molest : Conduct
const enact : Conduct
const use : Conduct

const private : Authority
const legislative : Authority
const judicial : Authority
const administrative : Authority
const constitutional : Authority

// oldDate y newDate son fechas simbólicas de vigencia, no números de ronda.
// tau indexa las proposiciones del expediente observado. No se identifica con
// la vigencia ni con DerivationTime. La comparación de fechas es un dato;
// no se presupone un orden de calendario ni que after sea sucesor discreto.
const oldDate : PropositionTime
const newDate : PropositionTime
const tau : PropositionTime

const transferDuty : Fact
const transferRight : Fact
const transferAct : Fact
const deliveryDuty : Fact
const deliveryRight : Fact
const deliveryAct : Fact
const noMolesting : Fact
const molestingAct : Fact
const noOmission : Fact
const holding : Fact
const grantedPower : Fact
const enactment : Fact
const statutoryDuty : Fact
const carPermission : Fact
const carProhibition : Fact
const boatDuty : Fact
const boatProhibition : Fact
const housePermission : Fact
const houseProhibition : Fact
const seatDuty : Fact
const seatProhibition : Fact
const olderPermission : Fact
const newerProhibition : Fact
const sameDatePermission : Fact
const sameDateProhibition : Fact

// Act y CounterAct son predicados positivos distintos con los mismos roles.
// CounterAct no es EvidentialNot: tampoco la ausencia de Act lo satisface.
// Fact identifica el documento/hecho; no reifica una proposición del core.
// La autoridad y la vigencia se conservan en los argumentos y en provenance.
seeded verb Act(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb CounterAct(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb right(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb obligationFiled(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb prohibition(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb omissionProhibition(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb permission(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb extinguished(id : Fact, authority : Authority, effective : PropositionTime)

// Relaciones de comparación aportadas como datos del caso. Sólo se usan las
// comparaciones explícitamente acreditadas; no hay ranking especial en Rust.
seeded verb higherAuthority(higher : Authority, lower : Authority)
seeded verb notEarlier(later : PropositionTime, earlier : PropositionTime)

seeded verb patrimonialHolding(id : Fact, owner : Person, object : Thing, authority : Authority, effective : PropositionTime)
seeded verb powerGranted(id : Fact, holder : Person, conduct : Conduct, object : Thing, authority : Authority, effective : PropositionTime)
seeded verb legislationText(act : Fact, norm : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)

// obligationFiled es el ingreso externo. obligation es la conclusión Derived
// común a ese ingreso y a las obligaciones que produzca law.
derived verb obligation(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb compliance(norm : Fact, act : Fact, subject : Person, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb violation(norm : Fact, act : Fact, subject : Person, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb enforceability(norm : Fact, act : Fact, subject : Person, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb patrimonialAsset(owner : Person, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb patrimonialLiability(debtor : Person, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb transferred(act : Fact, subject : Person, object : Thing, recipient : Person, authority : Authority, effective : PropositionTime)
derived verb owns(owner : Person, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb capacity(holder : Person, conduct : Conduct, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb legislativeCapacity(holder : Person, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb legislativePower(holder : Person, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb law(norm : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb annulled(inferior : Fact, superior : Fact, authority : Authority, effective : PropositionTime)

rule filedObligation(id, subject, conduct, object, beneficiary, authority, effective, t) =
    obligationFiled(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t

rule obligationCompliance(norm, act, subject, conduct, object, beneficiary, authority, effective, actAuthority, actEffective, t) =
    obligation(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    & Act(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= compliance(norm, act, subject, object, beneficiary, authority, effective) @ t

rule rightCompliance(norm, act, subject, conduct, object, beneficiary, authority, effective, actAuthority, actEffective, t) =
    right(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    & Act(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= compliance(norm, act, subject, object, beneficiary, authority, effective) @ t

rule obligationViolation(norm, act, subject, conduct, object, beneficiary, authority, effective, actAuthority, actEffective, t) =
    obligation(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    & CounterAct(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= violation(norm, act, subject, object, beneficiary, authority, effective) @ t

rule rightEnforceability(norm, act, subject, conduct, object, beneficiary, authority, effective, actAuthority, actEffective, t) =
    right(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    & CounterAct(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= enforceability(norm, act, subject, object, beneficiary, authority, effective) @ t

rule prohibitedAct(norm, act, subject, conduct, object, beneficiary, authority, effective, actAuthority, actEffective, t) =
    prohibition(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    & Act(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= violation(norm, act, subject, object, beneficiary, authority, effective) @ t

rule prohibitedOmission(norm, act, subject, conduct, object, beneficiary, authority, effective, actAuthority, actEffective, t) =
    omissionProhibition(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    & CounterAct(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= violation(norm, act, subject, object, beneficiary, authority, effective) @ t

// Política patrimonial de este expediente: la prestación cumplida acredita
// el activo de la cosa a su beneficiario; la violación registra el pasivo del
// obligado. Son hechos Derived, no mutaciones de un Store ni valoración monetaria.
rule complianceAsset(norm, act, subject, object, beneficiary, authority, effective, t) =
    compliance(norm, act, subject, object, beneficiary, authority, effective) @ t
    <= patrimonialAsset(beneficiary, object, norm, authority, effective) @ t

rule violationLiability(norm, act, subject, object, beneficiary, authority, effective, t) =
    violation(norm, act, subject, object, beneficiary, authority, effective) @ t
    <= patrimonialLiability(subject, object, norm, authority, effective) @ t

rule existingAsset(id, owner, object, authority, effective, t) =
    patrimonialHolding(id, owner, object, authority, effective) @ t
    <= patrimonialAsset(owner, object, id, authority, effective) @ t

rule patrimonyTitle(owner, object, cause, authority, effective, t) =
    patrimonialAsset(owner, object, cause, authority, effective) @ t
    <= owns(owner, object, cause, authority, effective) @ t

rule transferOccurrence(act, subject, object, recipient, authority, effective, t) =
    Act(act, subject, transfer, object, recipient, authority, effective) @ t
    <= transferred(act, subject, object, recipient, authority, effective) @ t

rule transferTitle(act, subject, object, recipient, authority, effective, t) =
    transferred(act, subject, object, recipient, authority, effective) @ t
    <= owns(recipient, object, act, authority, effective) @ t

rule ownershipCapacity(owner, object, cause, authority, effective, t) =
    owns(owner, object, cause, authority, effective) @ t
    <= capacity(owner, use, object, cause, authority, effective) @ t

// El poder legislativo exige su propio hecho de atribución: la titularidad
// de una cosa no confiere por sí sola poder para legislar.
// Su capacidad tiene un predicado distinto de la capacidad patrimonial de uso;
// así el grafo de dependencias del expediente es también explícitamente acíclico.
rule grantedCapacity(id, holder, object, authority, effective, t) =
    powerGranted(id, holder, enact, object, authority, effective) @ t
    <= legislativeCapacity(holder, object, id, authority, effective) @ t

rule capacityPower(owner, object, cause, authority, effective, t) =
    legislativeCapacity(owner, object, cause, authority, effective) @ t
    <= legislativePower(owner, cause, authority, effective) @ t

rule enactLaw(act, legislatorPerson, object, beneficiary, authority, effective, powerCause, powerAuthority, powerEffective, norm, subject, conduct, t) =
    Act(act, legislatorPerson, enact, object, beneficiary, authority, effective) @ t
    & legislativePower(legislatorPerson, powerCause, powerAuthority, powerEffective) @ t
    & legislationText(act, norm, subject, conduct, object, beneficiary, authority, effective) @ t
    <= law(norm, subject, conduct, object, beneficiary, authority, effective) @ t

rule statutoryObligation(norm, subject, conduct, object, beneficiary, authority, effective, t) =
    law(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    <= obligation(norm, subject, conduct, object, beneficiary, authority, effective) @ t

// Conflicto sólo si coinciden sujeto, conducta, cosa y beneficiario.
// La superioridad debe ser estricta y la vigencia superior no anterior.
// Se consideran ambas orientaciones de permission/prohibition y
// obligation/prohibition. annulled añade una marca, jamás retracta el inferior.
rule permissionAnnulsProhibition(high, low, subject, conduct, object, beneficiary, highAuthority, lowAuthority, highDate, lowDate, t) =
    permission(high, subject, conduct, object, beneficiary, highAuthority, highDate) @ t
    & prohibition(low, subject, conduct, object, beneficiary, lowAuthority, lowDate) @ t
    & higherAuthority(highAuthority, lowAuthority) @ t
    & notEarlier(highDate, lowDate) @ t
    <= annulled(low, high, highAuthority, highDate) @ t

rule prohibitionAnnulsPermission(high, low, subject, conduct, object, beneficiary, highAuthority, lowAuthority, highDate, lowDate, t) =
    prohibition(high, subject, conduct, object, beneficiary, highAuthority, highDate) @ t
    & permission(low, subject, conduct, object, beneficiary, lowAuthority, lowDate) @ t
    & higherAuthority(highAuthority, lowAuthority) @ t
    & notEarlier(highDate, lowDate) @ t
    <= annulled(low, high, highAuthority, highDate) @ t

rule obligationAnnulsProhibition(high, low, subject, conduct, object, beneficiary, highAuthority, lowAuthority, highDate, lowDate, t) =
    obligation(high, subject, conduct, object, beneficiary, highAuthority, highDate) @ t
    & prohibition(low, subject, conduct, object, beneficiary, lowAuthority, lowDate) @ t
    & higherAuthority(highAuthority, lowAuthority) @ t
    & notEarlier(highDate, lowDate) @ t
    <= annulled(low, high, highAuthority, highDate) @ t

rule prohibitionAnnulsObligation(high, low, subject, conduct, object, beneficiary, highAuthority, lowAuthority, highDate, lowDate, t) =
    prohibition(high, subject, conduct, object, beneficiary, highAuthority, highDate) @ t
    & obligation(low, subject, conduct, object, beneficiary, lowAuthority, lowDate) @ t
    & higherAuthority(highAuthority, lowAuthority) @ t
    & notEarlier(highDate, lowDate) @ t
    <= annulled(low, high, highAuthority, highDate) @ t

// Escenarios independientes del mismo programa, sin hechos fabricados en Rust.
experiment fulfilledTransfer {
    seeds {
        obligationFiled(transferDuty, juan, transfer, car, pedro, private, oldDate) @ tau,
        right(transferRight, juan, transfer, car, pedro, private, oldDate) @ tau,
        Act(transferAct, juan, transfer, car, pedro, private, newDate) @ tau,
    }
    query residual owns(pedro, car, transferDuty, private, oldDate) @ tau
}

experiment counterOnly {
    seeds {
        obligationFiled(deliveryDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        right(deliveryRight, juan, deliver, car, pedro, private, oldDate) @ tau,
        omissionProhibition(noOmission, juan, deliver, car, pedro, judicial, oldDate) @ tau,
        CounterAct(deliveryAct, juan, deliver, car, pedro, private, newDate) @ tau,
    }
    query residual violation(deliveryDuty, deliveryAct, juan, car, pedro, private, oldDate) @ tau
}

experiment actAndCounter {
    seeds {
        obligationFiled(deliveryDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        Act(deliveryAct, juan, deliver, car, pedro, private, newDate) @ tau,
        CounterAct(deliveryAct, juan, deliver, car, pedro, private, newDate) @ tau,
        extinguished(deliveryDuty, judicial, newDate) @ tau,
    }
    query residual compliance(deliveryDuty, deliveryAct, juan, car, pedro, private, oldDate) @ tau
}

experiment prohibitedOccurrence {
    seeds {
        prohibition(noMolesting, juan, molest, car, pedro, administrative, oldDate) @ tau,
        Act(molestingAct, juan, molest, car, pedro, private, newDate) @ tau,
    }
    query residual violation(noMolesting, molestingAct, juan, car, pedro, administrative, oldDate) @ tau
}

experiment legislation {
    seeds {
        patrimonialHolding(holding, legislator, legislativeSeat, constitutional, oldDate) @ tau,
        powerGranted(grantedPower, legislator, enact, legislativeSeat, constitutional, oldDate) @ tau,
        Act(enactment, legislator, enact, car, pedro, legislative, newDate) @ tau,
        legislationText(enactment, statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau,
    }
    query residual obligation(statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau
}

experiment authorityOverrides {
    seeds {
        higherAuthority(constitutional, private) @ tau,
        higherAuthority(legislative, administrative) @ tau,
        higherAuthority(judicial, private) @ tau,
        notEarlier(newDate, oldDate) @ tau,
        notEarlier(oldDate, oldDate) @ tau,
        permission(carPermission, juan, transfer, car, pedro, constitutional, newDate) @ tau,
        prohibition(carProhibition, juan, transfer, car, pedro, private, oldDate) @ tau,
        obligationFiled(boatDuty, juan, deliver, boat, pedro, legislative, newDate) @ tau,
        prohibition(boatProhibition, juan, deliver, boat, pedro, administrative, oldDate) @ tau,
        prohibition(houseProhibition, juan, molest, house, pedro, judicial, newDate) @ tau,
        permission(housePermission, juan, molest, house, pedro, private, oldDate) @ tau,
        prohibition(seatProhibition, juan, transfer, legislativeSeat, pedro, constitutional, newDate) @ tau,
        obligationFiled(seatDuty, juan, transfer, legislativeSeat, pedro, private, oldDate) @ tau,
        permission(olderPermission, juan, transfer, book, pedro, judicial, oldDate) @ tau,
        prohibition(newerProhibition, juan, transfer, book, pedro, private, newDate) @ tau,
        permission(sameDatePermission, juan, transfer, bicycle, pedro, constitutional, oldDate) @ tau,
        prohibition(sameDateProhibition, juan, transfer, bicycle, pedro, private, oldDate) @ tau,
    }
    query residual annulled(carProhibition, carPermission, constitutional, newDate) @ tau
}

experiment unperformed {
    seeds {
        obligationFiled(deliveryDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
    }
    query residual violation(deliveryDuty, deliveryAct, juan, car, pedro, private, oldDate) @ tau
}

experiment enactmentWithoutPower {
    seeds {
        Act(enactment, legislator, enact, car, pedro, legislative, newDate) @ tau,
        legislationText(enactment, statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau,
    }
    query residual obligation(statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau
}
