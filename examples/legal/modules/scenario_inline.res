module SaleScenarioInline

// Oráculo inline equivalente, sin imports.

// Vocabulario y efectos reutilizables; sin datos ni experiments.
entity Thing
entity Conduct
entity Authority
entity Fact
entity PerformanceMode
entity Person {
    patrimony : Patrimony
}
entity Relation {
    debtor : Person
    thing : Thing?
    conduct : Conduct?
    creditor : Person
    mode : PerformanceMode
}
entity Patrimony {
    assets : Set Thing
    rights : Set Relation
    obligations : Set Relation
}

const Doing : PerformanceMode
const Abstaining : PerformanceMode
const transfer : Conduct
const private : Authority

seeded verb Act(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb obligation(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)
derived verb permission(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)

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

rule obligationCreatesRelation(id, subject, conduct, object, beneficiary, authority, effective, t) =
    obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= do(subject, conduct, object, beneficiary) @ after t


// Programa parametrizado reutilizable, activado sólo por su verbo asociado.
rule givingProgram(subject, object, recipient, t) =
    I <= give(subject, object, recipient) @ after t

entity ReciprocityPackage
const Plain : ReciprocityPackage
const Retention : ReciprocityPackage
const pay : Conduct

seeded verb sale(id : Fact, seller : Person, object : Thing, buyer : Person, price : Thing, duty : Fact, permission : Fact, authority : Authority, effective : PropositionTime)
seeded verb paymentFor(sale : Fact, act : Fact)
derived verb selectedReciprocity(sale : Fact, package : ReciprocityPackage)
seeded verb reciprocity(sale : Fact, package : ReciprocityPackage)
seeded verb sells(subject : Person, object : Thing, recipient : Person) = program givingProgram

// Mismo antecedente para las dos lambdas; paquetes diferentes como resultado.
rule normalSale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    <= selectedReciprocity(saleId, Plain) @ t

rule retentionSale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    <= selectedReciprocity(saleId, Retention) @ t

// Reglas obligatorias del programa: interpretan el paquete seleccionado.
rule reciprocalTransferPermission(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, package, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & selectedReciprocity(saleId, package) @ t
    <= permission(permit, seller, transfer, object, buyer, authority, effective) @ t

rule plainTransferObligation(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & selectedReciprocity(saleId, Plain) @ t
    <= obligation(duty, seller, transfer, object, buyer, authority, effective) @ t

rule paidRetentionObligation(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, payment, paymentAuthority, paymentEffective, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & selectedReciprocity(saleId, Retention) @ t
    & paymentFor(saleId, payment) @ t
    & Act(payment, buyer, pay, priceObject, seller, paymentAuthority, paymentEffective) @ t
    <= obligation(duty, seller, transfer, object, buyer, authority, effective) @ t


// Canal Seeded independiente del canal Derived seleccionado.
rule suppliedTransferPermission(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, package, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & reciprocity(saleId, package) @ t
    <= permission(permit, seller, transfer, object, buyer, authority, effective) @ t

rule suppliedPlainObligation(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & reciprocity(saleId, Plain) @ t
    <= obligation(duty, seller, transfer, object, buyer, authority, effective) @ t

rule suppliedPaidRetentionObligation(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, payment, paymentAuthority, paymentEffective, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & reciprocity(saleId, Retention) @ t
    & paymentFor(saleId, payment) @ t
    & Act(payment, buyer, pay, priceObject, seller, paymentAuthority, paymentEffective) @ t
    <= obligation(duty, seller, transfer, object, buyer, authority, effective) @ t


seeded verb retentionAgreed(sale : Fact)

// El bloqueo es explícito. La excepción además conserva su relación de
// prioridad; blocked y defeated son registros distintos del evaluador actual.
default NormalSale =
    supernormal normalSale
    blocked when
        retentionAgreed(saleId) @ t

default RetentionSale =
    exception retentionSale
    to NormalSale
    when
        retentionAgreed(saleId) @ t


const juan : Person
const pedro : Person
const car : Thing
const price : Thing
const oldDate : PropositionTime
const newDate : PropositionTime
const tau : PropositionTime
const saleCase : Fact
const saleTransferDuty : Fact
const saleTransferPermission : Fact
const pricePayment : Fact

experiment noRetention {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionUnpaid {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
    }
    // No se deriva la obligación; tampoco se afirma ~obligation.
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionPaid {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment normalBlocked {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
    }
    // Mismo expediente que retentionUnpaid; aquí se observa la lambda ganadora.
    query residual selectedReciprocity(saleCase, Retention) @ tau
}

experiment sellsReal {
    seeds { sells(juan, car, pedro) @ tau, }
    query residual give(juan, car, pedro) @ after tau
}
experiment sellsAbsent {
    seeds {}
    query residual give(juan, car, pedro) @ after tau
}
experiment suppliedPlain {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Plain) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}
experiment suppliedOpposition {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Retention) @ tau,
        retentionAgreed(saleCase) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        ~Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}
