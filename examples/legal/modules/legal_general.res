module LegalGeneral

// Extracción íntegra de la parte reutilizable no específica de compraventa.
// Los cuerpos y nombres proceden de case_monotonic_formal.res sin cambios.

entity Person {
    patrimony : Patrimony
}
entity Thing
entity Conduct
entity PerformanceMode
entity Relation {
    debtor   : Person
    thing    : Thing?
    conduct  : Conduct?
    creditor : Person
    mode     : PerformanceMode
}
entity Patrimony {
    assets      : Set Thing
    rights      : Set Relation
    obligations : Set Relation
}
entity Authority
entity Fact
const Doing : PerformanceMode
const Abstaining : PerformanceMode

const transfer : Conduct
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
seeded verb Act(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb right(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb obligationFiled(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb prohibition(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb omissionProhibition(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb permissionFiled(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
seeded verb extinguished(id : Fact, authority : Authority, effective : PropositionTime)

// La misma compraventa puede estudiarse bajo dos paquetes aportados como
// datos positivos. reciprocity no selecciona defaults ni excluye otro paquete.
seeded verb higherAuthority(higher : Authority, lower : Authority)
seeded verb notEarlier(later : PropositionTime, earlier : PropositionTime)

seeded verb patrimonialHolding(id : Fact, owner : Person, object : Thing, authority : Authority, effective : PropositionTime)
seeded verb powerGranted(id : Fact, holder : Person, conduct : Conduct, object : Thing, authority : Authority, effective : PropositionTime)
seeded verb legislationText(act : Fact, norm : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)

// obligationFiled es el ingreso externo. obligation es la conclusión Derived
// común a ese ingreso y a las obligaciones que produzca law.
derived verb obligation(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb permission(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb compliance(norm : Fact, act : Fact, subject : Person, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb violation(norm : Fact, act : Fact, subject : Person, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb enforceability(norm : Fact, act : Fact, subject : Person, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb inconsistent(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb disputedPerformance(norm : Fact, act : Fact, subject : Person, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb patrimonialAsset(owner : Person, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb patrimonialLiability(debtor : Person, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb transferred(act : Fact, subject : Person, object : Thing, recipient : Person, authority : Authority, effective : PropositionTime)
derived verb owns(owner : Person, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb capacity(holder : Person, conduct : Conduct, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb legislativeCapacity(holder : Person, object : Thing, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb legislativePower(holder : Person, cause : Fact, authority : Authority, effective : PropositionTime)
derived verb law(norm : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb annulled(inferior : Fact, superior : Fact, authority : Authority, effective : PropositionTime)

// Effects conocidos y State se mantienen separados. do/notDo constituyen
// relaciones de hacer/abstenerse; no son evidencia de una conducta histórica.
// mode es un campo ordinario del record, no una polaridad de Act.
effect verb give(subject : Person, object : Thing, recipient : Person) =>
    subject.patrimony.assets -= object
    recipient.patrimony.assets += object

effect verb do(subject : Person, conduct : Conduct, indirectObject : Thing?, beneficiary : Person) =>
    subject.patrimony.obligations += Relation {
        debtor = subject,
        thing = indirectObject,
        conduct = conduct,
        creditor = beneficiary,
        mode = Doing,
    }
    beneficiary.patrimony.rights += Relation {
        debtor = subject,
        thing = indirectObject,
        conduct = conduct,
        creditor = beneficiary,
        mode = Doing,
    }

effect verb notDo(subject : Person, conduct : Conduct, indirectObject : Thing?, beneficiary : Person) =>
    subject.patrimony.obligations += Relation {
        debtor = subject,
        thing = indirectObject,
        conduct = conduct,
        creditor = beneficiary,
        mode = Abstaining,
    }
    beneficiary.patrimony.rights += Relation {
        debtor = subject,
        thing = indirectObject,
        conduct = conduct,
        creditor = beneficiary,
        mode = Abstaining,
    }

// Reglas puente expresas del expediente. La norma queda en Known; su efecto
// puede ejecutarse después. Autoridad y vigencia se conservan en provenance.
rule obligationCreatesRelation(id, subject, conduct, object, beneficiary, authority, effective, t) =
    obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= do(subject, conduct, object, beneficiary) @ after t

rule prohibitionCreatesAbstention(id, subject, conduct, object, beneficiary, authority, effective, t) =
    prohibition(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= notDo(subject, conduct, object, beneficiary) @ after t

rule historicalTransferEffect(act, subject, object, recipient, authority, effective, t) =
    transferred(act, subject, object, recipient, authority, effective) @ t
    <= give(subject, object, recipient) @ after t

rule filedObligation(id, subject, conduct, object, beneficiary, authority, effective, t) =
    obligationFiled(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t

// Los permisos externos y los producidos por reglas tienen la misma forma
// Derived. El documento sembrado permanece en Known, como obligationFiled.
rule filedPermission(id, subject, conduct, object, beneficiary, authority, effective, t) =
    permissionFiled(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= permission(id, subject, conduct, object, beneficiary, authority, effective) @ t

// Reciprocidad monotónica. El permiso es común a los dos paquetes; Plain
// añade el deber desde la compraventa, Retention lo condiciona al pago.
// t es el índice del expediente observado; effective conserva la vigencia
// de la norma y paymentEffective la del documento de pago, sin identificarlas.
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
    & ~Act(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= violation(norm, act, subject, object, beneficiary, authority, effective) @ t

rule rightEnforceability(norm, act, subject, conduct, object, beneficiary, authority, effective, actAuthority, actEffective, t) =
    right(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    & ~Act(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= enforceability(norm, act, subject, object, beneficiary, authority, effective) @ t

rule prohibitedAct(norm, act, subject, conduct, object, beneficiary, authority, effective, actAuthority, actEffective, t) =
    prohibition(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    & Act(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= violation(norm, act, subject, object, beneficiary, authority, effective) @ t

rule prohibitedOmission(norm, act, subject, conduct, object, beneficiary, authority, effective, actAuthority, actEffective, t) =
    omissionProhibition(norm, subject, conduct, object, beneficiary, authority, effective) @ t
    & ~Act(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
    <= violation(norm, act, subject, object, beneficiary, authority, effective) @ t

// Marcas ordinarias del dominio. No dan prioridad a violation, no bloquean
// reglas y no generan otros hechos salvo los expresamente declarados aquí.
// La oposición exacta exige mismos argumentos e índice, incluida la metadata.
rule conflictingActEvidence(act, subject, conduct, object, beneficiary, authority, effective, t) =
    Act(act, subject, conduct, object, beneficiary, authority, effective) @ t
    & ~Act(act, subject, conduct, object, beneficiary, authority, effective) @ t
    <= inconsistent(act, subject, conduct, object, beneficiary, authority, effective) @ t

rule performanceDispute(norm, act, subject, object, beneficiary, authority, effective, t) =
    compliance(norm, act, subject, object, beneficiary, authority, effective) @ t
    & violation(norm, act, subject, object, beneficiary, authority, effective) @ t
    <= disputedPerformance(norm, act, subject, object, beneficiary, authority, effective) @ t


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
