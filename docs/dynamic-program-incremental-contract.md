# Contrato experimental de incrementalidad con Program activados

**Seguimiento actual:** el scheduler experimental ya está integrado en
`close_program`; véase [integración de Program](program-semi-naive-integration.md).
Las secciones siguientes conservan el contrato y las observaciones anteriores
a esa integración.

**Estado posterior:** la discrepancia de bindings descrita en las secciones
7–9 queda corregida por [sustituciones tipadas canónicas](canonical-typed-substitutions.md).
Se conservan aquí los resultados históricos que motivaron la reparación.
Ahora pasan las 448 comparaciones y el producto pequeño coincide en 223
eventos y 244 justificaciones; el contraejemplo mínimo conserva una sola
justificación independientemente del pivote. La reparación se realiza al
almacenar bindings en el motor, no al comparar records. El scheduler de
Program semi-naive sigue sin integrarse en producción.

## Estado del experimento antes de la reparación canónica

Rama `experiment-peano-after`, sobre el checkpoint aritmético `a7822ef` y el
cierre indexado del experimento anterior. `main` y `v1` permanecen intactos.
Este incremento añade documentación, fixtures y verificadores. No modifica
`close`, `close_program`, el matcher, `IncrementalClosure`, Rule, Program,
TypedAST, el parser, la deduplicación ni el runtime.

**Resultado:** el contrato separa eventos, justificaciones y activaciones.
El adaptador semi-naive satisface la equivalencia en 448 comparaciones de un
universo finito, pero falla la equivalencia de provenance en el producto
polinómico. Hay un contraejemplo mínimo temporal. No se integra todavía como
scheduler de producción ni se declara una mejora de rendimiento equivalente.

## 1. Estado y cambios distintos

Para un módulo fijo Π, con imports resueltos y sin defaults, consideramos:

\[
\Sigma=(K,J,A).
\]

- K: eventos conocidos, con la identidad lógica existente
  `(verb, arguments, propositionTime, polarity)`.
- J: justificaciones aceptadas, identificadas por
  `(Event, DerivationRecord)`, antes del sellado. Conserva rule, testigos en orden
  y sustitución; no es un conjunto de árboles de pruebas desplegados.
- A: instancias activadas de RuleUse, con referencia al Program/Rule y su origen,
  trigger conocido y entorno de bindings. Si dos verbos referencian el mismo
  Program, sus triggers siguen siendo distintos contextos de activación.

Los cambios canónicos se distinguen:

\[
\Delta Events=K'\setminus K,\quad
\Delta Derivations=J'\setminus J,\quad
\Delta Activations=A'\setminus A.
\]

No se exige que la conclusión de una justificación nueva sea un evento nuevo:

\[
events(\Delta Derivations)\not\subseteq\Delta Events
\quad\text{en general}.
\]

`derivedAt` se asigna una vez por justificación aceptada, con el reloj externo.
Las justificaciones anteriores conservan sus sellos. La recomputación
independiente puede descubrirlas en otro orden y darles otros valores concretos
de `derivedAt`; eso no cambia la identidad de la justificación.

## 2. Qué activa un Program

Para un trigger positivo conocido de un Verb Seeded o Derived con `program`,
se resuelve la asociación existente, se compilan sus RuleUses y se obtienen los
bindings compartidos a partir de los argumentos y el índice temporal del trigger.
Los parámetros locales permanecen locales. Captures, constantes, constructores,
imports y origen de Rule/Program mantienen la interpretación existente.

Una activación se identifica por su referencia de origen/miembro, el trigger y
su entorno, no sólo por el nombre de la regla. No se reactiva por recibir una
segunda justificación del mismo trigger lógico. EvidentialNot no activa el
programa positivo. Declarar/importar un Verb no constituye una instancia conocida.
Effect sigue fuera de esta activación; no se ejecuta State automáticamente.

ΔActivations también es trabajo pendiente durante la inicialización o después
de consumir una frontera de eventos. Por eso `DeltaEvents = empty` no autoriza
omitir una regla recién activada. No se está implementando modificación de
imports o asociaciones en caliente: Π permanece fijo.

## 3. Contrato de evaluación semi-naive

Las reglas ordinarias y las instancias activadas usan el mismo cierre conjunto.

1. **Inicialización:** evaluar una vez los axiomas ordinarios; descubrir las
   activaciones correspondientes al Known inicial.
2. **Nueva activación:** evaluar cada instancia nueva contra todo el Known
   disponible una vez. Sus testigos pueden ser antiguos; su cuerpo puede ser I.
3. **Reglas ya inicializadas:** buscar matches con al menos un testigo de evento
   de la frontera nueva. El resto se une contra Known con bindings compatibles.
4. **Aceptación:** reconstruir la misma justificación que produciría el matcher
   original para ese testigo y contexto. Conservarla si no estaba en J y sellarla
   una vez. Si su conclusión no estaba en K, insertarla y programar ese evento.
5. **Propagación:** los eventos nuevos pueden activar otros Programs. Una
   justificación nueva de un evento antiguo entra en ΔDerivations, aunque no
   añada ese evento a la frontera lógica.
6. **Parada:** vaciar tanto la frontera de eventos como las activaciones pendientes,
   después de aceptar todas las justificaciones habilitadas por esos cambios.

Un cuerpo I no convierte conocimiento vacío en un evento. Un RuleUse con I se
inicializa al activarse su Program, conservando el trigger como evidencia, como
ya hace `instantiate_program`.

El contrato permite otro orden de descubrimiento, no perder alternativas ni
añadir variantes de una misma justificación por elegir otro pivote. No cambia
Eq de Event, DerivationRecord ni SubstitutionValue, y no introduce hash dedup.

## 4. Por qué una prueba nueva no obliga a repetir matching downstream

Actualmente DerivationRecord contiene `antecedents: Vec<Event>`. No contiene
identificadores de las derivaciones de esos antecedentes; las reglas no consultan
provenance como condición de matching.

Si `shared` recibe una segunda justificación, una regla `shared <= finished`
con el mismo testigo y sustitución no obtiene por ello un DerivationRecord nuevo.
La derivación de `finished` sigue apuntando al evento `shared`, y ahora J conserva
dos explicaciones de ese evento. Un recorrido de provenance puede ver ambas
sin duplicar la regla downstream.

Esto es específico de la semántica existente. No afirma que los cambios de
provenance sean irrelevantes para cualquier lenguaje futuro. Si las reglas
consultaran identidades de pruebas, ΔDerivations requeriría otra propagación.

## 5. Relación con la derivada discreta

Para una base completamente cerrada B bajo Π y adiciones ΔS, el contrato lógico
continúa siendo:

\[
Close_\Pi(S\cup\Delta S)=B\cup\Delta Events.
\]

Ahora el operador de consecuencia incluye la activación determinada por Known:

\[
H_\Pi(K)=S\cup K\cup
events\bigl(Matches(R_0\cup Activated_\Pi(K),K)\bigr).
\]

R₀ son las reglas ordinarias; las definiciones internas de los Programs no
participan como reglas obligatorias independientes. Matches conserva contexto
de activación y produce UnstampedDerivations. Para la provenance exigimos además
que J final contenga todas las justificaciones distintas habilitadas por K final.

La nueva obligación, respecto de `docs/discrete-derivative.md`, es considerar
ΔActivations junto con ΔEvents. Un match nuevo se habilita porque cambió alguno
de sus testigos o porque su instancia de regla acaba de inicializarse. Los
matches de reglas antiguas con todos sus testigos antiguos ya fueron procesados.

**Argumento condicional:** si el matcher conserva la misma identidad de
justificación, cada match habilitado se procesa y la instancia llega a un cierre
finito, la evaluación por estas fronteras debe alcanzar los mismos eventos y
justificaciones que la recomputación. Este argumento no demuestra el código;
la discrepancia temporal abajo muestra que su hipótesis de identidad todavía
no se cumple en el adaptador investigado.

## 6. Harness, sin cambiar el cierre productivo

Fuentes:

- `examples/experiments/dynamic_incremental_programs.res`: trigger Seeded,
  Program parcialmente aplicado, record con constante capturada y `after(t)`,
  activación Derived recursiva, evidencia antigua/tardía y soporte alternativo.
- `examples/experiments/dynamic_incremental_temporal_boundary.res`: contraejemplo
  mínimo de sustitución temporal.
- El producto reutiliza `peano_polynomial_product_scenario.res` sin modificarlo.
- `src/bin/residuum/cli/dynamic_incremental_contract_tests.rs`: runner/adaptador.

El adaptador recorre Verb.program del AST, compila cada Program existente y
antepone un guard de activación a cada RuleUse usando sus parámetros higiénicos:

```text
trigger(parameters) & original_body <= original_consequence
```

Después reutiliza IncrementalClosure sobre esas reglas guardadas fijas. Un
trigger nuevo actúa como pivote y puede combinarse con todo el soporte antiguo.
Las reglas ordinarias siguen disponibles; las definiciones internas sin guard
se excluyen igual que en close_program. No hay una tabla Rust de nombres de
verbos ni nueva sintaxis del lenguaje.

Este es un modelo de la activación, no una API productiva de ΔActivations ni
un scheduler que mantenga instancias dinámicas en close_program. El caso de
inicialización con ΔEvents vacío se comprueba aparte sobre una regla ligada a
un trigger ya presente en Known; no se cambia el conjunto fijo de reglas de
IncrementalClosure mediante una API inexistente.

El guard puede duplicar un trigger que ya figura en el cuerpo. La proyección
observacional elimina exclusivamente ese guard adicional cuando el testigo
original ya lo incluye, reproduciendo la convención de instantiate_program.
Conserva las repeticiones originales del cuerpo y sus substitutions. No fusiona
justificaciones ni normaliza valores Term/Time. La comparación exacta excluye
sólo el orden de descubrimiento y los valores concretos de derivedAt.

## 7. Resultados y frontera encontrada

| Caso | Resultado |
|---|---|
| Trigger nuevo con soporte antiguo | Misma consecuencia y provenance |
| Trigger Derived activa otro Program | Misma cadena y unidades inicializadas |
| Cuerpo I del Program | Requiere trigger, conserva evidencia de activación |
| Parámetro capturado y constructor en record | Se conservan en las comparaciones finitas |
| Evidencia tardía para un Program ya activado | Se deriva la consecuencia pendiente |
| Soporte alternativo de un evento antiguo | Nuevo record conservado; no se repite downstream |
| Repetir las mismas seeds | Sin hechos, justificaciones ni sellos nuevos |
| Nueva instancia pendiente, ΔEvents vacío | Genera una justificación nueva de shared ya conocido |
| Trigger negativo | No activa el programa positivo |
| 448 comparaciones de subconjuntos/actualizaciones | Igualdad de eventos y provenance exacta |
| Producto pequeño importado y parcialmente aplicado | Eventos iguales; provenance **no equivalente** |

Las 448 comparaciones enumeran los 64 subconjuntos de seis hechos y cada elección
de una actualización singleton, además de cada base. No son una prueba universal.

En el producto, el oráculo devuelve 223 eventos y 244 justificaciones. El
adaptador devuelve los mismos 223 eventos, pero 250 justificaciones. Hay 24
records exclusivos del adaptador y 18 exclusivos del oráculo. Sus productores,
eventos, testigos y valores simbólicos de bindings corresponden; difieren en la
variante almacenada de algunos bindings temporales.

El caso mínimo declara:

```text
seeded verb run() = program Timed
seeded verb evidence(value : TimeBox)
rule copyShared(t, observed) =
    evidence(TimeBox { value = t, }) @ observed <= copied() @ t
program Timed(t : PropositionTime) { use copyShared { t = t } }
```

Con `run() @ tau` antiguo y `evidence(TimeBox { value = tau }) @ tau` nuevo:

| Motor | Binding de #Timed:shared:t |
|---|---|
| close_program / cierre completo guardado | `Time(At("tau"))` |
| Actualización con pivote en evidence | `Term(Const("tau"))` |

Ambos resultados tienen tres eventos y una justificación, pero ésta no es
idéntica. Si trigger y evidencia se añaden juntos, ambos órdenes de pivote
producen dos records en lugar de uno. La comparación del cierre completo de
las mismas reglas guardadas sí coincide con close_program en este caso: la
discrepancia queda aislada al binding incremental, no a una nueva política de
activación o a una diferencia de valores temporales.

`bind` guarda el primer SubstitutionValue compatible y no lo reemplaza. El
matcher reconoce la correspondencia estructural Term/Time, pero Eq del record
conserva la variante almacenada. Poner el pivote antes del orden original puede
cambiar cuál representación queda primero. La nueva evidencia no justifica
inventar ese segundo record.

**Clasificación:** incumplimiento de equivalencia de sustitución/provenance del
matcher incremental en este fragmento temporal experimental. No es pérdida de
resultados lógicos, ni una política sobre P/~P, ni un problema que un índice/hash
de deduplicación deba resolver. Los modelos finitos sin ese transporte mixto no
lo habían detectado.

## 8. Decisión antes de integrar

El contrato exige que la sustitución registrada sea independiente del pivote.
Para mantener la identidad actual, una vía a investigar es reconstruir los
bindings del testigo en el orden original, precargando el entorno de activación,
antes de emitir/deduplicar la justificación. Eso preservaría la representación
que usa close_program sin redefinir Eq globalmente.

No se implementa aquí esa reparación, no se ignoran los bindings al comparar
y no se fusionan los records extra. Los tests de frontera afirman expresamente
la **no equivalencia** y caracterizan su causa; que esos verificadores pasen
no significa que el adaptador satisfaga el contrato para polinomios.

Hasta corregir y verificar esa obligación, no se conecta esta evaluación a
close_program, no se intenta el producto mayor y no se afirma una aceleración
semi-naive equivalente. También quedan fuera eliminaciones, cambios de módulos,
defaults, negación por ausencia, residual incremental y ejecución de State.

## 9. Verificación limitada

Se ejecutaron cinco verificadores del contrato y una regresión existente de
provenance, individualmente mediante `scripts/test_safe.py`. Se mantuvieron
1 GiB total, sin swap, un job de Cargo, un hilo del harness y 15 segundos por
test. No se ejecutó la suite completa. Los verificadores finales pasaron;
dos de ellos confirman contraejemplos, no equivalencia exitosa.

La primera fixture mezclaba premises con query residual; el elaborador la
rechazó. Se corrigió a la combinación existente premises/fourFusion, sin cambiar
gramática. El campo de consulta de ese snapshot sólo sirve como observación
bien tipada; este harness no ejecuta FOUR ni residual.

La primera comprobación del producto falló la comparación exacta. Se conservó
el resultado y se aisló como test de frontera; no se relajó la definición de
equivalencia. Los diagnósticos locales guardan ese fallo inicial.

Informes:

- `target/test-diagnostics/20261009T050958060331Z-3901081`: rechazo inicial de
  la fixture; compilación con 378.14 MiB pico RSS.
- `target/test-diagnostics/20261009T051128611442Z-3902359`: dos casos básicos.
- `target/test-diagnostics/20261009T051223606952Z-3903362`: 448 comparaciones
  exitosas y fallo original de provenance en el producto.
- `target/test-diagnostics/20261009T051721234020Z-3908556`: cinco verificadores
  finales, incluido el contraejemplo; producto 1.8723 s y 23.79 MiB pico RSS.
- `target/test-diagnostics/20261009T053911174870Z-3923854`: formato final.
- `target/test-diagnostics/20261009T053911819044Z-3923872`: aislamiento adicional
  mediante cierre completo guardado y regresión existente de provenance.

Cambios sin commit. El evaluador productivo sigue siendo el cierre indexado
del experimento anterior.
