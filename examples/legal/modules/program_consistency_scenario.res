module ProgramConsistencyScenario

// Reutiliza las reglas de oposición del experimento y los Programs de Sales.
// Los nueve experiments del módulo importado NO se ejecutan aquí.
import "./default_opposition_scenario.res"

const priorDuty : Fact
const otherSale : Fact
const otherDuty : Fact
const otherPermission : Fact

experiment emptyKnown {
    seeds {}
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

// La conclusión negativa es nueva; su opuesta positiva ya pertenece a Known.
experiment newNegativeAgainstKnown {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        obligationFiled(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau,
        negativeDutyOnSelectedPackage(saleCase) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

// Los tres ámbitos coexisten para NormalSale. No son estados excluyentes:
// priorDuty ya tiene oposición; otherDuty se opone a Known; saleTransferDuty
// tiene sus dos polaridades nuevas dentro del cierre hipotético.
experiment mixedOpposition {
    seeds {
        obligationFiled(priorDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        ~obligationFiled(priorDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        sale(otherSale, juan, car, pedro, price, otherDuty, otherPermission, private, oldDate) @ tau,
        ~obligationFiled(otherDuty, juan, transfer, car, pedro, private, oldDate) @ tau,
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        negativeDutyOnSelectedPackage(saleCase) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}
