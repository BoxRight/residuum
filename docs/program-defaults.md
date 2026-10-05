# Program y defaults

Estado: IMPLEMENTED + TESTED en fixtures seleccionadas. No es una demostración
general del cálculo de defaults; consistency permanece NotChecked.

## Unidades y entorno

LegalMonotonicCase proporciona las reglas generales, incluida
obligationCreatesRelation. Sales declara dos programas reales:

| Program | RuleUses |
| --- | --- |
| normalSaleProgram | normalSale, reciprocalTransferPermission con Plain, plainTransferObligation |
| retentionSaleProgram | retentionSale, reciprocalTransferPermission con Retention, paidRetentionObligation |

Los parámetros compartidos y bindings se declaran en Sales. Los parámetros
locales del pago permanecen locales a paidRetentionObligation. Rule continúa
representando una sola cláusula. Program conserva su módulo de definición.

obligationCreatesRelation no pertenece a ninguno de los programas: cualquier
obligación puede activarla, incluso una obligación ajena a compraventa. Es una
regla ordinaria del entorno ejecutable y participa en el mismo fixpoint que los
miembros seleccionados. No se clona ni se atribuye al paquete.

Los programas se llaman normalSaleProgram y retentionSaleProgram para conservar
los nombres existentes de los defaults NormalSale y RetentionSale: el namespace
actual rechaza colisiones entre declaraciones de distintas categorías.

## Resolución y selección

SaleDefaults importa Sales sin copiar sus reglas. La sintaxis previa se conserva:

```text
default NormalSale =
    supernormal normalSaleProgram
    blocked when
        retentionAgreed(saleId) @ t

default RetentionSale =
    exception retentionSaleProgram
    to NormalSale
    when
        retentionAgreed(saleId) @ t
```

El elaborador resuelve el destino a RuleRef o ProgramRef y utiliza su firma para
tipar condiciones y comprobar compatibilidad de excepciones. Las referencias a
Rules individuales siguen admitidas.

evaluate_defaults mantiene el orden applicability, blocking, priority y selección.
DefaultExtension conserva las referencias a los programas seleccionados y las
Rules resultantes de compile_program. La compilación ocurre después de determinar
los sobrevivientes, sustituyendo todos los bindings de cada RuleUse, incluidos
tiempos y capturas. La aplicación parcial de Program sigue siendo la operación
β existente en elaboración; la selección no introduce aplicación por instancia.

Las sustituciones de applicability son evidencia de selección de la lambda
completa; no se usan para fijar sus parámetros a una venta concreta. El alcance
de bloqueo se informa como ProgramLambda, RuleLambda o Lambda para destinos mixtos.
Es el mismo bloqueo global previo, no una nueva política.

El runner construye el contexto obligatorio excluyendo los miembros de todos los
programas controlados por defaults. Después de seleccionar, entrega conjuntamente
el entorno y todos los miembros elegidos al cierre forward. No se ejecuta un
cierre independiente por miembro, no se ejecuta StateTransform y no se reevalúa
la selección a partir de sus propias consecuencias.

reciprocity sigue siendo Seeded y selectedReciprocity Derived. No existe una regla
automática entre ambos. Importar una declaración no activa un default ni ejecuta
experiments ajenos. La asociación simultánea de un miembro controlado mediante
Verb.program se rechaza en este runner: aún no tiene una semántica acordada.

## Comparación y provenance

Los cuatro primeros escenarios de defaults_scenario.res se comparan con
case_sale_defaults_formal.res, que permanece sin cambios:

| Caso | Programa elegido | Obligación de transmitir |
| --- | --- | --- |
| Sin reserva | normalSaleProgram | Sí |
| Reserva sin pago | retentionSaleProgram | No |
| Reserva con pago estructuralmente coincidente | retentionSaleProgram | Sí |
| NormalSale bloqueado por reserva | retentionSaleProgram | No |

Una evidencia negativa de pago o un pago con destinatario incorrecto no satisface
la condición. No se interpreta la falta de obligación como ~obligation.

Cada miembro compilado conserva su identidad Program::índiceDeUse::Rule.
program_rule_origin y explain exponen programa, regla y módulos de origen sin
modificar DerivationRecord. La comparación con Rules inline restaura las
sustituciones desde la regla fuente y los mismos antecedentes: no elimina
justificaciones ni ignora sustituciones. Se exige correspondencia uno a uno,
incluidos tiempos proposicionales y transiciones. Sólo derivedAt puede diferir
como sello operacional de la ejecución.

program_environment_scenario.res comprueba que una obligación general sigue
produciendo do aun cuando NormalSale esté bloqueado, y que una obligación de
compraventa usa la misma regla general sin provenance duplicada.
program_priority_scenario.res comprueba que una excepción aplicable derrota al
default base aun sin blocker, conservando la diferencia entre blocked y defeated.

Los prototipos anteriores mantienen un adaptador sólo en tests para compararse
con su oráculo histórico de RuleRef. Los tests de integración nativa cargan
directamente las fixtures: el motor no tiene una tabla Nombre → Program.

## Aislamiento con evidencia suficiente para miembros no seleccionados

IMPLEMENTED + TESTED, no teorema general de aislamiento. La fixture
[program_isolation_scenario.res](../examples/legal/modules/program_isolation_scenario.res)
importa los mismos Sales y SaleDefaults, sin modificar sus declaraciones.
Los dos programas tienen miembros distintos y comparten la definición de
reciprocalTransferPermission con capturas diferentes. Ambos usan el entorno
general filedObligation → obligationCreatesRelation, fuera de sus RuleUses.

El escenario declara únicamente una sonda local: testPackageEvidence y
testExposePackage. Esta regla ordinaria aporta deliberadamente el hecho
selectedReciprocity del paquete NO seleccionado. No es una regla jurídica nueva
de la ontología, una activación de Program, ni un puente desde reciprocity.
Los hechos intermedios de ambos paquetes pueden coexistir; la prueba identifica
qué reglas se activan mediante provenance, no mediante la ausencia del hecho.

| Programa seleccionado | Evidencia adicional de prueba | Obligación de transmitir | Justificaciones del otro programa |
| --- | --- | --- | --- |
| normalSaleProgram | Retention y pago válido | Sí, por plainTransferObligation | Ninguna |
| retentionSaleProgram | Plain, sin pago | No | Ninguna |
| retentionSaleProgram | Plain y pago válido | Sí, por paidRetentionObligation | Ninguna |

Cada caso tiene su control sin sonda: seis experiments en total. El verificador
confirma mediante matching que los antecedentes de la regla no seleccionada
sí están satisfechos. Si se filtrara al cierre, produciría una obligación ausente
en el caso sin pago, o una justificación adicional en los otros casos.
También comprueba una sola justificación para el permiso compartido y para
cada do producido por la regla general, conservando Program + Rule + Module.

La única diferencia con el control es el seed de la sonda, su hecho intermedio y
la justificación testExposePackage. Se compara el resto de eventos, tiempos y
provenance uno a uno, comprobando las sustituciones almacenadas y restaurando
los nombres fuente tras β. Sólo derivedAt puede variar. La proyección se hace
sobre una copia en el verificador: Known conserva toda la evidencia.

Pasaron dos tests individuales del target CLI mediante test_safe.py. Compilación:
269,82 MiB de pico RSS; ejecuciones: 15,43 y 15,25 MiB. Registros:
target/test-diagnostics/20261005T184548604937Z-51181/.
No se modificaron el motor, el selector, consistency ni competes.
Esta frontera es la del cierre orquestado por defaults: declarar un Program
por sí solo no convierte las reglas ordinarias de un módulo en privadas.

## Verificación y límites

Pasaron trece tests individuales del target CLI: cinco de integración nativa,
cinco regresiones de selección/cierre/extracción modular y tres comprobaciones
de β, elaboración inválida e imports pasivos. La regresión de extracción compara
los dieciséis escenarios monotónicos con el baseline.

Toda compilación y ejecución usa scripts/test_safe.py, con 1 GiB, sin swap,
un trabajo y un hilo, más sus límites preventivos. Se verifican tests individuales
del target CLI, no la suite completa ni el target de tests de biblioteca.

Una primera compilación fue detenida preventivamente por memoria; no hubo OOM
ni ejecución de tests. Se encapsularon tipos de combinadores Chumsky con boxed()
en fronteras de declaraciones ya completas. Esto reduce el tamaño de tipos sin
cambiar tokens, gramática, SurfaceAST ni semántica. La compilación posterior
terminó con pico RSS de 600,75 MiB; las ejecuciones seleccionadas usan cerca de
15–16 MiB. Los registros quedan en target/test-diagnostics/.

No se añade competes, consistency, !, nuevas prioridades, reglas jurídicas,
semántica de estado ni consultas residuales sobre Program. La selección de
defaults sigue siendo la política provisional de lambdas completas.
