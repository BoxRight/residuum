module ProgramEnvironmentScenario
import "./sale_defaults.res"
import "./default_comparison/data.res"
const generalDuty : Fact

experiment generalObligation {
    seeds {
        obligationFiled(generalDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
    }
    query residual do(juan, deliver, car, pedro) @ after tau
}
experiment generalObligationWithReservation {
    seeds {
        obligationFiled(generalDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
    }
    query residual do(juan, deliver, car, pedro) @ after tau
}
experiment saleAndGeneralObligation {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        obligationFiled(generalDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
    }
    query residual do(juan, transfer, car, pedro) @ after tau
}
