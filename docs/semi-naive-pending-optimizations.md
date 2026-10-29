# Optimizaciones pendientes del cierre semi-naive

Inventario de 2026-10-29, posterior a la rama `semi-naive-dedup-index`
(índice en `IncrementalClosure`, deduplicación con `BTreeSet`, pivotes sin
repetición y `new_by_derivative`). Este documento no modifica código ni
contratos. Ordena trabajo posible; ninguna entrada está medida todavía y su
impacto es una estimación por lectura del código, no un resultado.

Cada entrada separa el problema observado, la propuesta, lo que debe
conservarse y cómo verificarla. Salvo que se diga lo contrario, todas deben
preservar los eventos lógicos, el conjunto de justificaciones, su multiplicidad
de un sello por justificación y la independencia de `DerivationTime` respecto
de rondas y `PropositionTime` ([discrete-derivative.md](discrete-derivative.md) §§3, 6).

## Resumen

| # | Optimización | Impacto estimado | Riesgo | Toca |
|---:|---|---|---|---|
| 1 | Índice por argumentos ya ligados | Alto | Medio | `known_index.rs`, matching |
| 2 | Mapa verbo → reglas relevantes | Medio | Bajo | `incremental.rs`, `program_incremental.rs` |
| 3 | Orden de uniones por selectividad | Medio | Medio | `match_delta_antecedents` |
| 4 | Evaluación por componentes de dependencia | Medio | Medio | schedulers de cierre |
| 5 | Menos copias en el join | Medio (constante) | Bajo | matching |
| 6 | Rondas con instantánea de Known | Bajo–medio | Medio | `update`, `drain` |
| 7 | Explicación bajo demanda | Alto en memoria | Alto (contrato) | `Closure`, `explain` |
| 8 | Procedencia en semianillos | Nueva capacidad | Alto (diseño) | capa de consulta |
| 9 | Prueba mínima: tropical y (min, max) | Nueva capacidad; `explain` más legible | Medio | capa de procedencia, `explain` |
| 10 | Uniones *worst-case optimal* | Alto en cuerpos cíclicos | Alto | matching |
| 11 | Evaluación dirigida por objetivo | Depende de la consulta | Alto | residual / forward |

Fuera de alcance por diseño: `close` sigue siendo el cierre ingenuo de
referencia y oráculo de comparación. Las optimizaciones van en los evaluadores
incrementales; `new_by_derivative` ya ofrece una base construida por derivada.

## 1. Índice por argumentos ya ligados

**Problema.** `KnownIndex` agrupa sólo por verbo, polaridad y aridad. En
`present(x) & motive(x)`, con `x` ya ligado por el pivote, el join recorre
todos los `motive(...)` y llama a `match_event` con cada uno.

**Propuesta.** Índice secundario por posición de argumento y término ground,
consultado cuando el patrón tiene esa posición ya ligada en los bindings
actuales. Puede construirse de forma perezosa por (verbo, posición) la primera
vez que un patrón lo requiera.

**Conservar.** `Candidates` debe seguir devolviendo eventos en el orden de
Known para que el orden de descubrimiento, y por tanto la asignación de sellos,
no cambie. Los términos no ground (`RecordLiteral` con variables, constructores
temporales) siguen por el camino actual.

**Verificar.** Comparar indexado contra `profile::linear_scan()` como en
`indexed_known_matches_linear_product_with_identical_provenance`: misma
igualdad `Debug` de cierres y mismos conteos salvo `probes` y `header_rejects`.

## 2. Mapa verbo → reglas relevantes

**Problema.** Cada ronda de `update` examina todas las reglas
(`rules_examined`), aunque ningún evento de la frontera tenga un verbo que
aparezca en sus antecedentes.

**Propuesta.** Precalcular, al construir el estado, el conjunto de reglas por
(verbo, polaridad) de sus átomos. Por ronda, examinar sólo la unión de las
reglas asociadas a los encabezados de la frontera. Igual para las reglas de
activaciones en `IncrementalProgramClosure`.

**Conservar.** El orden relativo de las reglas examinadas debe ser el orden de
`rules`, para no alterar el orden de descubrimiento. `rules_examined` bajará;
los tests que fijan `rules_examined == 0` en actualizaciones duplicadas siguen
valiendo.

**Verificar.** Los tests existentes de igualdad con `close` y un test que
compare la secuencia de `delta.derivations` con y sin el filtro.

## 3. Orden de uniones por selectividad

**Problema.** Tras el pivote, los demás conjuntos se unen en el orden escrito
en la regla. Un átomo muy selectivo al final multiplica resultados intermedios.

**Propuesta.** Elegir el orden de los no-pivotes según el tamaño de su bucket
en el índice (o según argumentos ya ligados, si existe el índice del punto 1).

**Conservar.** El vector de testigos ya se reconstruye en el orden original;
eso debe mantenerse. El orden de los matches devueltos puede cambiar, así que
el resultado debe reordenarse a un orden canónico antes de retenerse, o este
cambio debe aceptar explícitamente otra asignación de sellos.

**Verificar.** Igualdad de conjuntos de justificaciones contra `close`; si se
reordena canónicamente, igualdad exacta de secuencias.

## 4. Evaluación por componentes de dependencia

**Problema.** Todas las reglas participan en cada ronda hasta el punto fijo
global, aunque partes no recursivas del programa ya se hayan estabilizado.

**Propuesta.** Grafo de dependencias verbo de antecedente → verbo de
consecuente; evaluar sus componentes fuertemente conexas en orden topológico,
cada una hasta su punto fijo. Con activaciones de `Verb.program`, el grafo
crece en tiempo de ejecución; el orden debe recalcularse o limitarse a las
reglas ordinarias.

**Conservar.** El conjunto de eventos y justificaciones es el mismo menor
punto fijo (§3 admite schedules distintos). El orden de descubrimiento sí
cambia: decidir si se acepta o si se documenta como modo separado.

**Verificar.** Comparación por conjuntos contra `close`, incluidos programas
con ciclos (`program_incremental_cycles.res`).

## 5. Menos copias en el join

**Problema.** Por cada candidato se clonan `bindings` (`BTreeMap`) y el vector
de testigos, y los testigos son `Event` completos.

**Propuesta.** Testigos como posiciones en Known (`usize`) durante el join,
convertidos a `Event` sólo al construir el `AntecedentMatch` final; bindings
como pila con deshacer en vez de mapas clonados. A más largo plazo, internar
eventos con identificadores estables.

**Conservar.** `AntecedentMatch` y `DerivationRecord` no cambian de forma.

**Verificar.** Mismos resultados exactos; medir con el profiler
`profile_semi_naive_polynomial_product_fixture`.

## 6. Rondas con instantánea de Known

**Problema.** Known crece a mitad de ronda (estilo Gauss-Seidel). Un match
puede descubrirse en una ronda y otra vez en la siguiente; el `BTreeSet` lo
descarta, pero el trabajo de encontrarlo ya se hizo. Tras `semi-naive-dedup-index`
no quedan repeticiones dentro de una misma llamada, sólo entre rondas.

**Propuesta.** Cada ronda lee Known hasta la longitud que tenía al comenzar;
los eventos de la ronda actual se usan desde la siguiente. Con las posiciones
anteriores al pivote fuera de la frontera y las posteriores dentro de la
instantánea, cada match se produce exactamente una vez en toda la evaluación.

**Conservar.** El menor punto fijo. Puede aumentar el número de rondas y
cambiar el orden de descubrimiento y de sellos.

**Verificar.** Que `full_matches` sea igual al número de justificaciones
retenidas en las fixtures forward, además de la igualdad con `close`.

## 7. Explicación bajo demanda

**Observación matemática.** Por la definición del §6, las justificaciones son
función del cierre: \(J_R(X)\) es el conjunto de instancias con todos sus
testigos en X. Guardarlas durante el cierre forward es un caché, no una
necesidad lógica. Para el goal de un escenario basta recorrer hacia atrás su
cono: instancias cuya conclusión es el evento y cuyos testigos están en el
cierre, recursivamente. El costo es proporcional a la explicación, no al
cierre completo. ΔJ es la misma derivada aplicada a J.

**Pendiente de investigar.**

- `derivedAt` proviene del reloj externo y no se puede recalcular desde el
  cierre. Si `explain` lo necesita, debe guardarse o sustituirse por un dato
  canónico, como la etapa de Kleene: la menor ronda n con el evento en Tⁿ.
- Si `explain` debe mostrar todas las justificaciones del expediente o sólo las
  del cono del goal.
- Defaults, cláusulas residuales y activaciones de programas: su evaluación no
  es Horn positivo puro y requiere conservar estado de selección y activación.

**Conservar.** El contrato actual de `ClosureDelta.derivations`. Un modo bajo
demanda sería una API nueva, no un reemplazo.

## 8. Procedencia en semianillos

**Observación.** Green, Karvounarakis y Tannen (2007) unifican las formas de
procedencia como evaluación de la misma expresión en distintos semianillos.
Para Residuum:

| Semianillo | Pregunta |
|---|---|
| Booleano (∨, ∧) | ¿se deriva? (el cierre actual) |
| Conjuntos de soportes | ¿qué seeds bastan? (*why-provenance*) |
| Tropical (min, +) | prueba más pequeña o más barata |
| (min, max) | profundidad mínima: la etapa de Kleene |
| Conteo (+, ×) | cuántas derivaciones hay |

Como el cierre es monótono, la familia de conjuntos de seeds que derivan un
goal es ascendente en \(\mathcal P(\text{Seeds})\). Queda determinada por su
anticadena de soportes mínimos: es la misma estructura que usa el experimento
de [recalcitrancia](legal-recalcitrance-justificatory-functions.md)
con anticadenas de soportes. Esa anticadena puede mantenerse
con la derivada aditiva.

Con suma idempotente (booleano, soportes, tropical, (min, max)) la evaluación semi-naive
sigue siendo válida; el conteo requiere otro tratamiento porque su suma no es
idempotente. Khamis, Ngo, Pichler, Suciu y Wang (PODS 2022) estudian la
convergencia de Datalog sobre semianillos. El tropical se desarrolla en el
punto 9.

**Pendiente de investigar.** Interpretación de `EvidentialNot`, del axioma
`I <= C` (unidad del semianillo) y de la fusión FOUR en este marco.

## 9. Prueba mínima: semianillo tropical y (min, max)

**Idea.** En el semianillo tropical (min, +), el «y» de un cuerpo suma costos
y el «o» entre justificaciones alternativas toma el mínimo. Asignando un costo
a cada regla (y a cada seed, 0 por defecto), cada evento recibe

\[
c(e)=\min_{j\in J,\;head(j)=e}\Bigl(w(rule(j))+\sum_{a\in witnesses(j)}c(a)\Bigr).
\]

Con costo 1 por regla, \(c(e)\) es el **tamaño** del menor árbol de prueba
(subpruebas compartidas cuentan una vez por uso). Con pesos por tipo de regla o
de evidencia, es la explicación «más barata».

La **profundidad** no sale de (min, +), sino del semianillo (min, max): mínimo
entre justificaciones y máximo entre testigos,
\(d(e)=\min_j\bigl(1+\max_{a}d(a)\bigr)\), con \(d=0\) para seeds y \(d=1\)
para un axioma `I <= C`. Ése es exactamente la etapa de Kleene del punto 7, la
menor ronda n con el evento en Tⁿ, y es el dato reproducible que puede
sustituir a `derivedAt` en comparaciones. Ambos semianillos tienen suma
idempotente; todo lo que sigue vale para los dos.

**Usos.** `explain` puede presentar primero la prueba de costo mínimo, y las
demás justificaciones sólo a petición; los empates se conservan, no se
desempatan por orden de descubrimiento. Es independiente del reloj y del
schedule, así que sirve para comparar ejecuciones.

**Algoritmo.** Es un problema de caminos más cortos en el hipergrafo de
justificaciones (Datalog tropical ≈ Bellman-Ford). Con pesos no negativos,
la generalización de Dijkstra de Knuth (1977) para hipergrafos fija cada evento
una sola vez en orden de costo creciente. La suma min es idempotente, así que
la evaluación semi-naive sigue siendo válida, y la convergencia está asegurada
con pesos no negativos aunque haya ciclos.

**Consecuencia para la derivada.** En el cierre booleano, un evento ya conocido
no vuelve a la frontera. En el tropical, una adición puede **mejorar** el costo
de un evento viejo al abrir una prueba más corta. La acción de cambio pasa a
ser \(c\mapsto\min(c,\Delta c)\), monótona en el orden invertido de costos,
y la frontera debe incluir los eventos cuyo costo bajó, no sólo los nuevos.
Ésa es la diferencia de diseño respecto de los evaluadores incrementales,
que hoy registran una prueba nueva de un evento viejo sin volver a programarlo
(comentario en `program_incremental.rs`, `retain`; mismo comportamiento en
`IncrementalClosure::update`).

**Conservar.** El cierre de eventos y el conjunto de justificaciones no
cambian; el costo es una anotación adicional calculada sobre ellos. Puede
calcularse en una pasada posterior sobre las justificaciones ya retenidas, sin
tocar el cierre forward, como primer paso de bajo riesgo.

**Verificar.** Comparar contra la enumeración exhaustiva de pruebas en las
fixtures pequeñas (`forward_provenance.res` tiene dos caminos a `shared`) y
contra un Bellman-Ford de referencia sobre el hipergrafo.

## 10. Uniones *worst-case optimal*

Para cuerpos con variables compartidas en ciclo (`e(x,y) & e(y,z) & e(z,x)`),
los joins binarios pueden producir intermedios mayores que el resultado.
Algoritmos como Leapfrog Triejoin o Generic Join tienen cota óptima en el peor
caso. Requieren índices ordenados por variable; dependen del punto 1. Sólo vale
la pena si aparecen reglas de ese tipo en las ontologías reales.

## 11. Evaluación dirigida por objetivo

El cierre forward calcula todas las consecuencias. Cuando sólo interesa un goal,
transformaciones como *magic sets* restringen la evaluación a lo relevante para
él. La búsqueda residual ya es backward; habría que delimitar qué fragmento del
forward se beneficia sin duplicar su contrato.

## Fuera de este inventario

Las eliminaciones y reemplazos (DRed, conteo de soportes) son una capacidad
nueva, no una optimización; siguen pendientes según el §4 de
[discrete-derivative.md](discrete-derivative.md).
