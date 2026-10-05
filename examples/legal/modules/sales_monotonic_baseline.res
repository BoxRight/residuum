module SalesMonotonicBaseline

import "./legal_monotonic.res"
import "./associated_program.res"

// Baseline de regresión de la compraventa monotónica, previo a defaults.
// reciprocity permanece Seeded; no se infiere desde selectedReciprocity.

// La asociación pertenece a Sales; el consumidor importa el verbo y no
// necesita repetir el cuerpo de la lambda que constituye el efecto give.
seeded verb sells(subject : Person, object : Thing, recipient : Person) = program givingProgram

entity ReciprocityPackage

const Plain : ReciprocityPackage
const Retention : ReciprocityPackage
const pay : Conduct
seeded verb sale(id : Fact, seller : Person, object : Thing, buyer : Person, price : Thing, duty : Fact, permission : Fact, authority : Authority, effective : PropositionTime)
seeded verb reciprocity(sale : Fact, package : ReciprocityPackage)
// El documento de pago debe corresponder a esta compraventa. Su ocurrencia
// se acredita separadamente mediante Act, con comprador, precio y vendedor.
seeded verb paymentFor(sale : Fact, act : Fact)

// Relaciones de comparación aportadas como datos del caso. Sólo se usan las
// comparaciones explícitamente acreditadas; no hay ranking especial en Rust.
rule reciprocalTransferPermission(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, package, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & reciprocity(saleId, package) @ t
    <= permission(permit, seller, transfer, object, buyer, authority, effective) @ t

rule plainTransferObligation(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & reciprocity(saleId, Plain) @ t
    <= obligation(duty, seller, transfer, object, buyer, authority, effective) @ t

rule paidRetentionObligation(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, payment, paymentAuthority, paymentEffective, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & reciprocity(saleId, Retention) @ t
    & paymentFor(saleId, payment) @ t
    & Act(payment, buyer, pay, priceObject, seller, paymentAuthority, paymentEffective) @ t
    <= obligation(duty, seller, transfer, object, buyer, authority, effective) @ t

// Estas reglas constituyen normas, que pueden derivar do mediante el puente.
// No afirman Act(transfer) ni ejecutan State. Una transmisión ipso iure
// requerirá su regla explícita; give sólo se deriva aquí de transferred.


// Se conserva el símbolo Derived del experimento anterior, sin puente ni regla
// que lo derive en este programa monotónico.
derived verb selectedReciprocity(sale : Fact, package : ReciprocityPackage)
