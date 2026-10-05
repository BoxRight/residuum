module SaleProgramSpecialization

import "./data.res"

// Aplicación parcial existente: no declara un cuerpo compuesto ni un programa.
// El entorno compartido sigue siendo las reglas importadas de Sales/Legal.
rule normalForJuan = normalSale(saleCase, juan)
rule retentionForJuan = retentionSale(saleCase, juan)

// Especializar la entrada no especializa automáticamente estas relaciones.
rule permissionForJuan = reciprocalTransferPermission(saleCase, juan)
rule obligationForJuan = plainTransferObligation(saleCase, juan)

const secondDuty : Fact
const secondPermission : Fact

// El lenguaje no impone una restricción de unicidad sobre saleId.
// No postulamos que sea un expediente válido: prueba el ámbito de sustitución.
experiment specializationScope {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        sale(saleCase, pedro, car, juan, price, secondDuty, secondPermission, private, oldDate) @ tau,
    }
    query residual obligation(secondDuty, pedro, transfer, car, juan, private, oldDate) @ tau
}
