# Contrato experimental de consistency para Program

Estado: EXPERIMENTAL OBSERVATION + IMPLEMENTED/TESTED sólo en el harness.
El motor conserva ConsistencyStatus::NotChecked. Ninguna observación de este
documento se utiliza como condición de selección, bloqueo o derrota.

## Objeto observado

Para un snapshot K de Known y un default declarado D con destino ProgramRef:

\[
H_D = Close_{R_{obligatorias}\cup compile(Program_D)}(K),
\qquad \Delta_D=Known(H_D)\setminus K.
\]

En estas fixtures K es el cierre obligatorio que precede a la selección de
defaults. Se conservan su provenance y sellos originales. El cierre hipotético
usa una copia de sus eventos, conserva el entorno y añade todos los miembros
compilados de un solo candidato; excluye los miembros de los demás defaults.
No copia las consecuencias de la selección real ni ejecuta StateTransform.

Los candidatos se inspeccionan independientemente incluso si no son aplicables,
están bloqueados o no fueron seleccionados. Esos casos se identifican como
contrafácticos; no se declara que el default deba activarse en el expediente.
Los resultados dependen del snapshot suministrado y del conjunto de reglas.
Las fixtures son acíclicas y los cierres terminan bajo los límites del runner;
no se afirma terminación general.

## Evidencia de oposición

Un par positivo/negativo tiene el mismo verbo, argumentos y PropositionTime,
con polaridades opuestas. Es la identidad lógica existente de Event: transition,
derivedAt y provenance no deciden si existe el par. Autoridad, fecha de vigencia
y beneficiario se conservan como argumentos; diferencias en esos argumentos
impiden formar un par exacto.

El contrato experimental ObservedConsistencyStatus contiene:

```text
candidate : DefaultRef
program : ProgramRef
prior : [EvidenceOpposition]
against_known : [EvidenceOpposition]
within_delta : [EvidenceOpposition]
delta : Set<Event>
hypothetical : Closure
```

Para cada par observado en H_D:

| Campo | Ubicación de sus dos eventos |
| --- | --- |
| prior | Ambos pertenecen a K |
| against_known | Uno pertenece a K y el otro a Δ_D |
| within_delta | Ambos pertenecen a Δ_D |

Son colecciones independientes: pueden estar vacías o ser no vacías al mismo
tiempo. No constituyen un enum de estados mutuamente excluyentes. Cero pares
nuevos significa únicamente que no se detectó oposición explícita nueva bajo
este criterio; no certifica consistencia jurídica, exclusividad de paquetes
ni compatibilidad con otros candidatos.

EvidenceOpposition conserva los dos eventos. Cada lado registra pertenencia a
K y todas sus justificaciones en K y en H_D por separado. No se reemplazan los
sellos de las justificaciones anteriores por los del cálculo hipotético.
Un evento ya conocido puede recibir una justificación adicional de Program_D
sin entrar en Δ_D: por eso delta de eventos no basta para registrar provenance.
La cadena de antecedentes permite identificar contribuciones indirectas del
candidato a través de reglas generales del entorno.

Este contrato está definido en tests, no como una nueva variante del enum
ConsistencyStatus del motor. La evaluación experimental no devuelve Consistent,
Inconsistent, Blocked, Defeated ni un ganador. Tampoco deriva inconsistent por
sí sola: ese verbo conserva su interpretación ordinaria del dominio.

## Casos y resultados

Se reutilizan los nueve escenarios de
[default_opposition_scenario.res](../examples/legal/modules/default_opposition_scenario.res)
y se añaden tres en
[program_consistency_scenario.res](../examples/legal/modules/program_consistency_scenario.res).
Cada uno se examina con NormalSale y RetentionSale: 24 observaciones de Programs
reales definidos en Sales. Los experiments importados no se ejecutan en el
consumidor: sus tres casos son independientes de los nueve anteriores.

| Escenario | Candidato | prior | against_known | within_delta |
| --- | --- | --- | --- | --- |
| clearSale | NormalSale | 0 | 0 | 0 |
| opposedNormal | NormalSale | 0 | 1 | 0 |
| opposedRetentionPaid | RetentionSale | 0 | 1 | 0 |
| alreadyOpposed | Ambos, individualmente | 2 | 0 | 0 |
| internalOpposition | NormalSale | 0 | 0 | 1 |
| emptyKnown | Ambos, individualmente | 0 | 0 | 0 |
| newNegativeAgainstKnown | Ambos, individualmente | 0 | 1 | 0 |
| mixedOpposition | NormalSale | 2 | 1 | 1 |
| mixedOpposition | RetentionSale | 2 | 0 | 0 |

Los dos pares previos son obligationFiled/~obligationFiled y la obligación
interpretada positiva/negativamente por reglas explícitas. No se colapsan los
dos verbos. Los casos con diferente autoridad, beneficiario o tiempo no producen
oposición exacta. La evidencia negativa no se obtiene por ausencia de hechos.

newNegativeAgainstKnown comprueba además que la obligación positiva conserva
su justificación anterior filedObligation y recibe otra justificación hipotética
plainTransferObligation de normalSaleProgram. El evento positivo no es nuevo;
el negativo sí. Su regla conditionalNegativeDuty pertenece al entorno, y sus
antecedentes enlazan con el miembro normalSale del programa candidato.

Se comprueba que ningún otro Program controlado aporta justificaciones a H_D,
que los antecedentes pertenecen al cierre y que las observaciones no modifican
Known, el módulo ni applicability/blocking/priority/selection. Las decisiones
reales se comparan antes y después; NotChecked permanece intacto.

Informe completo, incluyendo sustituciones, tiempos, origen Program/Rule/Module
y antecedentes: target/program-consistency-observations.txt.

Pasaron cinco tests individuales mediante scripts/test_safe.py: dos del contrato
nuevo y tres regresiones de oposición individual y conjunta. Compilación:
314,28 MiB de pico RSS; ejecuciones: 15,25–15,48 MiB. Registros:
target/test-diagnostics/20261005T191448089447Z-74908/. No se ejecutó la suite completa.

## Decisiones pendientes

Queda por decidir qué evidencia tiene significado de inadmisibilidad para el
dominio, si alguna; cómo se combina con priority; y si se evalúa una lambda
completa o instancias particulares. Una observación individual tampoco certifica
la combinación de varios candidatos: la regresión conjunta conserva el caso
en que dos defaults separados no introducen oposición, pero juntos sí.

No se introduce competes, !, exclusión automática por P/~P, nuevas prioridades
ni cambios de Program, selector, close, matcher o runtime.
