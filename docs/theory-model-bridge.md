# Puente entre evidencia, teorías y modelos

Experimento de 2026-10-01, posterior a las cuatro correcciones de scope/tipos.
Sólo fixtures, tests y documentación; no cambia TypedAST, matcher, close,
runtime, búsqueda residual, asociaciones de verbos ni incrementalidad.

**Resultado central:** en las familias examinadas, Horn preserva la clase de
modelos de la teoría al saturar sus hechos. No necesita ser el mínimo de los
modelos FOUR para hacerlo. La interpretación de fórmulas generales conserva
esa clase de modelos, no sólo una valoración del closure o del modelo mínimo.

Los programas están en [TheoryBridge](../examples/experiments/theory_bridge.res)
y [TheoryBridgeTensor](../examples/experiments/theory_bridge_tensor.res).
Los seis contextos son premises de experimentos semánticos, no nuevos seeds
Derived del lenguaje. El [harness](../src/algebra/tests/evidence_pairs/satisfaction_bridge/model_entailment/theory_bridge.rs)
usa el parser/elaborador y close existentes, con las valoraciones FOUR anteriores.

## 1. Objetos y dos lecturas de una teoría

Fijamos una firma finita Σ de átomos ground, con sus dos polaridades y tiempos
constantes. Sea E el conjunto de literales con signo:

\[
K=\mathcal P(E),\qquad W=FOUR^{Atoms},\qquad \Gamma=(S,R).
\]

Cada mundo w se representa por el conjunto de soportes positivos/negativos de
sus átomos. Recorrer todos esos conjuntos enumera todas las valoraciones FOUR;
no obliga a que el mundo sea el conocimiento realmente disponible S.

Comparamos dos políticas D de satisfacción:

| | Soporte Horn, H | FOUR-fusión residuada, F |
|---|---|---|
| Hecho con signo P | P pertenece al soporte de w | I ≤_T v_w(P), equivalente a soporte de ese signo |
| I | Satisfecho sin soporte adicional | v_w(I)=B; B ≤_T B |
| Producto A & B en el body | Soporte de ambos factores | v_w(A) ⊗ v_w(B), con fusión FOUR |
| Regla A <= C | Si A tiene soporte, C tiene soporte | v_w(A) ≤_T v_w(C) |

El mismo filtro {T,B} para hechos no hace iguales las dos condiciones sobre
reglas. La lectura H no reclama implementar un residual algebraico universal.
La lectura F conserva la desigualdad completa y el filtro de su unidad.

Para cada política:

\[
Mod_D(S,R)=\{w\in W\mid w\models_D S\text{ y }w\models_D R\}.
\]

Este conjunto se calcula evaluando las reglas en **todas** las interpretaciones;
no se filtra a partir del resultado de close. Para F:

\[
\Gamma\models_F\varphi\iff
\forall w\in Mod_F(\Gamma),\ I\le_Tv_w(\varphi).
\]

En el fragmento evaluado, el mundo con ambos soportes de todos los átomos siempre
satisface cualquier hecho/regla. No hay teorías sin modelos ni entailment vacuo
producido por contradicciones. La clase vacía sigue siendo un objeto válido de
las operaciones matemáticas entre clases de modelos, no un hecho del programa.

## 2. El puente de teorías preserva significado al cerrar evidencia

**THEOREM condicional, fragmento ground.** Si las reglas tienen cuerpos finitos
Unit/Event/And y un literal como head, las inferencias Horn explícitas son sound
para las dos lecturas anteriores. Para H esto es su definición. Para F, el soporte
positivo de un producto es la conjunción de los soportes positivos; una
desigualdad de verdad obliga a que el head tenga ese soporte. Por inducción,
cada paso forward está satisfecho por todos los modelos iniciales.

Como S está contenido en su cierre y todos los hechos derivados ya son
consecuencias de la teoría:

\[
\boxed{Mod_D(S,R)=Mod_D(Close_R(S),R).}
\]

El argumento es sobre el cierre lógico abstracto con matching correcto. No es
una prueba formal del código Rust, de los programas parametrizados generales
ni de su terminación.

**MODEL RESULT:** la igualdad se verificó para las 4.016 teorías de la familia
de dos átomos en cada lectura, con cero discrepancias. Incluir el closure como
premisas conserva la teoría; no afirma que su única valoración sea un modelo.

Para R fijo, añadir evidencia también restringe modelos:

\[
S\subseteq T\Rightarrow Mod_D(T,R)\subseteq Mod_D(S,R).
\]

Por ello, la consecuencia universal de una fórmula fija es monótona al añadir
hechos, aunque su **valoración en un único mundo** no sea monótona en knowledge
order. Son cuantificaciones distintas. También, por la definición de hechos:

\[
Mod_D(S\cup\Delta S,R)=Mod_D(S,R)\cap Mod_D(\Delta S,R).
\]

Estas igualdades no introducen una derivada general ni identifican intersección
de modelos con tensor proposicional.

## 3. Literales comunes, mínimos y closure operacional

Definimos el núcleo atómico de una teoría:

\[
\gamma_D(S)=Common(Mod_D(S,R))
=\bigcap_{w\in Mod_D(S,R)}Support(w).
\]

Contiene exactamente los literales satisfechos por todos los modelos. No almacena
fórmulas arbitrarias, provenance ni DerivationTime. Todo requirement/hipótesis
del motor sigue separado del conocimiento real.

Para H, los modelos de R son conjuntos cerrados bajo sus implicaciones de
soporte. Su intersección es cerrada, y:

\[
\gamma_H(S)=Close_R(S).
\]

Para F, en la familia probada también son cerrados por intersecciones de
conocimiento: Common es un modelo y el menor por inclusión. Sin embargo:

\[
Close_R(S)\subseteq\gamma_F(S),
\]

y la inclusión puede ser estricta. Se verificó además
`Mod_D(γ_D(S),R)=Mod_D(S,R)`; para interpretar la teoría sigue siendo necesario
conservar R y la política D.

| Lectura | Teorías | Cambios de modelos al cerrar S | Horn distinto del modelo mínimo | Literales adicionales del mínimo |
|---|---:|---:|---:|---:|
| Soporte H | 4.016 | 0 | 0 | 0 |
| FOUR-fusión F | 4.016 | 0 | 640 | 680 |

La familia comprende 16 evidencias sobre P/Q y 251 programas: vacío, cada una
de 60 reglas unidad/unarias/binarias con signo, y los 190 pares distintos de
las 20 reglas unidad/unarias. Incluye ciclos, repetición y oposición. No es
exhaustividad sobre todos los programas binarios ni todos los tamaños de Σ.

### Por qué FOUR añade restricciones

Para A ⊗ B <= C, la desigualdad de fusión equivale, en componentes, a:

\[
\begin{aligned}
p_A\land p_B&\Rightarrow p_C,\\
n_C\land p_B&\Rightarrow n_A,\\
n_C\land p_A&\Rightarrow n_B.
\end{aligned}
\]

La primera es el paso Horn explícito. Las otras dos no están instaladas en el
motor. Para un único factor aparecen `P <= Q` y su restricción negativa
`~Q <= ~P`. Para cero factores, I sólo exige el soporte del head.

**THEOREM condicional, fusión elegida:** para un producto finito de literales,
su componente positiva es la conjunción de las positivas; su componente
negativa es la conjunción, para cada factor, de «todos los demás soportados
implica soporte negativo de ese factor». Se obtiene por inducción de la fórmula
binaria de fusión. Así la desigualdad se describe por el paso forward y una
restricción rotada por factor. Esto explica una familia de modelos cerrada por
intersecciones; no depende de negación por ausencia.

**MODEL RESULT:** sólo el harness materializa esas restricciones como cláusulas
ground de comparación. En los 251 programas, sus modelos de soporte son
exactamente los modelos FOUR-fusión de las reglas originales; su cierre coincide
con γ_F para cada evidencia. La enumeración de modelos es el oráculo independiente.
**No se propone ni se implementa contraposition automática en M1.**

## 4. Adjunción entre evidencia y clases de modelos

Con R y D fijos, sea W_R el conjunto de sus modelos. Para cualquier clase
X ⊆ W_R:

\[
X\subseteq Mod_D(S,R)
\iff S\subseteq Common(X).
\]

**THEOREM:** los dos lados dicen que todos los mundos de X soportan todos los
hechos de S. Si las clases se ordenan por **inclusión inversa**, se escribe:

\[
Mod_D:K\longrightarrow\mathcal P(W_R)^{op},\qquad
Mod_D\dashv Common.
\]

Common de la clase vacía es top_K, el conjunto de todos los literales. El compuesto
γ_D es extensivo, monótono e idempotente por esta adjunción. En general puede no
ser él mismo un modelo; esa propiedad adicional requiere cierre de la clase
de modelos por intersecciones, comprobado y explicado arriba para este fragmento.

**MODEL RESULT:** 2.170.880 pares evidencia/clase comprobados. Se enumeraron todas
las clases para los fondos vacío y `P <= Q`, en H y F: respectivamente 16/16/12/9
mundos y 65.536/65.536/4.096/512 clases. No se muestrearon clases.

La inclusión inversa usada en esa adjunción no se renombra ≤_L. El orden de
teorías por implicación, `Γ implica Δ` cuando `Mod(Γ) ⊆ Mod(Δ)`, usa inclusión
directa y hace antítono el mapa desde evidencia. El orden de fórmulas FOUR que
sigue es un tercer objeto. No se identifican por conveniencia de notación.

## 5. Fórmulas en una misma clase de modelos

Para Γ y F fijos, interpretamos una fórmula por la función:

\[
\llbracket A\rrbracket_\Gamma:Mod_F(\Gamma)\to FOUR,
\qquad w\mapsto v_w(A).
\]

El carrier de funciones FOUR^Mod tiene operaciones y truth order punto a punto.
Las denotaciones de fórmulas son una subálgebra cerrada por los operadores que
se admitan; se consideran módulo igualdad en todos los modelos de Γ. Su orden
candidato es:

\[
A\le_\Gamma B\iff
\forall w\in Mod_F(\Gamma),\ v_w(A)\le_Tv_w(B).
\]

**THEOREM condicional:** una realización residuada puntual conserva las leyes de
su carrier FOUR, incluida:

\[
A\otimes B\le_\Gamma C
\iff A\le_\Gamma B\multimap C.
\]

Con el filtro de I, en esa misma clase:

\[
\Gamma\models_F(A\otimes B)\multimap C
\iff A\otimes B\le_\Gamma C.
\]

**MODEL RESULT:** 1.592 triples de fórmulas sobre los seis contextos de las
fixtures. Se comprobaron ambas direcciones, equivalencia con el entailment de
la fórmula residual y bounds universales de join/meet. Los operandos de esos
triples son literales con ambos signos, I y bottom lógico; se construyen
productos, joins, meets y residuales en el modelo de tests. No se añadió esa
gramática al lenguaje ni se enumeraron todas las fórmulas posibles.

### El modelo mínimo tampoco reemplaza a la teoría

Con Γ=(∅,{P <= Q}), hay nueve modelos FOUR y el mínimo tiene P=Q=U. En ese
único mundo:

\[
v_{min}(Q\multimap P)=U\multimap U=T.
\]

Pero P=U, Q=T también es un modelo de Γ; allí Q ⊸ P vale U y no es designado.
Por tanto Γ **no** entraña Q ⊸ P. Esto impide evaluar toda la lógica mirando
sólo γ_F(S), incluso cuando es el modelo mínimo y recupera exactamente el
entailment de literales. El valor puntual no conserva por sí solo la
cuantificación universal sobre modelos.

## 6. Resultados observables de las fixtures

Los siguientes son **EXPERIMENTAL OBSERVATION** de archivos parseados/elaborados
y **MODEL RESULT** de la enumeración de sus interpretaciones:

| Regla / evidencia | Horn | Mínimo FOUR | Modelos FOUR | Valoración Horn es modelo |
|---|---|---|---:|---|
| P <= Q / ∅ | ∅ | ∅ | 9 | Sí |
| P <= Q / P | P,Q | P,Q | 3 | Sí |
| P <= Q / ~Q | ~Q | ~P,~Q | 3 | No |
| P <= Q / P,~P | P,~P,Q | P,~P,Q | 2 | Sí |
| P <= Q / P,Q,~Q | P,Q,~Q | P,~P,Q,~Q | 1 | No |
| A & B <= C / A,B,~C | A,B,C,~C | A,~A,B,~B,C,~C | 1 | No |

La última fila reproduce C=B en FOUR sin clasificarlo como fallo de soundness.
Horn mantiene sus consecuencias explícitas; todos los modelos soportan C,
aunque la valoración del resultado Horn no satisfaga la desigualdad completa.
Los hechos de entrada no se modifican con los literales del núcleo semántico.

## 7. Qué queda establecido y qué no

Tenemos un puente experimental a nivel de teoría:

```text
S junto con R y una política D
           ↓
        Mod_D(S,R)
           ├─ Common → núcleo de literales / comparación con Horn
           └─ valoraciones de fórmulas → orden y entailment universal
```

Cerrar S por Horn conserva ese significado de teoría en el fragmento. Esto
permite mantener M1 como inferencia por reglas explícitas y estudiar en otra
capa la interpretación algebraica completa, sin corregir el primero para hacer
que compute todo γ_F. La preservación de modelos tiene un argumento condicional
general y evidencia finita; los conteos de mínimos y adjunciones pertenecen a
los dominios declarados.

No elegimos FOUR como semántica definitiva de M2, no identificamos su orden
con el operacional de programas, no implementamos un decisor general de L ni
extendemos los resultados a asociaciones arbitrarias o tiempos generados sin
cota. Programas asociados y su composición siguen en el núcleo operativo
existente, con su alcance previamente documentado.

Antes de investigar completitud abductiva habrá que declarar si el objetivo es
alcanzar una consecuencia Horn o una consecuencia de todos los modelos F:
son relaciones distintas. La derivada general queda para el siguiente trabajo;
este experimento no modifica DClose. La
[caracterización aditiva posterior](discrete-derivative.md) y la
[semántica operacional consolidada](monotonic-core-specification.md) registran
el estado siguiente. Defaults, ! y preferencias no intervienen.

## 8. Verificación limitada

Tres tests individuales, todos aprobados mediante test_safe.py; no se ejecutó
la suite completa. Tras añadir el contraejemplo del modelo mínimo y la
preservación al saturar con γ, se repitieron sólo los dos tests modificados.
El fmt-check inicialmente señaló una línea larga del nuevo test; se corrigió
esa línea y el check posterior pasó.

La compilación inicial consumió 502,71 MiB RSS; la posterior, 368,40 MiB.
La enumeración más larga tardó 2,91 s y consumió 15,24 MiB RSS. Los otros tests
tardaron aproximadamente 0,062 s / 0,041 s y unos 15,4 MiB. Se mantuvieron
1 GiB total, swap cero, un job, un hilo y timeouts; no hubo OOM ni incremento
de límites.

Registros: [primera ejecución](../target/test-diagnostics/20261001T075651354398Z-81257/report.md)
y [verificación final de los dos tests ampliados](../target/test-diagnostics/20261001T075945629709Z-83478/report.md).
Los nueve tests CNL siguen ignored.
