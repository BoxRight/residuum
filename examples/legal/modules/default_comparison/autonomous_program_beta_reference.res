module AutonomousProgramBetaReference

import "./data.res"

// Oráculo ground: sólo sintaxis existente. No declara Program ni cambia Sales.
rule groundEntry() =
    sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau
    <= selectedReciprocity(saleCase, Plain) @ tau

rule groundPermission() =
    sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau
    & selectedReciprocity(saleCase, Plain) @ tau
    <= permission(saleTransferPermission, juan, transfer, car, pedro, private, oldDate) @ tau

rule groundObligation() =
    sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau
    & selectedReciprocity(saleCase, Plain) @ tau
    <= obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau

// Compara además el alcance real de la aplicación de Rule que ya existe.
rule entryViaExistingBeta = normalSale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate, tau)
rule permissionViaExistingBeta = reciprocalTransferPermission(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate, Plain, tau)
rule obligationViaExistingBeta = plainTransferObligation(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate, tau)
