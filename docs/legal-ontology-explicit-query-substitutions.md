# Consultas con sustituciones explícitas: primera etapa experimental

La investigación posterior de [ámbitos por dependencias del AST](legal-ontology-query-dependency-scope.md)
distingue el cono estructural, las reglas habilitadas y los parámetros pendientes.
Identifica además una frontera entre certificación Horn y activación de programas.

La especialización existente permite fijar los parámetros que una conclusión
jurídica oculta y consultar las reglas resultantes. Los parámetros restantes
conservan sus tipos y ámbitos. Este experimento no añade una API de query con
sustituciones, ni resolución existencial, ni cambios al motor o a la ontología.

**Representación disponible.**

Se reutilizan `Program.parameters: Vec<Arg>`, `RuleUse.bindings`,
`ProgramBinding { parameter, value: Term }` y la β-reduction existente.
El nombre del parámetro se interpreta dentro de su RuleUse; no es una
sustitución global basada únicamente en coincidencias de nombres. La compilación
crea identificadores distintos para parámetros compartidos y parámetros locales.
Un binding proporcionado se convierte en Const; un parámetro pendiente conserva
su identidad de Var y su Arg tipado.

La [fixture](../examples/experiments/legal_geometry/explicit_substitutions.res)
importa el experimento general y declara un ámbito explícito de tres reglas
existentes: obligationCompliance, rightCompliance y filedObligation. No copia sus
cuerpos ni añade inferencias jurídicas. Usa:

```text
program CompliancePartial = ComplianceContext(gConduct)
program ComplianceBound = CompliancePartial(private, gDate)
program ComplianceGround = ComplianceBound(gObject, gTau)
```

ComplianceContext tiene cinco parámetros: conduct, actAuthority, actEffective,
object y t. La primera aplicación fija conduct. La segunda fija autoridad y
vigencia del acto; object y t permanecen libres y la consulta ground los resuelve
desde su objetivo. La tercera fija también esos dos valores. Los parámetros
locales de cada regla que no reciben bindings permanecen parametrizados.

La aplicación directa de los cinco argumentos y la aplicación por etapas dejan
los mismos RuleUses y el mismo módulo léxico; las identidades nominales de los
programas especializados pueden diferir. Se verifica la sustitución de object
en los argumentos, t en todos los índices temporales y actEffective como valor
temporal dentro de los argumentos de Act. Un argumento Authority en una posición
Conduct es rechazado por el especializador existente.
También se consulta un objetivo en gOtherTau: ComplianceBound puede instanciar
su t pendiente con ese tiempo, mientras ComplianceGround, fijado en gTau, no
produce una explicación. El tiempo ya aplicado no vuelve a tratarse como variable.

**Ámbito explícito y límite de la API.**

`query_program_residual` recibe Module, no ProgramRef ni un mapa de bindings.
El harness llama a `compile_program`, construye una vista de módulo con las tres
reglas especializadas seleccionadas y entrega esa vista al query real. Conserva
las declaraciones de tipos, verbos, constantes y programas, y los orígenes de las
reglas. No implementa una búsqueda residual distinta.

La vista **no es toda la ontología general**: por ejemplo, no contiene la vía
statutoryObligation/enactLaw. El conjunto de reglas de la consulta es una elección
explícita del experimento. La coincidencia que se observa con las dos fronteras
del dominio Σ no autoriza a suprimir esa vía de otras consultas o escenarios.

La llamada sobre el módulo importado completo sigue devolviendo Err por un
parámetro no determinado por el objetivo; declarar Program no hace que el query
lo seleccione automáticamente. Por ello esta fixture prueba elaboración y
especialización mediante el harness, no una nueva capacidad de selección de
Program desde `residuum query`. Esa integración pública queda pendiente.

**MODEL RESULT: fronteras condicionadas por evidencia.**

Se usa el objetivo compliance original, con gNorm, gAct, gSubject, gObject,
gBeneficiary, administrative, gDate y gTau. Sean O el obligationFiled de esa
instancia, R el right correspondiente y A su Act positivo con autoridad private.
Las fronteras independientes se obtienen utilizando las 27 reglas generales y
el universo E/Σ de la investigación geométrica anterior, no las respuestas del
query especializado.

| Evidencia | Soportes mínimos pendientes |
|---|---|
| ∅ | `{O,A}` o `{R,A}` |
| A | `{O}` o `{R}` |
| O | `{A}` |
| R | `{A}` |
| O y R | `{A}` |
| O y A | `∅` |
| R y A | `∅` |
| O, R y A | `∅` |

Se ejecutan los ocho casos con ComplianceBound y ComplianceGround: 16 consultas.
Cada una conserva dos árboles de prueba, distinguiendo obligationCompliance de
rightCompliance, incluso cuando sus requisitos coinciden o una rama ya basta.
La proyección a conjuntos y la selección de soportes mínimos se hacen sólo para
la comparación geométrica en el harness; no modifican ni descartan las pruebas
del buscador. Un requisito vacío se representa mediante Unit, no como evidencia
negativa ni como un evento que se añada a Known.

Cada una de las 32 ramas se certifica mediante cierres hipotéticos con la vista
especializada, la ontología general original y la patrimonial original: 96
comprobaciones de suficiencia, con auditoría de los registros forward. La evidencia
de cada consulta permanece intacta. Las referencias compiladas permiten recuperar
Program, RuleUse, regla jurídica original y módulo LegalGeneral. No se exige
igualdad literal de derivedAt entre ejecuciones independientes.

**Parámetros pendientes no equivalen todavía a búsqueda existencial.**

Después de fijar sólo conduct, la consulta local a rightCompliance conserva dos
parámetros pendientes: actAuthority : Authority y actEffective : PropositionTime.
object y t quedan determinados por el objetivo. Los identificadores están bajo
el ámbito de CompliancePartial, separados de los locales de cada regla.
`ResidualRequirement.parameters` y el antecedente parametrizado ya representan
esta información, pero la búsqueda sobre el programa parcial todavía devuelve
Err. No resuelve esos valores automáticamente ni enumera un dominio.

Esto respalda la primera etapa como especialización contextual: los valores
aportados son restricciones de la consulta, no evidencia de un acto ni valores
descubiertos por el motor. La segunda etapa deberá precisar resolución contra
Known, condiciones existenciales pendientes y dominios admisibles; aquí no se
implementa ni se declara completitud de esa búsqueda.

**Clasificación y reproducción.**

- IMPLEMENTED + TESTED: bindings tipados, β por etapas, ámbito local/compartido,
  sustitución temporal, consulta local con parámetros pendientes.
- MODEL RESULT: 16 consultas contextuales, dos pruebas por consulta, coincidencia
  de soportes mínimos en el universo señalado y certificación de suficiencia.
- NOT IMPLEMENTED: nueva API pública de query con substitutions/ProgramRef y
  búsqueda de variables existenciales. No se postula un teorema de completitud.

Verificador: `src/bin/residuum/cli/legal_geometry_explicit_query_tests.rs`.
Pasaron sus dos tests individuales y las dos regresiones de consultas anteriores
mediante `scripts/test_safe.py`; no se ejecutó la suite completa. La compilación
final consumió 396.92 MiB RSS y 4.0207 s. Los tests nuevos tardaron 0.1634 s y
0.5310 s, con picos de 15.56 y 15.35 MiB; las regresiones tardaron 0.3676 s y
0.3879 s. Se conservaron 1 GiB, cero swap, un trabajo de Cargo, un hilo y los
timeouts existentes. Log:
`target/test-diagnostics/20261010T011317274519Z-655347`.
Los exports conservan requisitos, parámetros y pruebas en
`docs/legal-geometry/data/explicit-query-{pending,frontiers}.tsv`.
Las fuentes jurídicas, main, v1 y el motor permanecen intactos en la rama
experiment-legal-order-geometry.
