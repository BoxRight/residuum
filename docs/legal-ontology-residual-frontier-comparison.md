# Fronteras geométricas frente a consultas residuales

La primera etapa posterior, con parámetros proporcionados explícitamente, está
documentada en [consultas con sustituciones explícitas](legal-ontology-explicit-query-substitutions.md).
Reutiliza β en un ámbito de reglas seleccionado por el harness; no cambia los
límites del query sobre el módulo completo descritos aquí.

El buscador residual actual **no recupera las alternativas mínimas de las
principales conclusiones regulativas de esta ontología**. Las reglas son válidas
para el cierre forward, pero varios parámetros de sus antecedentes no quedan
determinados por el objetivo. `query_program_residual` rechaza ese caso mediante
Err, tal como especifica su contrato vigente.

En las consultas que completan dentro del dominio finito, los soportes
obtenidos coinciden con la frontera geométrica. Esto no demuestra completitud
para toda la ontología: todos esos éxitos desde evidencia vacía tienen una sola
alternativa dentro del dominio, y los objetivos con varias alternativas relevantes
encuentran la limitación antes de poder enumerarlas.

**Alcance y criterio de comparación.**

Se conserva `E=C_P(Σ)`: 73 eventos ground, 23 Seeded, las mismas fuentes y
SHA-256 del [informe geométrico](legal-ontology-order-geometry.md). Se comparan
ambos módulos jurídicos sin cambiar declaraciones, matching, AST, efectos,
defaults ni consultas. No se ejecuta State ni se utilizan datos del expediente.
La rama sigue siendo `experiment-legal-order-geometry`, desde main
`7b14cb8ac86d07c8d101dc36ca9f1161580f6c22`; main y v1 permanecen intactos.

La comparación principal usa evidencia vacía y las fronteras restringidas a Σ
del [experimento anterior](legal-ontology-minimal-frontiers.md):

\[
\partial^\Sigma_q=\min_{\subseteq}\{H\subseteq\Sigma\mid q\in C(H)\}.
\]

No se compara contra la frontera irrestricta que incluye `{q}` para todo evento
Derived: el buscador sólo admite hipótesis Seeded por defecto. Una conclusión
Derived puede ser evidencia conocida, pero no se inventa como hipótesis.

Cada rama devuelta conserva su `ResidualProof`, sustituciones y referencias a
reglas. Para contrastarla con la geometría se proyectan sus factores pendientes
a un conjunto de Event y se minimizan los conjuntos **sólo en el harness**.
Esto no identifica Tensor con unión universalmente ni cambia el query: es una
comparación con el soporte operacional de Known como conjunto. Las ramas y la
multiplicidad de pruebas se guardan por separado.

Para cada respuesta se calcula un cierre hipotético independiente y se verifica
que contiene el objetivo. Se auditan todos sus DerivationRecord y los orígenes
de las reglas. La evidencia original permanece intacta. No se exige identidad
literal entre un árbol ResidualProof y registros forward, pues son objetos
distintos; ambos conservan las referencias y los testigos que corresponden.

**MODEL RESULT: consultas a los 73 objetivos por módulo.**

Se llama a `query_program_residual` real con sus SearchLimits por defecto, sin
modificar el programa ni precalcular sus respuestas:

| Resultado de la consulta | General | Patrimonial |
|---|---:|---:|
| Completa y coincide con la frontera mínima dentro de Σ | 30 | 29 |
| Err: parámetro no determinado por el objetivo | 39 | 44 |
| Completa, pero propone evidencia fuera de Σ/E | 4 | 0 |
| Total | 73 | 73 |

De los éxitos que coinciden, 23 por módulo son objetivos Seeded: el query
devuelve ese mismo hecho como requisito abducible. Los otros son:

- General: tres permissions, inconsistent, legislativeCapacity, transferred y
  el patrimonialAsset con causa gHolding.
- Patrimonial: los mismos salvo el asset gHolding, cuya consulta encuentra la
  nueva regla complianceAsset y devuelve Err.

No hubo fallos por límites de búsqueda, ciclos o recursos. Cada conjunto de
hipótesis que se devolvió permite derivar su objetivo mediante el cierre real.
No aparece un contraejemplo de suficiencia entre estas respuestas completadas.
Un Err no representa un conjunto vacío de explicaciones, una ausencia de
derivación ni un resultado parcial que pueda usarse como respuesta completa.

**La frontera principal: parámetros ocultos por la conclusión.**

Ejemplo real:

```text
obligation(norm, subject, conduct, object, beneficiary, authority, effective) @ t
& Act(act, subject, conduct, object, beneficiary, actAuthority, actEffective) @ t
<= compliance(norm, act, subject, object, beneficiary, authority, effective) @ t
```

El objetivo compliance fija norm, act, subject, object, beneficiary, authority,
effective y t, pero no conduct, actAuthority ni actEffective. El buscador comprueba
que todos los parámetros quedaron fijados **antes de evaluar el antecedente**.
Por eso devuelve exactamente:

```text
rule `obligationCompliance` has parameter `conduct` not determined by the goal
```

Aunque Act conocido pueda determinar esos valores, el algoritmo vigente no
utiliza ese matching para resolverlos en esta fase.

| Objetivo seleccionado | Soportes geométricos Seeded | Primera frontera operacional observada |
|---|---:|---|
| compliance | 2 | obligationCompliance: conduct |
| violation | 5 | obligationViolation: conduct |
| enforceability | 2 | rightEnforceability: conduct |
| disputedPerformance | 6 | expansión de compliance: conduct |
| annulled | 4 | permissionAnnulsProhibition: subject |
| law | 1 | enactLaw: act |
| do | 4 | obligationCreatesRelation: id |
| notDo | 3 | prohibitionCreatesAbstention: id |

Se informa el primer parámetro señalado, no una lista exhaustiva de todos los
no determinados. El orden de exploración puede cambiar cuál se encuentra primero.
Para obligation gNorm también se aborta: la rama filedObligation es válida, pero
la expansión de la alternativa statutoryObligation lleva a enactLaw y su act libre.

La capacidad que falta para esta comparación es **resolver parámetros del
antecedente que no aparecen en la cabeza durante la búsqueda backward**,
preservando su ámbito y tipo. No falta una regla jurídica ni una inferencia
forward. No se diseña ni implementa esa extensión en este experimento.

**Consulta local y evidencia parcial.**

`query_residual`, sobre una Rule individual, tiene otro contrato. Puede devolver
un requisito simbólico con parámetros pendientes en `ResidualRequirement`:

| Consulta local a rightCompliance | Resultado |
|---|---|
| Goal compliance; evidencia vacía | right & Act con conduct, actAuthority y actEffective pendientes |
| Mismo goal; Act coincidente conocido | right ground; ningún parámetro pendiente |

En el segundo caso ese right es exactamente una de las dos completaciones
geométricas con Act conocido. La otra utiliza obligationFiled y requiere resolver
además la cadena filedObligation. El test no introduce un adaptador que construya
artificialmente una búsqueda completa a partir de llamadas locales.

La consulta de programa a compliance devuelve Err con evidencia vacía, con Act
solo y con obligationFiled + Act aportados como hechos sin cerrar. Si se entrega
el **cierre** de los dos últimos hechos, compliance ya está conocido y devuelve
Unit mediante Evidence(compliance). Esa respuesta es correcta: no falta evidencia.
No demuestra que el buscador haya reconstruido las alternativas anteriores.

Para inconsistent todos los parámetros sí aparecen en el objetivo. Se comparan
las cuatro completaciones de la evidencia histórica explícita en ambos módulos:

| Evidencia disponible | Requisito residual y frontera de completación |
|---|---|
| ∅ | `{Act, ~Act}` |
| Act | `{~Act}` |
| ~Act | `{Act}` |
| Act y ~Act | `∅`, representado por Unit |

En los ocho casos coinciden y el cierre hipotético verifica suficiencia. Unit
es un producto de requisitos sin factores, no una identificación con conocimiento
vacío. No se obtiene ~Act por ausencia y inconsistent sigue siendo un Derived
ordinario.

**Diferencia de dominio abducible.**

Las cuatro consultas generales de patrimonialAsset con causas gHigh, gLow,
gNorm y gStatute devuelven una hipótesis patrimonialHolding con esa misma causa.
Ejemplo para gNorm:

```text
patrimonialHolding(gNorm, gBeneficiary, gObject, administrative, gDate) @ gTau
```

Ese hecho es Seeded y produce el asset por existingAsset, con suficiencia
verificada. Pero no pertenece a Σ ni a E: el holding declarado en la fixture
tiene causa gHolding. Por eso no contradice la frontera geométrica vacía para
ese asset en G/Σ. El query admite hipótesis según verbo y polaridad; su API no
restringe las instancias ground al catálogo finito de hechos de este experimento.

Se conserva la respuesta y se clasifica fuera del dominio. No se amplía
silenciosamente Σ ni se considera su respuesta una recuperación de la frontera
original. Para esas comparaciones hace falta delimitar el dominio de hipótesis,
además de resolver los parámetros pendientes.

**Añadir reglas puede salir del fragmento soportado por el query.**

En G, el asset con causa gHolding tiene una consulta completa mediante
existingAsset que recupera su soporte único. En P, la misma consulta alcanza
también complianceAsset, cuyos parámetros act y subject no quedan fijados por
el asset objetivo. El primer error es act y se aborta toda la consulta: no se
devuelve la alternativa válida como si fuera una búsqueda completa.

Esto no altera `C_G(S)⊆C_P(S)` ni la suficiencia de esa alternativa. Es una
limitación del fragmento operacional del buscador frente a ampliaciones de reglas,
no una pérdida de monotonicidad del cierre jurídico. No se corrige el motor ni
se omite la regla para obtener una comparación favorable.

**Artefactos y verificación.**

- [Clasificación de las 146 consultas](legal-geometry/data/residual-frontier-comparison.tsv).
- [Hipótesis, pruebas residuales, orígenes y records forward](legal-geometry/data/residual-frontier-witnesses.tsv).
- [Casos locales, evidencia parcial y parámetros pendientes](legal-geometry/data/residual-frontier-boundaries.tsv).
- [Resumen estructurado](legal-geometry/residual-frontier-results.json).

```text
python3 scripts/test_safe.py --test-target cli \
  --test cli::tests::legal_geometry::residual_comparison::actual_program_query_reports_its_boundary_against_ground_frontiers \
  --test cli::tests::legal_geometry::residual_comparison::local_residual_and_partial_evidence_make_the_missing_parameter_boundary_explicit
python3 scripts/analyze_legal_geometry.py --render
```

Pasaron ambos tests individuales. Los tests comprueban también los errores
esperados: su resultado verde no implica que el query pueda cubrir la ontología.
No se ejecutó la suite completa. Se conservan 1 GiB, sin swap, un job de Cargo,
un hilo y los límites temporales del runner, además de los límites de búsqueda
residual por defecto.

Verificación final en `target/test-diagnostics/20261010T005801583675Z-641915`:
consulta de los 146 objetivos, 0.3875 s / 15.57 MiB RSS; consultas locales y
evidencia parcial, 0.3886 s / 15.57 MiB. La compilación inicial del harness
consumió 414.95 MiB, y la final después del ajuste de las completaciones
geométricas exportadas, 349.44 MiB. No hubo timeout ni OOM.

El único trabajo nuevo es el módulo de tests
`src/bin/residuum/cli/legal_geometry_residual_tests.rs` y los artefactos de
análisis. Los resultados son finitos y diagnósticos, no una demostración de
completitud general del residual. La investigación se detiene en las dos
fronteras documentadas; no incorpora una nueva semántica para superarlas.
