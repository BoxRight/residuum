# Explicaciones sobre expedientes tipados y una teoría jurídica fija

Experimento en `experiment-legal-order-geometry`, posterior al examen del contrato
de las [preguntas minimax](legal-minimax-questions-juridical-assessment.md).
Se conserva LegalGeneral íntegra. Las hipótesis no reemplazan productores de
compliance ni añaden reglas al cierre. Las anteriores valoraciones binarias
no se reutilizan para decidir estos casos.

**Resultado:** la misma presentación de un informe produce consecuencias distintas
cuando consta un deber o cuando sólo se aporta un permiso. Las evaluaciones del
harness distinguen compatibilidad, incompatibilidad e indeterminación. Un tercer
caso con una pretensión documentada discrimina dos explicaciones del mecanismo
sin cambiar la teoría. La finalidad jurídica de la norma permanece indeterminada.

## 1. Contrato nuevo del experimento

Para cada expediente S se calcula primero el cierre completo de T=LegalGeneral:

\[
X_T(S)=(S,C_T(S),D_T(S)).
\]

D conserva todas las justificaciones, con reglas, módulos, testigos ordenados,
sustituciones, tipos y tiempos. Las evaluaciones explicativas consumen esa
estructura ya calculada y una objeción registrada externamente. Su resultado
incluye estado, motivo, testigos, referencias a justificaciones y cuestiones
pendientes. No se convierte en un evento de Known.

Los estados del harness significan:

| Estado | Contrato |
|---|---|
| Compatible | El caso observado o su derivación encaja con la afirmación concreta evaluada. No prueba su validez general ni su adecuación jurídica. |
| Incompatible | Hay un contraejemplo en la computación completa o una justificación real que contradice la afirmación sobre ese mecanismo. |
| Indeterminate | No hay una prueba del objetivo que permita evaluar la ruta propuesta, falta evidencia positiva para probar una premisa, o el significado de la explicación no está expresado por la ontología. |

La conservación de candidatos elimina sólo incompatibilidades. Una evaluación
indeterminada no se convierte en rechazo ni en aprobación. El campo separado
`legal_adequacy` queda `not_established_by_reference_derivation` en todas las filas.
No se implementa una nueva lógica trivalente en Residuum.

La no derivabilidad de un objetivo en un cierre completo puede refutar una
afirmación sobre **lo que ese evaluador deriva**. No permite producir ~compliance
ni demostrar que una conducta no ocurrió. En particular, la falta de Act conserva
su carácter de ausencia de evidencia.

## 2. Los dos expedientes concretos

La [fixture Formal](../examples/experiments/legal_geometry/typed_explanatory_cases.res)
importa directamente la ontología general. Sólo añade constantes y experiments,
sin nuevas entidades, verbos o reglas jurídicas.

Los expedientes alternativos son sintéticos, no expedientes de 12hectareas:

- Sujeto de la prestación: `ana : Person`.
- Beneficiario: `instituto : Person`.
- Objeto: `informeAnual : Thing`.
- Conducta: `presentarInforme : Conduct`.
- Documento normativo: `instrumentoInforme : Fact`.
- Acto: `presentacionInforme : Fact`.
- Vigencia: `vigenciaInforme : PropositionTime`.
- Índice del expediente: `observacionInforme : PropositionTime`.

El tipo Person no distingue aquí personas naturales de instituciones. La lectura
institucional de la constante es parte del guion, no una ampliación de los tipos.
Los tiempos son símbolos explícitos; no son fechas de calendario inventadas.

En ambos expedientes se aporta exactamente:

```text
Act(presentacionInforme, ana, presentarInforme, informeAnual,
    instituto, private, vigenciaInforme) @ observacionInforme
```

El primer expediente añade obligationFiled y el segundo permissionFiled, con
instrumentoInforme, la misma prestación y administrative como autoridad. No se
introduce un permiso para otra conducta, una omisión o un destinatario distinto.
El identificador del documento es común en estos escenarios alternativos; no
se trata de dos documentos simultáneos cuya identidad jurídica haya sido validada.

El objetivo común es:

```text
compliance(instrumentoInforme, presentacionInforme, ana, informeAnual,
           instituto, administrative, vigenciaInforme) @ observacionInforme
```

**EXPERIMENTAL OBSERVATION.** Los cierres completos son:

| Expediente | Seeds | Consecuencias nuevas | Eventos / justificaciones |
|---|---|---|---:|
| owedAndPerformed | obligationFiled y Act | obligation, compliance, do | 5 / 3 |
| permittedAndPerformed | permissionFiled y el mismo Act | permission | 3 / 1 |

obligation y compliance conservan observacionInforme. do queda en
after(observacionInforme), con su interpretación de transición de estado asociada.
No se ejecuta ese efecto. La realización histórica ya procede del Act sembrado.

El permiso aporta una posición diferente de la obligación documentada. Su ausencia
como productor de compliance no significa que la actuación esté prohibida, que
exista ~Act ni que se haya demostrado ausencia de un deber en el mundo real.
Todas las justificaciones proceden de LegalGeneral, sin productores hipotéticos.

Los [casos elaborados](legal-geometry/typed-explanations/cases.tsv),
[Known completos](legal-geometry/typed-explanations/known.tsv) y
[justificaciones](legal-geometry/typed-explanations/provenance.tsv) conservan los
argumentos, metadata operacional y orígenes. No se proyectan a O/R/P/A.

## 3. Objeción externa y fundamento

La [objeción](../examples/experiments/legal_geometry/typed_explanatory_objections.tsv)
pregunta por qué el deber más la presentación documentada y el permiso más esa
misma presentación reciben consecuencias diferentes.

Su fundamento es un supuesto experimental independiente: mantener fija la
prestación hace relevante deliberar sobre la diferencia entre posiciones normativas.
No exige tratarlas igual, no declara injusta la norma ni obtiene una etiqueta de
las mismas reglas examinadas. Queda `open_external_question`, con fuente
`stipulation:typed-explanations/objection`.

El motor responde qué deriva cada expediente. La objeción pregunta por qué esa
distinción podría merecer una justificación jurídica. Esas dos respuestas no
son equivalentes.

## 4. Explicaciones construidas y límite interpretativo

El [catálogo del harness](../examples/experiments/legal_geometry/typed_explanatory_hypotheses.tsv)
declara afirmaciones y fuentes externas. Las referencias de reglas se resuelven
y verifican contra las declaraciones originales y sus módulos.

| Candidata | Afirmación evaluada |
|---|---|
| HRelational | Las pruebas observadas de compliance se apoyan en una posición normativa y el Act correspondiente, mediante obligationCompliance o rightCompliance. |
| HFiledDuty | Las pruebas observadas de compliance usan la rama obligationCompliance. Se investiga esa posible explicación documental. |
| HActSuffices | El Act correspondiente basta para que el evaluador de referencia derive compliance en estos expedientes, cualquiera que sea el registro normativo. |
| HRecipientPurpose | La razón de la diferencia es proteger el interés del destinatario, no la categoría documental. |

Las dos primeras son condiciones sobre referencias a reglas y testigos tipados
de las derivaciones. La tercera es una afirmación contrastable sobre el cierre,
no una nueva regla Act→compliance. La cuarta se conserva como objeción explicativa
abierta: la ontología no tiene una relación que identifique finalidad, interés
protegido o razón independiente de una norma. No se inventa un predicado para
evaluarla ni se identifica automáticamente beneficiary con una finalidad normativa.

**MODEL RESULT**, referido exclusivamente a los casos observados:

| Expediente | HRelational | HFiledDuty | HActSuffices | HRecipientPurpose |
|---|---|---|---|---|
| Deber y presentación | Compatible | Compatible | Compatible | Indeterminate |
| Permiso y presentación | Indeterminate | Indeterminate | Incompatible | Indeterminate |

En el segundo expediente no existe una derivación de compliance cuya rama pueda
examinarse: las explicaciones de ruta no se rechazan por ello. HActSuffices sí
tiene un contraejemplo operacional: el Act correspondiente está presente y el
cierre completo no deriva el objetivo. La compatibilidad de esa candidata con
el primer expediente era sólo compatibilidad con una muestra; no probaba que
Act fuera causalmente suficiente.

Tras el par se conservan HRelational, HFiledDuty y HRecipientPurpose. Las
[evaluaciones estructuradas](legal-geometry/typed-explanations/assessments.tsv)
incluyen testigos, justificaciones, causas de indeterminación y fuentes.

## 5. Nuevo caso discriminante y control de falta de evidencia

Se propone un expediente con right del Instituto respecto de la prestación de
Ana y el mismo Act. No se aporta obligationFiled. El cierre original deriva
compliance mediante rightCompliance; no deriva obligation ni do en este expediente.
Esto no es una conclusión de inexistencia jurídica del deber correlativo.

| Candidata superviviente | Evaluación del caso propuesto |
|---|---|
| HRelational | Compatible: la ruta rightCompliance está contemplada. |
| HFiledDuty | Incompatible: una prueba observada usa la rama rightCompliance. |
| HRecipientPurpose | Indeterminate: el caso no identifica la finalidad de la norma. |

El caso discrimina hipótesis sobre el mecanismo de derivación, no decide por
sí solo la justificación normativa. Sus dos seeds son mínimos por inclusión
para este contraste: retirando cualquiera desaparece la prueba del objetivo y
las dos explicaciones de ruta quedan indeterminadas. No se afirma minimalidad
global sobre todos los hechos de la ontología.

La [tabla de etapas](legal-geometry/typed-explanations/candidate-stages.tsv)
registra por separado el par inicial y la evaluación computacional del caso
propuesto. Este último no recibe una valoración jurídica externa ficticia.

Como control independiente, un expediente con obligationFiled pero sin Act
deriva obligation y do. No deriva compliance, violation, Act ni ~Act.
Todas las candidatas quedan indeterminadas en ese control; ninguna se elimina
por no haberse observado la prestación. No hay puente automático de do a Act.

No se afirma que otro caso expresable con la misma ontología pueda resolver la
finalidad protegida. Harían falta interpretación y razones externas. Este límite
permanece registrado, sin diseñar una extensión del lenguaje.

## 6. Conservación y alcance de la certificación

El [verificador](../src/bin/residuum/cli/legal_geometry_typed_explanation_tests.rs)
carga separadamente la teoría original y exige igualdad literal del AST elaborado
y del origen de todas sus declaraciones importadas. La fixture conserva exactamente
27 reglas y no contiene Program, Default ni reglas explicativas nuevas.

Las evaluaciones no alteran el Closure, ni siquiera sus sellos ya calculados.
La recomputación con seeds en orden inverso conserva los eventos completos y
el multiconjunto de justificaciones; sólo se ignora derivedAt al comparar esas
ejecuciones independientes. Se auditan reglas, antecedentes y sustituciones
mediante el matcher existente.

La [invariancia](legal-geometry/typed-explanations/invariance.tsv) está comprobada
para los cuatro expedientes. No se creó T′ y no se reutilizan las revisiones del
experimento anterior. Tampoco se afirma que un nuevo universo de casos herede
automáticamente los resultados previos de antiintercambio, extremos o fronteras:
esta etapa certifica conservación de la teoría y sus derivaciones, no una nueva
geometría exhaustiva.

Se separan tres alcances:

- **Comprobado:** consecuencias originales, contraste de rutas y evaluaciones
  estructuradas en cuatro expedientes tipados.
- **Compatible con observaciones:** explicaciones parciales del mecanismo;
  la conservación de una candidata indeterminada no constituye apoyo positivo.
- **No establecido:** finalidad jurídica, justificación sustantiva de la norma
  o adecuación de una revisión. Requieren deliberación independiente.

Pasaron tres tests individuales con `test_safe.py`, sin suite completa. La
[verificación](legal-geometry/typed-explanations/verification.md) y el
[registro SHA-256](legal-geometry/typed-explanations/input-sha256.txt) fijan comandos,
recursos y entradas. Se mantuvieron motor, ontología, main y v1 intactos.
No hubo commit ni merge.
