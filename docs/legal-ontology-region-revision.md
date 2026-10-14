# Ampliaciones hipotéticas de la región de cumplimiento

Experimento en `experiment-legal-order-geometry`. No es una propuesta jurídica
aprobada ni una modificación de la ontología. Los datos son sintéticos y las
etiquetas son supuestos externos deliberados. No se utilizan expedientes de
12hectareas.

La pregunta es qué revisiones ground `H <= compliance(...)` son compatibles con
unas observaciones y qué geometría producen. La respuesta depende del dominio de
soportes y del criterio de minimalidad; no hay una revisión óptima sin especificarlos.

## Fuentes, casos y alcance

[region_revision.res](../examples/experiments/legal_geometry/region_revision.res)
importa `general.res`, que importa la ontología general vendor de 27 reglas.
El AST se elabora con el frontend actual. Los símbolos de los casos son:

| Símbolo | Evento y categoría |
|---|---|
| O | `obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate)`; Seeded |
| R | `right(...)` con los mismos siete argumentos de O; Seeded |
| A | `Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate)`; Seeded |
| P | `permissionFiled(...)` con los argumentos de O; Seeded |
| M | `omissionProhibition(...)` con los argumentos de O; Seeded |
| D | `disputedPerformance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate)`; Derived |
| C | `compliance(...)` con los siete argumentos de D; Derived, objetivo |
| V | `violation(...)` con los siete argumentos de D; Derived, sólo en el universo geométrico ampliado |

Todos tienen polaridad positiva e índice `@ gTau`. `gDate` es un argumento de
vigencia distinto del índice. `gConduct` es una constante diferente de `transfer`.
Las constantes son las sintéticas de `general.res`. No se ejecuta State.

**D y V son premisas de instantáneas de teoría**, no seeds autorizados, hipótesis
abducibles ni hechos que afirmemos haber derivado. La etiqueta negativa de un caso
significa «no debe derivarse C según los supuestos», no `EvidentialNot` de sus hechos.
Nada de este experimento interpreta `P/~P` como inconsistencia automática.

El catálogo de entrada es `T={O,R,A,P,M,D}` y el espacio de casos es `P(T)`:
64 conjuntos de evidencia. Este dominio es explícitamente restringido: no se
buscan soportes con otros individuos, autoridades, tiempos o verbos de la ontología.
«Todos los candidatos» significa todos los subconjuntos de este catálogo, no todas
las revisiones que podría expresar Residuum.

Las etiquetas se leen de
[region_revision_labels.tsv](../examples/experiments/legal_geometry/region_revision_labels.tsv),
separadamente de las reglas y de sus resultados:

| Etiqueta externa | Casos |
|---|---|
| Positivo ya cubierto | OA, RA |
| Positivo fuera de la región original | APD, APM |
| Negativo | vacío, A, P, D, M, PD, PM, O, R |

Son 13 observaciones: cuatro positivas y nueve negativas. No representan doctrina,
una decisión judicial ni una inferencia de que el permiso o la disputa justifican
cumplimiento. Elegir D permite examinar explícitamente revisiones circulares.

El parser actual exige `premises` con `query fourFusion`, y `seeds` con
`query residual`. Se conserva esa sintaxis: el harness lee el input y goal
elaborados de los experiments, **pero calcula cierres Horn, no entailment FOUR**.
No se amplía la CLI ni se presentan estos experiments como consultas FOUR
investigadas. Esta restricción de sintaxis no impide observar sus cierres con las
APIs existentes.

## Generación exhaustiva y tres criterios

La región original sobre los 64 casos es

\[
\mathcal R_0=\uparrow\{O,A\}\cup\uparrow\{R,A\}.
\]

Contiene 24 casos y su frontera sigue siendo OA, RA. Para cada uno de los 64
soportes H se calcula su cono literal `↑H={S : H⊆S}`. Un soporte es admisible para
la búsqueda de extensiones mínimas si:

1. su cono contiene al menos un positivo externo no cubierto;
2. su cono no contiene ninguna observación negativa.

Se omiten soportes sin contribución a los positivos: añadirlos nunca es necesario
para una extensión mínima por eliminación de reglas. Los paquetes arbitrarios con
reglas irrelevantes no son el objeto de la búsqueda.

Se obtienen exactamente cinco soportes admisibles:

| H | Tamaño del cono | Casos nuevos fuera de R₀ | Mínimo por inclusión de soportes |
|---|---:|---|---|
| AP | 16 | AP, APM, APD, APMD | sí |
| AM | 16 | AM, APM, AMD, APMD | sí |
| AD | 16 | AD, APD, AMD, APMD | sí |
| APM | 8 | APM, APMD | no: contiene AP y AM |
| APD | 8 | APD, APMD | no: contiene AP y AD |

Cada candidato propone una regla ground H→C; son hipótesis, no nuevas reglas de
la biblioteca. Se verifica que su región adicional calculada mediante cierre
coincida con `↑H \ R₀`. Esa igualdad es un resultado de este catálogo, no una
identificación universal entre cono literal y habilitación tras cerrar evidencia.

Se enumeran los 32 subconjuntos de estos cinco candidatos. 25 son compatibles con
las 13 etiquetas. Exactamente cinco no admiten quitar una regla sin dejar de
cubrir algún positivo. Son **todas las extensiones mínimas por eliminación de
reglas dentro de esta familia**:

| Paquete hipotético | Nuevas reglas | Frontera añadida a OA, RA | Casos adicionales | Antiintercambio |
|---|---:|---|---:|---|
| {AP→C} | 1 | AP | 4 | conserva |
| {AD→C, AM→C} | 2 | AD, AM | 6 | falla |
| {AD→C, APM→C} | 2 | AD, APM | 5 | falla |
| {APD→C, AM→C} | 2 | APD, AM | 5 | falla |
| {APD→C, APM→C} | 2 | APD, APM | 3 | falla |

Los criterios no son equivalentes:

- **Inclusión de soportes:** AP, AM y AD no tienen un soporte admisible propio.
  Reducir un soporte amplía su cono. Entre los cinco paquetes mínimos, AP y
  AD+AM usan exclusivamente esos soportes mínimos.
- **Número mínimo de reglas:** sólo AP cubre ambos positivos con una regla.
  Un paquete mínimo por eliminación puede tener dos reglas y no ser mínimo en
  número de reglas.
- **Cobertura adicional mínima por inclusión:** sólo APD+APM tiene la región
  adicional más pequeña entre los 25 paquetes compatibles. Sus tres casos nuevos
  son APM, APD y APMD. Esa región está incluida estrictamente en las otras cuatro
  regiones de la tabla. Aquí coincide también con la menor cardinalidad de
  cobertura, pero no se supone esa coincidencia en general.

AP captura un caso extra AP para ahorrar una regla. APD+APM conserva condiciones
más específicas, pero usa D y destruye antiintercambio. Ningún criterio es declarado
política del lenguaje ni criterio jurídico correcto.

## Cierres independientes y procedencia

[El harness](../src/bin/residuum/cli/legal_geometry_region_revision_tests.rs)
no modifica el motor. Genera reglas ground tipadas sobre copias del módulo y
etiqueta su origen `HypotheticalComplianceRevision`. El módulo original y sus
inputs se comparan antes/después. Las declaraciones originales quedan intactas,
en el mismo orden, con sus orígenes de módulo y archivo.

Los cinco soportes se exportan como Formal a
`target/legal-geometry/hypothetical-compliance-supports.res`. El propio test vuelve
a cargarlos mediante parser, resolución de imports y elaborador, y exige el mismo
AST tipado de cada regla. Es una comprobación adicional de que el generador no
fabrica reglas que el frontend no pueda expresar. El archivo exportado agrupa
candidatos para inspección; **no representa un paquete seleccionado**.

Para baseline y los cinco paquetes mínimos se comparan las 64 entradas:
384 evaluaciones por método. Se utilizan:

1. saturación independiente de una tabla ground con bitsets;
2. `close_program_reference`, cierre de referencia;
3. `close_program`, cierre incremental actual.

Los eventos completos de ambos motores coinciden, incluidos tipos, polaridades,
argumentos, tiempos y metadata de transición. Sus registros de provenance también
coinciden como listas ordenadas, preservando multiplicidad, regla, antecedentes
ordenados y sustituciones canónicas. Se ignora sólo el valor de `derivedAt` al
comparar ejecuciones, siguiendo la especificación.

Cada registro se valida volviendo a instanciar su regla sobre sus testigos; el
origen se recupera del módulo y se contrasta con el original o con el origen
experimental. Las reglas nuevas son ground y su sustitución es vacía; las reglas
jurídicas originales conservan sus sustituciones. La igualdad de eventos entre
paquetes **no obliga a igualdad de provenance**: agregar una regla redundante
puede añadir otra justificación válida.

Las verificaciones no insertan hipótesis en el Known original ni ejecutan efectos.
Cada cierre hipotético parte de una copia de la evidencia. No hay selección de
defaults, cambios de activación ni prioridades en este experimento.

## Geometría: comprobación completa en otro dominio explícito

Para estudiar antiintercambio se usa un universo mayor:

\[
E'=C_G(T\cup\{V\}).
\]

Tiene 12 eventos: O, R, A, P, M, D, V, C, `obligation`, `permission`,
`patrimonialLiability` y `do @ after gTau`. El evento `do` se deriva pero no se
ejecuta. Las únicas siete instancias ground no redundantes originales que actúan
sobre E' provienen de `filedObligation`, `filedPermission`,
`obligationCreatesRelation`, `obligationCompliance`, `rightCompliance`,
`performanceDispute` y `violationLiability`.

E' es cerrado para las reglas originales y las hipotéticas: sólo se añade C,
que ya pertenece a E'. Se saturan las 4.096 entradas de `P(E')`, por separado,
para el baseline y los 25 paquetes compatibles. No se enumera todo el universo
jurídico anterior de 73 puntos. La tabla ground procede del matcher sobre el AST
real; la saturación de bitsets es independiente de la planificación del evaluador.
Sus resultados sobre los 384 casos se contrastan con ambos motores; los testigos
de antiintercambio se reproducen además con el motor real y se audita su provenance.

**MODEL RESULT:** los 26 cierres son extensivos, monótonos e idempotentes, con
vacío cerrado. Nueve conservan antiintercambio y diecisiete lo violan. Los números
incluyen el baseline y paquetes redundantes. Entre las cinco extensiones mínimas
por eliminación, sólo AP lo conserva.

Para todo estado cerrado F se calculan los extremos quitando cada punto:

\[
ex(F)=\{x\in F:x\notin C(F\setminus\{x\})\}.
\]

En las nueve geometrías con antiintercambio, `C(ex(F))=F` para todos sus estados
cerrados. En las diecisiete restantes hay estados donde los extremos no generan F,
y se enumeran varias bases mínimas del estado testigo.

La prueba anterior de geometría convexa exigía un grafo inferencial acíclico.
Las hipótesis con AD o APD pueden infringir precisamente esa condición. No se
traslada la demostración a ellas por tener observaciones compatibles.

### Contraejemplo concreto

Para AD→C, sea F el cierre original de `{A,V}`. F incluye
`patrimonialLiability`, pero no C ni D. Entonces:

- agregar D produce C por la hipótesis AD→C;
- agregar C produce D por `performanceDispute: C & V <= D`.

Por tanto `C∈C_H(F∪{D})` y `D∈C_H(F∪{C})`, con C,D fuera de F:
antiintercambio falla. Las bases `{A,V,C}` y `{A,V,D}` generan el mismo estado
cerrado; ya no hay base mínima única. Para APD→C se necesita además P: se toma
`F=C_G({A,P,V})`. El fenómeno no utiliza ausencia de evidencia ni P/~P.

**Contraejemplo a otra simplificación:** AP+APD contiene una arista circular
sintáctica D→C, pero APD es redundante porque AP ya produce C. Su operador de
cierre coincide con AP sobre las 4.096 entradas y conserva antiintercambio.
Un ciclo sintáctico por sí solo no demuestra pérdida geométrica. La aciclicidad
es suficiente para el argumento anterior, no necesaria para toda representación.

## Casos que discriminan hipótesis

Las cinco extensiones mínimas son indistinguibles en las 13 observaciones, pero
no tienen la misma región. Los tres casos aún sin etiqueta AP, AD y AM bastan
para distinguirlas:

| Paquete | AP | AD | AM |
|---|---|---|---|
| AP | sí | no | no |
| AD+AM | no | sí | sí |
| AD+APM | no | sí | no |
| APD+AM | no | no | sí |
| APD+APM | no | no | no |

Sus firmas son distintas. Preguntar por su clasificación externa separaría las
hipótesis; **el experimento no decide cuál debe ser la respuesta jurídica**.
Los casos son conjuntos de premisas completamente tipados, con las constantes,
polaridades y tiempos anteriores, no nombres informales de características.

Los paquetes AP, AP+APD, AP+APM y AP+APD+APM sí tienen exactamente la misma
región y cierre de eventos en este dominio. Ningún caso de este dominio puede
distinguirlos por derivabilidad; sus reglas y justificaciones pueden distinguirlos.
Para todas las parejas de paquetes mínimos se exportan casos discriminantes
mínimos en `revision-distinguishing-cases.tsv`.

## Qué está demostrado y qué no

**THEOREM, condicional:** un sistema Horn ground positivo sobre un universo finito
cerrado induce un operador extensivo, monótono e idempotente por saturación.
Extensividad se sigue de conservar la entrada; monotonicidad, de que una premisa
verdadera sigue presente al ampliar la evidencia; idempotencia, de que la salida
es un punto fijo. Añadir reglas también amplía su cierre. Estas propiedades no
implican antiintercambio.

**THEOREM, condicional:** si H⊆H', entonces `↑H'⊆↑H`. Todos los generadores de
un estado cerrado F contienen `ex(F)`: si faltara un extremo x, estarían dentro de
`F\{x}`, que por definición no deriva x. Si `ex(F)` genera F, es su único
generador mínimo. El test examina esa condición, no la presupone.

**MODEL RESULT:** enumeración de candidatos, minimalidades, tamaños de conos,
fronteras, clasificación geométrica y contraejemplos dentro de los dominios T y E'.

**IMPLEMENTED + TESTED:** equivalencia de los 384 cierres y registros, elaboración
de candidatos exportados, inmovilidad de inputs/módulo original y reproducción
operacional de los testigos de antiintercambio.

No se demuestra que los soportes sean jurídicamente correctos, que las etiquetas
sean exhaustivas, ni que esas propiedades sobrevivan al cambiar individuos,
autoridades, tiempos, universo de soportes, defaults o programas dinámicos.
No se generalizan automáticamente estas reglas ground a lambdas universales.
No hace falta una conjetura de geometría euclidiana para los resultados obtenidos.

## Artefactos y reproducción

Resultados compactos persistentes:
[docs/legal-geometry/region-revision](legal-geometry/region-revision/).
Los tests regeneran en `target/legal-geometry/`:

- `revision-supports.tsv`, `revision-cones.tsv`: soportes y membresía de cada cono.
- `revision-packages.tsv`: las 32 combinaciones y sus criterios de minimalidad.
- `revision-distinguishing-cases.tsv`: casos que separan hipótesis mínimas.
- `revision-geometric-universe.tsv`, `revision-geometric-properties.tsv`.
- `revision-engine-closures.tsv`, `revision-provenance.tsv`: 384 estados y sus registros.
- `revision-anti-exchange-counterexamples.tsv`: testigos con eventos y bases completas.
- `hypothetical-compliance-supports.res`: candidatos exportados para inspección.

Se usa únicamente `scripts/test_safe.py`, con 1 GiB, sin swap, un trabajo de
Cargo y un hilo; sin suite completa. Los nombres individuales están en
`src/bin/residuum/cli/legal_geometry_region_revision_tests.rs`.
Las pruebas anteriores de fronteras y antiintercambio se mantienen como regresiones.
Pasaron los tres tests nuevos y ambas regresiones, cada uno en proceso separado:
0,549 s, 0,511 s y 0,306 s para las pruebas nuevas; 1,551 s y 1,121 s
para las regresiones (el tiempo exacto actualizado figura en `verification.tsv`).
Todos los tests quedaron por debajo de 18 MiB RSS. La última compilación consumió
344,05 MiB RSS y 2,199 s. No hubo timeout ni OOM.
La compilación de tests utiliza `CARGO_PROFILE_TEST_DEBUG=0`, sin alterar semántica.

Fuentes jurídicas intactas, SHA-256:

- Original: `24655c98ede7b128b8cd82df3a4473c114428b987bb0fe6a6d370c2b37f5575e`.
- General vendor: `639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc`.
- Patrimonial vendor: `f7daf4d5eb937a7fde8534dba5618acc50d5aaa5b05ff86c9656892eb3a03fbf`.

No se modifica engine, ontología, main ni v1. No hay commit, tag, merge ni push
como parte de este experimento.

## Dependencias y justificaciones: investigación posterior

El [análisis de dependencias](legal-revision-dependencies-and-justification-boundary.md)
separa equivalencia del operador completo de coincidencia en los casos observados.
Los cinco paquetes mínimos de esta familia conservan la clasificación anterior.
Controles hipotéticos fuera de esa familia mantienen los 64 cierres iniciales,
pero cambian geometría en el universo ampliado. Ninguno de estos resultados
identifica una justificación normativa desconocida.
