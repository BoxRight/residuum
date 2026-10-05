module Sales

import "./legal_monotonic.res"
import "./associated_program.res"

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

// Miembros reutilizables de los Programs: interpretan el paquete seleccionado.
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

// Paquetes completos. La regla general obligationCreatesRelation permanece
// en LegalMonotonicCase, fuera de los miembros de estos Programs.
program normalSaleProgram(
    saleId : Fact, seller : Person, object : Thing, buyer : Person,
    priceObject : Thing, duty : Fact, permit : Fact, authority : Authority,
    effective : PropositionTime, t : PropositionTime
) {
    use normalSale {
        saleId = saleId, seller = seller, object = object, buyer = buyer,
        priceObject = priceObject, duty = duty, permit = permit,
        authority = authority, effective = effective, t = t,
    }
    use reciprocalTransferPermission {
        saleId = saleId, seller = seller, object = object, buyer = buyer,
        priceObject = priceObject, duty = duty, permit = permit,
        authority = authority, effective = effective, package = Plain, t = t,
    }
    use plainTransferObligation {
        saleId = saleId, seller = seller, object = object, buyer = buyer,
        priceObject = priceObject, duty = duty, permit = permit,
        authority = authority, effective = effective, t = t,
    }
}

program retentionSaleProgram(
    saleId : Fact, seller : Person, object : Thing, buyer : Person,
    priceObject : Thing, duty : Fact, permit : Fact, authority : Authority,
    effective : PropositionTime, t : PropositionTime
) {
    use retentionSale {
        saleId = saleId, seller = seller, object = object, buyer = buyer,
        priceObject = priceObject, duty = duty, permit = permit,
        authority = authority, effective = effective, t = t,
    }
    use reciprocalTransferPermission {
        saleId = saleId, seller = seller, object = object, buyer = buyer,
        priceObject = priceObject, duty = duty, permit = permit,
        authority = authority, effective = effective, package = Retention, t = t,
    }
    use paidRetentionObligation {
        saleId = saleId, seller = seller, object = object, buyer = buyer,
        priceObject = priceObject, duty = duty, permit = permit,
        authority = authority, effective = effective, t = t,
    }
}
