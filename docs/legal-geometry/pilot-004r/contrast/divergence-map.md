# h1/h2 divergence map (counterfactual pool; no valuations)

h1 = c1: rule obligationYieldsRight(id, subject, conduct, object, beneficiary, authority, effective, t) =
    obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= right(id, subject, conduct, object, beneficiary, authority, effective) @ t
h2 = c2: rule obligationYieldsRightCreditor(id, subject, conduct, object, beneficiary, authority, effective, t) =
    obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= right(id, beneficiary, conduct, object, subject, authority, effective) @ t

## right, obligated party as subject — `right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB`
head `right` is scored by the current loss

minimal supports O:
  - {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
minimal supports O+h1:
  - {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
  - {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
minimal supports O+h2:
  - {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
compare_partition region pool and signatures (h1, h2):
  [0] {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} h1=true h2=false  <- discriminant
  [1] {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} h1=true h2=true
covers (general, specific): [("c1", "c2")]
derivations of the goal on each minimal configuration:
  configuration {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
    O: not derived
    O+h1: {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by filedObligation [LegalGeneral] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h1: {obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by obligationYieldsRight [ContrastH1] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: not derived
  configuration {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
    O: given in the configuration (seeded leaf)
    O+h1: given in the configuration (seeded leaf)
    O+h2: given in the configuration (seeded leaf)

## right, beneficiary as subject — `right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB`
head `right` is scored by the current loss

minimal supports O:
  - {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
minimal supports O+h1:
  - {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
minimal supports O+h2:
  - {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
  - {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
compare_partition region pool and signatures (h1, h2):
  [0] {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} h1=false h2=true  <- discriminant
  [1] {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} h1=true h2=true
covers (general, specific): [("c2", "c1")]
derivations of the goal on each minimal configuration:
  configuration {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
    O: not derived
    O+h1: not derived
    O+h2: {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by filedObligation [LegalGeneral] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: {obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB by obligationYieldsRightCreditor [ContrastH2] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
  configuration {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
    O: given in the configuration (seeded leaf)
    O+h1: given in the configuration (seeded leaf)
    O+h2: given in the configuration (seeded leaf)

## compliance by the obligated party — `compliance(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB`
head `compliance` is NOT scored by the current loss

minimal supports O:
  - {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
  - {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
minimal supports O+h1:
  - {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
  - {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
minimal supports O+h2:
  - {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
  - {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
compare_partition region pool and signatures (h1, h2):
  [0] {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} h1=true h2=true
  [1] {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} h1=true h2=true
covers (general, specific): [("c1", "c2"), ("c2", "c1")]
derivations of the goal on each minimal configuration:
  configuration {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
    O: {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by filedObligation [LegalGeneral] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O: {obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => compliance(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by obligationCompliance [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]
    O+h1: {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by filedObligation [LegalGeneral] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h1: {obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by obligationYieldsRight [ContrastH1] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h1: {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => compliance(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by rightCompliance [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]
    O+h1: {obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => compliance(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by obligationCompliance [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by filedObligation [LegalGeneral] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: {obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => compliance(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by obligationCompliance [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]
  configuration {Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
    O: {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => compliance(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by rightCompliance [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]
    O+h1: {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => compliance(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by rightCompliance [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => compliance(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by rightCompliance [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]

## compliance by the beneficiary — `compliance(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB`
head `compliance` is NOT scored by the current loss

minimal supports O:
  - {Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
minimal supports O+h1:
  - {Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
minimal supports O+h2:
  - {Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
  - {Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
compare_partition region pool and signatures (h1, h2):
  [0] {Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} h1=false h2=true  <- discriminant
  [1] {Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} h1=true h2=true
covers (general, specific): [("c2", "c1")]
derivations of the goal on each minimal configuration:
  configuration {Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
    O: not derived
    O+h1: not derived
    O+h2: {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by filedObligation [LegalGeneral] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: {obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB by obligationYieldsRightCreditor [ContrastH2] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} => compliance(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB by rightCompliance [LegalGeneral] substitution [act=actU, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=jant, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=usa, t=tB]
  configuration {Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
    O: {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} => compliance(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB by rightCompliance [LegalGeneral] substitution [act=actU, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=jant, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=usa, t=tB]
    O+h1: {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} => compliance(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB by rightCompliance [LegalGeneral] substitution [act=actU, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=jant, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=usa, t=tB]
    O+h2: {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} => compliance(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB by rightCompliance [LegalGeneral] substitution [act=actU, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=jant, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=usa, t=tB]

## enforceability against the obligated party — `enforceability(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB`
head `enforceability` is NOT scored by the current loss

minimal supports O:
  - {~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
minimal supports O+h1:
  - {~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
  - {~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
minimal supports O+h2:
  - {~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
compare_partition region pool and signatures (h1, h2):
  [0] {~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} h1=true h2=false  <- discriminant
  [1] {~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} h1=true h2=true
covers (general, specific): [("c1", "c2")]
derivations of the goal on each minimal configuration:
  configuration {~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
    O: not derived
    O+h1: {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by filedObligation [LegalGeneral] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h1: {obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by obligationYieldsRight [ContrastH1] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h1: {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, ~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => enforceability(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by rightEnforceability [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: not derived
  configuration {~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
    O: {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, ~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => enforceability(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by rightEnforceability [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]
    O+h1: {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, ~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => enforceability(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by rightEnforceability [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: {right(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB, ~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => enforceability(saB_j, actJ, jant, policyA, usa, private, d20210216) @ tB by rightEnforceability [LegalGeneral] substitution [act=actJ, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=jant, t=tB]

## enforceability against the beneficiary — `enforceability(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB`
head `enforceability` is NOT scored by the current loss

minimal supports O:
  - {~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
minimal supports O+h1:
  - {~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
minimal supports O+h2:
  - {~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
  - {~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
compare_partition region pool and signatures (h1, h2):
  [0] {~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} h1=false h2=true  <- discriminant
  [1] {~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} h1=true h2=true
covers (general, specific): [("c2", "c1")]
derivations of the goal on each minimal configuration:
  configuration {~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB}
    O: not derived
    O+h1: not derived
    O+h2: {obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB by filedObligation [LegalGeneral] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: {obligation(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB} => right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB by obligationYieldsRightCreditor [ContrastH2] substitution [authority=private, beneficiary=usa, conduct=adoptPolicy, effective=d20210216, id=saB_j, object=policyA, subject=jant, t=tB]
    O+h2: {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, ~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} => enforceability(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB by rightEnforceability [LegalGeneral] substitution [act=actU, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=jant, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=usa, t=tB]
  configuration {~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB}
    O: {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, ~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} => enforceability(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB by rightEnforceability [LegalGeneral] substitution [act=actU, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=jant, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=usa, t=tB]
    O+h1: {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, ~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} => enforceability(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB by rightEnforceability [LegalGeneral] substitution [act=actU, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=jant, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=usa, t=tB]
    O+h2: {right(saB_j, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB, ~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB} => enforceability(saB_j, actU, usa, policyA, jant, private, d20210216) @ tB by rightEnforceability [LegalGeneral] substitution [act=actU, actAuthority=private, actEffective=d20210216, authority=private, beneficiary=jant, conduct=adoptPolicy, effective=d20210216, norm=saB_j, object=policyA, subject=usa, t=tB]

## Brookside under the current loss (real dossier)
{h1}: loss 0 missed [] excess 0 valued 11 unvalued_derived 0 exposures ["h1:B7:true", "h1:B8:true", "h1:B9:true", "h1:B10:true", "h1:B11:true"]
{h2}: loss 5 missed ["B7", "B8", "B9", "B10", "B11"] excess 0 valued 11 unvalued_derived 5 exposures []
{h1, h2}: loss 0 missed [] excess 0 valued 11 unvalued_derived 5 exposures ["h1:B7:true", "h1:B8:true", "h1:B9:true", "h1:B10:true", "h1:B11:true"]
