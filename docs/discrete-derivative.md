# Derivada discreta del cierre Horn

Caracterización de 2026-10-01. El objeto es el cierre por inferencia Horn explícita
de una teoría, no una valoración FOUR ni su entailment algebraico completo.
Este incremento añade una fixture, un harness de tests y documentación. No
modifica close, IncrementalClosure, TypedAST, residual, runtime o ProgramVerb.

## 1. Dominio y significado de un cambio

Sea E el universo de eventos lógicos bien tipados, con tiempo proposicional
resuelto y polaridad explícita. Trabajamos sobre:

\[
K=\mathcal P(E),\qquad X\le_KY\iff X\subseteq Y,
\qquad X\vee_KY=X\cup Y.
\]

Los cambios de evidencia son conjuntos de adiciones, con acción X + ΔX = X ∪ ΔX.
No se identifican con I del álgebra lógica. EvidentialNot es un evento con signo;
su adición no elimina el evento positivo.

Para un conjunto R de reglas positivas, T_R(X) contiene las conclusiones de
todas sus instancias cuyo antecedente tiene testigos compatibles en X. Unit
no requiere testigos; And requiere una sustitución compartida. Es un operador
monótono. Interpretamos las cláusulas residuales a través de su vista Horn
cuando el motor forward admite esa vista.

\[
Close_R(S)=\mu X.\;(S\cup X\cup T_R(X)).
\]

El operador lógico existe sobre el dominio completo. Los cuerpos finitos hacen
finitarios sus pasos: el cierre es la unión de sus aproximaciones finitas.
Eso no garantiza que el código alcance un punto de parada. Un programa que
genera after(t), after(after(t)), … puede tener cierre infinito.

Se distinguen desde ahora:

| Cambio | Caracterización | Implementación productiva |
|---|---|---|
| Adición de evidencia, R fijo | Derivada por fixpoint de delta y frontera | IncrementalClosure existente |
| Adición de evidencia y reglas | Derivada conjunta descrita abajo | Sólo harness ground; no nueva API |
| Eliminación/reemplazo de hechos o reglas | No cabe en una acción de unión positiva | Pendiente |
| Activaciones arbitrarias de Verb.program | Requieren considerar su estado de activación | No se extiende aquí |
| Defaults, !, residual incremental, ejecución de efectos | Otro alcance semántico | Fuera del experimento |

## 2. Caracterización que no depende de recomputar close

Sea B = Close_R(S) una base ya cerrada y ΔS una adición. Definimos sobre
\(\mathcal P(E\setminus B)\):

\[
\Phi_{B,R,\Delta S}(D)
=(\Delta S\setminus B)\cup(T_R(B\cup D)\setminus B),
\qquad
\partial_R(B,\Delta S)=\mu D.\;\Phi_{B,R,\Delta S}(D).
\]

La construcción comienza con D₀ = ∅, y aplica Φ hasta estabilizar o toma la
unión de sus aproximaciones. No llama a close sobre las seeds unidas ni define
su resultado mediante una resta entre dos recomputaciones. Resta B únicamente
para mantener el delta canónico disjunto de la base.

**THEOREM condicional, Horn positivo con matching correcto:**

\[
\boxed{Close_R(S\cup\Delta S)=B\cup\partial_R(B,\Delta S).}
\]

Argumento de suficiencia y minimalidad:

1. En el punto fijo, B ∪ D contiene S ∪ ΔS y todas sus consecuencias inmediatas.
   Es una extensión cerrada; por minimalidad del cierre, contiene Close_R(S ∪ ΔS).
2. Todo conjunto cerrado X que contiene B ∪ ΔS contiene cada aproximación de D:
   por inducción, T_R(B ∪ Dₙ) está contenido en X. Por tanto B ∪ D está contenido
   en la menor extensión cerrada.
3. El nuevo cierre contiene B por monotonicidad. Las inclusiones anteriores
   dan igualdad; D es precisamente la parte fuera de B.

Así obtenemos, como consecuencia de la caracterización:

\[
D_R(S,\Delta S)=\partial_R(Close_R(S),\Delta S)
=Close_R(S\cup\Delta S)\setminus Close_R(S).
\]

La última expresión sirve como oráculo extensional, no como algoritmo requerido.
El argumento prueba el operador abstracto, no formalmente el código Rust.

## 3. Evaluación por frontera

Como B está cerrado, cualquier match compuesto sólo de eventos de B concluye
un evento de B. Para obtener una conclusión fuera de B, algún testigo debe
pertenecer al delta. Ésa es la justificación del pivote de IncrementalClosure.

Una implementación por rondas mantiene Xₙ = B ∪ Dₙ y una frontera Fₙ de eventos
que entraron en la ronda anterior. Evalúa las instancias con todos sus testigos
en Xₙ y al menos uno en Fₙ. Deduplica las conclusiones y obtiene Fₙ₊₁; termina
cuando ésta es vacía. Cualquier match que acaba de habilitarse debe tocar la
frontera; los demás ya estaban habilitados en una ronda anterior.

Un axioma I <= C antiguo no toca ninguna frontera: su conclusión y su
justificación ya están en la base. Esto no equivale a convertir I en un evento.

Los algoritmos pueden usar schedules diferentes. La propiedad exigida es la
menor extensión cerrada, no un número particular de rondas ni un orden de
descubrimiento. DerivationTime sigue viniendo del reloj externo.

## 4. Leyes de la derivada y límites de la analogía diferencial

**THEOREM condicional:** para reglas fijas, con delta lógico canónico:

\[
\partial_R(B,\varnothing)=\varnothing,
\qquad
\Delta S\subseteq B\Rightarrow\partial_R(B,\Delta S)=\varnothing.
\]

Para B fijo, es monótona en la adición:

\[
\Delta S_1\subseteq\Delta S_2
\Rightarrow\partial_R(B,\Delta S_1)\subseteq\partial_R(B,\Delta S_2).
\]

También satisface una ley de composición secuencial. Si
D₁ = ∂_R(B, ΔS₁), entonces:

\[
\boxed{
\partial_R(B,\Delta S_1\cup\Delta S_2)
=D_1\cup\partial_R(B\cup D_1,\Delta S_2).
}
\]

Los dos deltas del lado derecho son disjuntos. El segundo se calcula respecto
de la nueva base cerrada. Es la ley de cociclo para esta acción de cambios,
derivada de la idempotencia y monotonicidad del cierre.

**Contraejemplo de aditividad en una misma base:** con P & ~P <= ~Q y B = ∅,
añadir sólo P da {P}; añadir sólo ~P da {~P}. Añadir ambos da {P,~P,~Q}. Así:

\[
\partial_R(B,\Delta S_1\cup\Delta S_2)
\ne\partial_R(B,\Delta S_1)\cup\partial_R(B,\Delta S_2)
\]

en general. La interacción conjuntiva aporta una consecuencia adicional. No
se está suponiendo linealidad ni independencia entre cambios.

Tampoco es monótona en la base como conjunto de eventos nuevos: para P <= Q,
añadir P a ∅ produce {P,Q}; añadirlo a la base cerrada {P,Q} produce ∅. El
resultado final sigue siendo monótono; el delta excluye lo que ya era conocido.

**Eliminación:** con seeds {P} y P <= Q, el cierre es {P,Q}. Retirar la seed P
deja cierre vacío. Ningún delta unido positivamente a {P,Q} puede producir ∅.
Una derivada para eliminaciones necesita otra acción de cambios y conservar
información sobre seeds y soportes. No se implementa ni diseña aquí.

## 5. Adiciones conjuntas de evidencia y reglas

La misma caracterización admite un cambio aditivo ΔR del programa, sin
redefinir reglas existentes. Para B cerrado bajo R:

\[
\Phi_{B,R,\Delta S,\Delta R}(D)
=(\Delta S\setminus B)\cup
\bigl((T_R(B\cup D)\cup T_{\Delta R}(B\cup D))\setminus B\bigr).
\]

**THEOREM condicional:** su menor punto fijo D cumple:

\[
Close_{R\cup\Delta R}(S\cup\Delta S)=B\cup D.
\]

El argumento de menor extensión cerrada es el mismo. La evaluación por
frontera tiene ahora una inicialización adicional: evaluar todas las reglas
nuevas sobre B ∪ ΔS, aunque sus testigos sean antiguos o su antecedente sea I.
Después, propagar mediante matches que toquen la frontera para todas las reglas.

Ejemplos que explican por qué no basta el pivote de seeds:

| Base cerrada | Regla nueva | Seeds nuevas | Delta |
|---|---|---|---|
| {P} | P <= Q | ∅ | {Q} |
| ∅ | I <= P | ∅ | {P} |
| {P} | P & ~P <= ~Q | {~P} | {~P,~Q} |

Esto generaliza el cambio matemático a adiciones de teoría. No modifica la
restricción de reglas privadas y fijas de IncrementalClosure. El evaluador de
esta variante existe sólo en el harness ground.

## 6. Delta lógico y delta de justificaciones

El evento y su derivación siguen teniendo identidades diferentes. Sea J_R(X)
el conjunto de instancias justificadas por X, representadas por el evento,
regla, sustitución y lista ordenada de testigos, sin sello temporal. Una base
operacional completa conserva J_R(B). El delta de justificaciones es:

\[
\Delta J=J_R(B\cup D)\setminus J_R(B).
\]

Puede haber ΔJ no vacío con una conclusión ya conocida. Filtrar justificaciones
por «head nuevo» perdería evidencia. La evaluación por pivotes encuentra los
nuevos testigos, pero debe registrar sus pruebas incluso si el head pertenece
a B. Varias posiciones pivot pueden descubrir la misma instancia; se registra
una vez. Una instancia adicional con la misma conclusión se conserva.

El harness compara este delta con los registros de update y del cierre
recalculado, preservando testigos, sustituciones y tiempos proposicionales.
También verifica que las derivaciones previas conservan sus sellos. No exige
igualdad de DerivationTime entre ejecuciones con relojes/schedules distintos.
La variante con adiciones de reglas se contrasta sólo en eventos lógicos;
no implementa mantenimiento de justificaciones para cambios de reglas.

## 7. Conexión con la semántica de teorías

Usando el [puente ya especificado](theory-model-bridge.md), la igualdad de cierres
es compatible con la conservación de modelos:

\[
Mod_D(S\cup\Delta S,R)=Mod_D(B\cup D_R(S,\Delta S),R).
\]

Es una consecuencia condicionada por soundness Horn de la política D, no una
identificación del closure con una valoración FOUR. La derivada actualiza el
cierre explícito; no calcula los literales adicionales del entailment FOUR ni
evalúa todas las fórmulas del álgebra. En particular, no resuelve la completitud
del residual por sí sola.

## 8. Experimento y verificación

[discrete_derivative.res](../examples/experiments/discrete_derivative.res)
declara P/Q, sus dos signos en un bloque de premises y cinco reglas ground:
un axioma, una cadena/ciclo, una conjunción y una consecuencia negativa. No
introduce sintaxis para cambios de reglas. El
[harness](../src/experiments/discrete_derivative_tests.rs) selecciona conjuntos
de estas reglas y adiciones de esas premises; el campo query sirve como
observación bien tipada y no se ejecuta residual ni FOUR en estos tests.

**MODEL RESULT / EXPERIMENTAL OBSERVATION:**

| Comprobación | Universo exhaustivo dentro de la familia | Casos |
|---|---|---:|
| Φ, frontera, update y cierre desde cero; registros y sellos antiguos | 32 conjuntos de reglas × 16 bases × 16 adiciones | 8.192 |
| ΔS y ΔR conjuntos; axiomas/reglas nuevas sobre hechos antiguos | 3 estados por cada una de 4 reglas × 16 bases × 16 adiciones | 20.736 |
| Composición secuencial y monotonicidad en la adición | 2 programas × 16 bases × 16 primeras × 16 segundas adiciones | 8.192 |

La primera comprobación enumera además todos los mundos cerrados de soporte
para cada conjunto de reglas y verifica que B ∪ D está contenido en cualquier
extensión cerrada que satisfaga B ∪ ΔS. Ese oráculo de minimalidad no usa el
matcher ni close. La evaluación ground de Φ y frontera tampoco los usa; close
y update son implementaciones independientes que se comparan después.

Se conservan explícitamente contraejemplos de aditividad, de monotonicidad en
la base y de representabilidad de eliminaciones por unión. Estos resultados
finitos no prueban exhaustividad sobre programas parametrizados, asociaciones
arbitrarias ni universos temporales infinitos.

Sólo se autorizaron los tres tests de este harness, mediante test_safe.py.
Se mantuvieron 1 GiB total, sin swap, un job, un hilo y timeouts; no se ejecutó
la suite completa. La primera ejecución se detuvo por comentarios // no
admitidos en la fixture; se retiraron sin cambiar la gramática. La compilación
alcanzó 503,36 MiB RSS. Tras corregir la fixture, los tres tests pasaron con
unos 15,5 MiB RSS y menos de un segundo cada uno. La comprobación adicional de
minimalidad pasó en una ejecución posterior del único test modificado:
0,5906 s y 15,34 MiB RSS; su compilación consumió 500,09 MiB RSS.

Registros: [los tres tests](../target/test-diagnostics/20261001T082336826196Z-100620/report.md)
y [el test ampliado de minimalidad](../target/test-diagnostics/20261001T082812026830Z-104205/report.md).

La deuda queda delimitada: tenemos una caracterización general del operador
Horn positivo para cambios aditivos y su mantenimiento productivo con R fijo.
La API conjunta, eliminaciones, activación incremental general de programas
asociados y cambios no monotónicos siguen fuera de esa garantía.
