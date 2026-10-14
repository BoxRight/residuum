# Riggs: argumentos sobre una teoría sucesoria de referencia fija

Experimento en `experiment-legal-order-geometry`. Su resultado es una atribución
literal certificada, dos líneas argumentativas documentadas y una pregunta
contrafactual que distingue lecturas de la objeción. La teoría se mantiene fija
durante todas las evaluaciones del harness.

## Fuentes y alcance histórico

Se utiliza la [sentencia original de la Corte de Apelaciones de Nueva York](https://www.nycourts.gov/reporter/archives/riggs_palmer.htm),
115 NY 506 (1889). Las referencias de página se registran junto a cada argumento.
Los hechos históricos se abstraen en `historicalRiggs`; los cinco expedientes
restantes son estipulaciones experimentales identificadas como tales.

La [tabla de argumentos](../examples/experiments/legal_geometry/riggs_arguments.tsv)
conserva la lectura causal de la objeción de Earl, una lectura experimental más
estrecha que exige finalidad adquisitiva y la ruta formal invocada por Gray.
La [tabla de objeciones](../examples/experiments/legal_geometry/riggs_objections.tsv)
conserva las preguntas y sus fundamentos externos. La variante estrecha queda
identificada como hipótesis de lectura, no como posición adicional atribuida al tribunal.

## Teoría de referencia

La [teoría sucesoria](../examples/experiments/legal_geometry/riggs_succession_reference.res)
importa LegalGeneral íntegra y agrega seis verbos Seeded, tres Derived y cuatro
reglas. Se usan exclusivamente tipos y construcciones existentes. La comparación
del AST elaborado confirma las 27 reglas originales con sus declaraciones y
módulos de origen intactos: la teoría experimental tiene 31 reglas en total.

El núcleo literal es:

```text
operativeDisposition(will, testator, beneficiary, object, authority, effective) @ t
& deathRecorded(death, testator, deathAuthority, deathEffective) @ t
& survivorRecorded(beneficiary, testator) @ t
<= testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
```

`operativeDisposition` es un supuesto externo positivo de disposición admitida
y operativa. Agrupa las cuestiones de validez y revocación para delimitar la
referencia. La regla es una formalización experimental de las condiciones
elegidas, no una transcripción completa de legislación sucesoria.

`residuaryInterest : Thing` representa el interés discutido bajo esos supuestos.
La distribución completa, las cargas y las sustituciones testamentarias quedan
fuera del dominio. La atribución patrimonial calculada tiene esa misma delimitación.

La segunda regla conecta entitlement con `patrimonialAsset`. Las reglas originales
`patrimonyTitle` y `ownershipCapacity` producen `owns` y `capacity`. Esto permite
observar consecuencias jurídicas del modelo y conservar su cadena de pruebas.
Ningún Effect se ejecuta en este experimento.

Las otras dos reglas producen relaciones descriptivas:

- `acquisitionLinkedToWrong`: la persona beneficiaria, el testador, el acto y la
  muerte coinciden estructuralmente en la atribución y en las evidencias causales.
- `purposefulAcquisition`: además consta finalidad adquisitiva para ese acto,
  disposición y beneficiario.

Estas relaciones exponen las premisas que el harness puede examinar. No incorporan
una consecuencia de exclusión sucesoria. T permanece idéntica en cada expediente.

En este dominio, `intentionalKilling` registra el homicidio intencional ilícito
del guion. La ilicitud se estipula en la interpretación de ese dato; no se deduce
de que una muerte sea intencional. Los supuestos de justificación de una muerte
quedan pendientes para una eventual ampliación explícita del expediente.

`will1880`, `death1882` y `dossierTime` son constantes PropositionTime distintas.
El índice del expediente se conserva; la fecha del instrumento y la de la muerte
ocupan sus propios argumentos. El experimento no calcula orden de calendario.
`private` y `judicial` son datos de las evidencias; no resuelven prioridad entre argumentos.

## Expedientes y cierres completos

La [fixture](../examples/experiments/legal_geometry/riggs_cases.res) contiene las
constantes, los seis expedientes y sus objetivos. Usa `query residual`, la sintaxis
actual disponible para declarar un objetivo. Este harness lee ese objetivo y
certifica derivabilidad mediante `close_program`; no evalúa la búsqueda residual.

Los [orígenes de las premisas](../examples/experiments/legal_geometry/riggs_case_sources.tsv)
se registran separadamente de las justificaciones producidas por el motor.
En los contrafactuales, incluso los datos con autoridad `judicial` son supuestos
tipados del expediente imaginado, no afirmaciones sobre otra decisión histórica.

| Expediente | Entitlement literal | Vínculo causal adquisitivo | Finalidad adquisitiva |
|---|---|---|---|
| historicalRiggs | Derivado | Derivado | Derivada |
| noBeneficiaryKilling | Derivado | Sin prueba; negación explícita del acto identificado | Sin prueba |
| killingWithoutAcquisitionPurpose | Derivado | Derivado | Negación explícita para el acto vinculado |
| killingByThirdParty | Derivado | Sin prueba para el beneficiario; otro actor | Sin prueba |
| causalLinkUnestablished | Derivado | Evidencia pendiente | Evidencia pendiente |
| survivalUnestablished | Evidencia pendiente | Evidencia pendiente | Evidencia pendiente |

Las tres consecuencias patrimoniales aparecen exactamente en los cinco primeros
expedientes. La atribución se mantiene incluso cuando las relaciones descriptivas
dan fundamento fáctico a una objeción. El modelo permite examinar esa tensión.

El test enumera los 64 subconjuntos de los seis seeds históricos. La frontera
mínima de la atribución en ese dominio contiene exactamente el soporte:

\[
\{operativeDisposition,deathRecorded,survivorRecorded\}.
\]

**RESULTADO FINITO:** la unicidad de esta frontera se certifica para estas
instancias tipadas y este dominio. No establece una frontera de todo el derecho sucesorio.

## Contrato de evaluación argumentativa

El harness carga las afirmaciones y sus fuentes desde TSV, resuelve sus
condiciones sobre el cierre completo y devuelve:

```text
Assessment {
    status,
    reason,
    witnesses,
    derivations,
    legal_adequacy = requires_external_deliberation
}
```

`Compatible` significa que las premisas representadas dan apoyo fáctico a la
aplicación de esa línea en ese expediente. `Incompatible` identifica evidencia
explícita que niega una premisa del argumento sobre el acto examinado.
`Indeterminate` registra premisas pendientes o evidencia opuesta.

La lectura causal examina `acquisitionLinkedToWrong`; la lectura estrecha examina
`purposefulAcquisition`. La ruta formal examina la prueba del objetivo literal.
Cada resultado conserva testigos y referencias a DerivationRecord. La tabla
completa de provenance permite seguir transitivamente esas referencias hasta los seeds.

El alcance de las negaciones es el único acto identificado en estos expedientes.
El harness no generaliza su negación a cualquier acto posible. Cuando se aportan
finalidad positiva y negativa para ese acto, ambas permanecen en Known y la
evaluación de esa premisa queda indeterminada.

| Expediente | Earl: vínculo causal | Lectura con finalidad | Gray: ruta formal |
|---|---|---|---|
| historicalRiggs | Compatible | Compatible | Compatible |
| noBeneficiaryKilling | Incompatible | Incompatible | Compatible |
| killingWithoutAcquisitionPurpose | Compatible | Incompatible | Compatible |
| killingByThirdParty | Incompatible | Incompatible | Compatible |
| causalLinkUnestablished | Indeterminate | Indeterminate | Compatible |
| survivalUnestablished | Indeterminate | Indeterminate | Indeterminate |

Las evaluaciones son locales a cada expediente. Un control contrafactual donde
falta una premisa no elimina globalmente una línea argumentativa. El harness
conserva simultáneamente las dos líneas y sus fundamentos. Compatibilidad de
premisas y autoridad jurídica para restringir la atribución son cuestiones distintas.

Las preguntas externas se archivan con fuente y página. Su fundamento en lenguaje
natural requiere interpretación jurídica externa; los tests verifican las
condiciones tipadas declaradas y no interpretan automáticamente ese texto.

## Pregunta discriminante

En `killingWithoutAcquisitionPurpose`, la lectura causal tiene apoyo y la lectura
estrecha tiene una premisa explícitamente negada. El harness registra la pregunta:

> Si el beneficiario mata intencionalmente al testador, pero consta que no persigue
> obtener la herencia, ¿el beneficio causal basta para sostener la objeción y qué
> autoriza restringir la eficacia testamentaria?

Es una pregunta experimental propuesta para deliberación independiente. El primer
componente discrimina dos lecturas de la objeción; el segundo mantiene abierta la
respuesta relativa a las competencias judiciales. La comprobación demuestra que
el expediente separa las condiciones declaradas, no que la pregunta haya sido
valorada jurídicamente por un interlocutor.

## Certificación y límites

Tres tests individuales verifican:

1. Cierres completos, frontera literal y reconstrucción de cada DerivationRecord
   desde su regla y antecedentes mediante el instanciador existente.
2. Matriz de evaluaciones, fuentes, objeciones y pregunta discriminante.
3. Invariancia del módulo, Known y multiplicidad de pruebas durante la evaluación;
   igualdad al invertir los seeds, ignorando únicamente `derivedAt` entre ejecuciones;
   conservación de la oposición explícita sobre finalidad.

Todos usan `scripts/test_safe.py`, 1 GiB, sin swap y ejecución individual.
Las cifras y comandos están en [verification.md](legal-geometry/riggs/verification.md).
Los resultados completos están en [el directorio de artefactos](legal-geometry/riggs/).

**OBSERVACIÓN EXPERIMENTAL:** el motor permite conservar la atribución literal,
sus premisas y la tensión argumentativa en seis expedientes controlados.
**PENDIENTE DE DELIBERACIÓN:** suficiencia jurídica de cada razón y competencia
para una interpretación restrictiva. No se afirma identificabilidad de una
justificación verdadera ni una nueva propiedad geométrica general.

El experimento añade fixtures, un verificador y documentación en la rama de
investigación. Mantiene intactos el motor, las declaraciones originales,
`main` y `v1`. No se realizó merge.
