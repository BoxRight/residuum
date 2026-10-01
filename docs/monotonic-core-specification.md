# Especificación del núcleo monotónico y su semántica operacional

Fecha: 2026-10-01. Núcleo operacional congelado para esta fase, con alcance
explícito. Esta consolidación incorpora la auditoría de scope/tipos, el puente
de teorías/modelos y la caracterización de la derivada discreta. No añade sintaxis,
fixtures, mecanismos de inferencia ni una semántica proposicional definitiva.

El contrato distingue el programa tipado Π, el operador de consecuencias que
determina, la entrada S y las consultas. El cierre forward, la actualización
incremental y la búsqueda residual tienen garantías y modalidades distintas.
«Congelado» significa que no hacen falta nuevas features para este fragmento;
no significa verificación formal del motor ni completitud de toda la lógica.

## 1. Estatuto de las afirmaciones

| Etiqueta | Significado | Alcance |
|---|---|---|
| **THEOREM** | Ley que el núcleo se compromete a garantizar, con hipótesis explícitas | Obligación de diseño; no certifica por sí sola el código Rust |
| **MODEL RESULT** | Resultado comprobado exhaustivamente en un modelo o familia finita identificada | Sólo ese dominio, interpretación y cuantificación |
| **EXPERIMENTAL OBSERVATION** | Comportamiento observado del motor o establecido por sus APIs y pruebas existentes | Fragmento implementado; no demuestra una ley universal |

Cada THEOREM indica si tiene justificación matemática condicional o si sigue
siendo una obligación de implementación. Las definiciones fijan el vocabulario;
no son resultados experimentales. Un test que comprueba todos los casos de un
carrier finito no demuestra que cualquier realización de M2 cumpla esas leyes.

La especificación separa cuatro relaciones:

| Relación | Objetos | Significado |
|---|---|---|
| \(<:\) | Tipos | Compatibilidad nominal/proposicional |
| \(\le_L\) | Fórmulas interpretadas | Orden lógico del álgebra objetivo |
| \(\le_K\) | Conjuntos de eventos | Inclusión de evidencia |
| \(\preceq_\Omega\) | Programas observados en un dominio finito | Inclusión de consecuencias para cada entrada |

No se identifica ninguna pareja de estas relaciones por notación o intuición.

## 2. ¿Qué es un programa?

Un programa monotónico tipado es, conceptualmente:

\[
\Pi=(\Sigma,R,\alpha).
\]

\(\Sigma\) contiene las declaraciones de entidades, campos, constantes y firmas
de verbos; \(R\), las reglas-lambda; \(\alpha\), las asociaciones opcionales de
verbos con referencias a reglas. Las entradas de conocimiento y las consultas
se suministran a ese programa; no son leyes de su álgebra. Un archivo puede
contener además bloques experimentales de seeds/premises y goals.

Una regla conserva sus parámetros tipados:

\[
r=\lambda\vec x.\;(A\le C).
\]

Una sustitución ground admisible instancia su cláusula. Una HornClause no es la
lambda completa; una aplicación parcial conserva los parámetros restantes.
Esta notación matemática no introduce un tipo ni una construcción de lenguaje.

**THEOREM T1 — especialización.** Una sustitución que respeta tipos y scope debe
conservar la estructura y validez de la cláusula instanciada: sustituye sus
átomos y tiempos, no afirma que el cuerpo ground sea idéntico al cuerpo abierto.
La aplicación parcial debe coincidir con su expansión inline, hasta equivalencias
explícitamente justificadas.
Es una obligación sobre elaboración y sustitución, no una consecuencia de
implementar un trait Rust.

**EXPERIMENTAL OBSERVATION E1.** `Rule { parameters, body }`, aplicación parcial y
β-reduction representan esa estructura. `Verb.program: Option<RuleRef>` referencia
una lambda cuya firma exige los argumentos del verbo, en orden, seguidos por
PropositionTime. No existen construcciones ProgramVerb, interface o Contract.
El pipeline activo es Formal → SurfaceAST → TypedAST → derivación/runtime;
CNL está conservado y pausado.

### Del programa al operador, no directamente al conocimiento

Para cada modalidad operacional admitida, la interpretación tiene forma:

\[
\mathcal O:\ Program\longrightarrow Mon(K,K),
\qquad \Pi\longmapsto T_\Pi,
\]

donde Mon(K,K) es el conjunto de funciones monótonas sobre conocimiento. T_Π
produce consecuencias inmediatas; puede no contener su propia entrada. La
ejecución completa añade una entrada S y toma el cierre de la sección 6.
La asociación \(\alpha\) forma parte del programa fijo, no de las seeds.

| Modalidad | Operador utilizado | API existente |
|---|---|---|
| Reglas explícitas R suministradas | T_R | close(seeds, rules, clock) |
| Programa con asociaciones activas | T_Π de la sección 7 | close_program(seeds, module, clock) |
| Adiciones de evidencia a una base cerrada, R fijo | Fixpoint de delta de la sección 12 | IncrementalClosure.update |
| Consulta de requisitos sobre declaraciones de reglas | Búsqueda y certificado, no un operador de cierre | query_program_residual |

La modalidad debe indicarse al comparar resultados. Una referencia a programa
asociado no convierte automáticamente las APIs de reglas explícitas en la
modalidad activa. β-reduction ocurre en elaboración: el motor recibe lambdas
tipadas ya especializadas, sin una operación de generación de reglas como hechos.

## 3. ¿Qué es una proposición?

Un evento ground es una instancia proposicional atómica bien tipada. Su clave es:

\[
key(p)=(verb(p),args(p),t_p(p),polarity(p)).
\]

La clasificación del verbo determina Seeded, Derived o Effect. La jerarquía es:

\[
Seeded(t)<:Prop(t),\qquad Effect(t)<:Derived(t)<:Prop(t).
\]

Seeded y Derived no son subtipos entre sí. No hay coerción entre índices temporales
distintos. La categoría se comprueba separadamente; no amplía la clave lógica.

Las fórmulas compuestas pertenecen al dominio lógico \(L\), que se pretende
cerrado bajo operaciones algebraicas. No toda fórmula es un Event: `I`, un
residual o una alternativa de requisitos no se insertan como eventos en Known.
Una fórmula puede reunir átomos de distintos tiempos sin imponer un tiempo
común a todos sus factores.

**THEOREM T2 — identidad e índices.** Todo evento admitido debe tener su
PropositionTime resuelto. Interpretaciones operacionales y registros de
derivación no deben alterar su identidad lógica. Una transformación que prometa
preservar metadata debe comprobarla además de comparar claves.

**EXPERIMENTAL OBSERVATION E2.** La igualdad/orden Rust de Event ignora tanto
`transition` como la categoría, conforme a esa clave. Los tipos se verifican en
elaboración o como precondición del caller de APIs internas. `after` conserva
estructura: `after(after(t))` no se normaliza a `after(t)`. No se ha declarado
que sea sucesor discreto, ni un orden temporal total.

## 4. ¿Qué es conocimiento?

Sea \(E_\Sigma\) el universo de claves ground bien tipadas, con ambas polaridades.
El dominio semántico de conocimiento es:

\[
K=\mathcal P(E_\Sigma),\quad
F\le_KG\iff F\subseteq G,\quad
F\vee_KG=F\cup G,\quad \bot_K=\varnothing.
\]

El motor almacena representaciones finitas de ese dominio. El mismo evento lógico
puede tener varias justificaciones, pero sólo ocupa una posición lógica en Known.
La representación operacional de un evento se conserva separadamente.

Para antecedente A, el puente implementado parcialmente es una relación con
testigos, no sólo un booleano:

\[
Match(F,A)=\{(\theta,w)\},
\]

donde θ es una sustitución compatible y w la lista ordenada de eventos utilizados.
El soporte \(F\models_{supp} A\) significa que existe ese testigo, en el scope
de parámetros de la regla. No se identifica este soporte con una desigualdad
puntual de valores FOUR ni con el entailment de todos los modelos de una teoría.

**EXPERIMENTAL OBSERVATION E3.** El matcher reconoce Unit, Event y And; no consume
Known y permite usar un mismo evento para varios factores. Conserva el orden y
la multiplicidad de los witnesses. No implementa satisfacción general para
meet, join o fórmulas residuales arbitrarias.

### Conocimiento operativo y teoría interpretada

Para reglas ordinarias fijamos la teoría Γ = (S,R). Una política semántica D
determina su clase de modelos Mod_D(S,R); no se identifica un conjunto Known
con un único modelo de Γ.

**THEOREM condicional — preservación de teoría.** En el fragmento ground
Unit/Event/And con heads literales, y para las políticas de soporte Horn y
FOUR-fusión descritas en el [puente de teorías/modelos](theory-model-bridge.md),
soundness de los pasos Horn implica:

\[
Mod_D(S,R)=Mod_D(Close_R(S),R).
\]

Cerrar hechos conserva significado de teoría. No implica que la valoración
puntual de Known sea un modelo FOUR ni que el cierre contenga todo entailment
algebraico. El argumento está condicionado al fragmento y a la política D;
no se extiende aquí a asociaciones arbitrarias.

**MODEL RESULT M8.** En 4.016 teorías por política se observaron cero cambios
de modelos al cerrar. Con soporte Horn, el cierre coincidió con el menor modelo;
con FOUR-fusión difirió en 640 teorías, con 680 literales adicionales del mínimo.
Incluso ese mínimo no decide fórmulas generales: con Γ = (∅,{P <= Q}) designa
Q -o P, pero un modelo P=U,Q=T refuta su entailment universal. La interpretación
general debe conservar la clase de modelos o una representación equivalente.

Esta realización finita proporciona un orden de fórmulas punto a punto sobre
una misma clase de modelos y conserva la residuación de FOUR. No elige la
semántica definitiva de L ni identifica ese orden con inclusión de Known.

## 5. ¿Qué significan `<=`, `&`, `I` y `-o`?

El objetivo algebraico sigue siendo:

\[
L=(L,\vee_L,\wedge_L,\otimes,I,\le_L,\multimap).
\]

Las representaciones de fórmulas pueden formar un preorden; su cociente por
\(A\simeq B\iff A\le_LB\land B\le_LA\) es el carrier ordenado. Para una teoría
fija, \(\le_L\) es el orden lógico de la realización que se adopte; el experimento
FOUR utiliza desigualdad de verdad en todos sus modelos. En una regla Formal, `A <= C`
declara una desigualdad dirigida; su compilación Horn permite producir C desde
evidencia compatible para A. No evalúa un booleano general `le(A,C)`.

Formal escribe el producto con `&`, cuya notación matemática es `⊗`; el lexer
no acepta `⊗` como alias literal. `I` denota su unidad. `-o` y `⊸` denotan el
residual derecho y sí son aliases aceptados. Meet no se identifica con producto;
join lógico no se identifica con unión de conocimiento.

**THEOREM T3 — contrato algebraico.** Toda realización declarada de L debe
satisfacer, hasta \(\simeq\):

\[
\begin{aligned}
&A\le_LA,\quad A\le_LB\land B\le_LC\Rightarrow A\le_LC;\\
&A\vee_LB\le_LC\iff A\le_LC\land B\le_LC;\\
&C\le_LA\wedge_LB\iff C\le_LA\land C\le_LB;\\
&(A\otimes B)\otimes C\simeq A\otimes(B\otimes C);\\
&I\otimes A\simeq A\simeq A\otimes I,\quad A\otimes B\simeq B\otimes A;\\
&A\otimes B\le_LC\iff A\le_LB\multimap C.
\end{aligned}
\]

Producto es monótono en ambos argumentos; residual es antítono en el primero y
monótono en el segundo. De la adjunción se deducen unidad/counidad, distribución
de producto sobre joins finitos y conservación de meets por el residual en su
resultado. Son consecuencias matemáticas condicionales de las hipótesis, no
pruebas de que el AST actual realice toda L. El
[contrato algebraico detallado](monotonic-core-laws.md) fija las demás leyes.

No se postulan integralidad \(A\le_LI\), idempotencia de producto, weakening,
contraction, explosión, existencia de un bottom lógico ni completitud de L.
En particular, \(A\otimes B\le_LA\) no es una ley adoptada.

**THEOREM T4 — unidad y contexto.** I es un requisito neutro, nunca ausencia de
conocimiento ni evidencia sobre un átomo. El producto de cero factores utilizados
es I, aunque el conocimiento disponible pueda ser no vacío. La regla ground
`I <= C` justifica C por ese axioma específico; I no implica cualquier C.

**EXPERIMENTAL OBSERVATION E4.** Unit produce un witness neutro sin nuevos
bindings o eventos. `residuate` separa la conjunción exterior y `unresiduate`
reconstruye la vista Horn; se conservan parámetros, agrupación, polaridad y tiempos.
El forward requiere ambos factores. El AST sólo representa un fragmento de
residuales, no todo el carrier algebraico. La reversibilidad estructural no
demuestra la caracterización universal del adjunto.

**MODEL RESULT M1.** Las leyes se comprobaron exhaustivamente en los modelos
Z/2Z-powerset, Łukasiewicz de tres valores y las realizaciones FOUR descritas
en [evidence-pair-models.md](evidence-pair-models.md). En FOUR con fusión, I=B;
con meet, I=T; conocimiento vacío valora cada átomo como U. Son modelos
distintos, no una elección definitiva de semántica de M2. La fusión completa
no es monótona en el orden de conocimiento de FOUR.

## 6. ¿Qué hace `close`?

Para R fijo, definimos el operador de consecuencias explícitas:

\[
T_R(F)=\{\theta C\mid r=(\lambda\vec x.A\le C)\in R,
\ (\theta,w)\in Match(F,A),\ \theta C\text{ ground}\}.
\]

El cierre lógico abstracto es:

\[
F_0=S,\quad F_{n+1}=F_n\cup T_R(F_n),\quad
Close_R(S)=\bigcup_{n<\omega}F_n.
\]

Para evitar confundir consecuencias inmediatas con cierre, definimos el
operador extensivo con entrada:

\[
H_{S,R}(F)=S\cup F\cup T_R(F),\qquad
Close_R(S)=\mu F.\;H_{S,R}(F).
\]

En la modalidad activa, se reemplaza T_R por T_Π y se obtiene Close_Π(S).
Por tanto «Close = μT» sólo es correcto si T nombra este operador con seeds
y acumulación, no la función de consecuencias inmediatas T_R. La API devuelve
además justificaciones y sellos; las ecuaciones describen su proyección Known.

**THEOREM T5 — least Horn closure.** Con matching positivo, reusable y
scope-correct, reglas fijas y cuerpos finitos, T_R es monótono. Close_R es el
menor conjunto cerrado bajo R que contiene S; es extensivo, monótono e
idempotente. Justificación condicional: los witnesses siguen disponibles al
añadir evidencia; cualquier cierre que contiene S contiene cada F_n; cada
prueba finita del límite ya usa evidencia de alguna ronda finita.

Esto caracteriza un objeto matemático incluso si no es finito. La ejecución
puede devolverlo en tiempo finito sólo cuando estabiliza. Monotonía no implica
terminación: una regla puede generar `t, after(t), after(after(t)), ...`.

**EXPERIMENTAL OBSERVATION E5.** `close(seeds,rules,clock)` itera hasta que no
aparecen nuevas claves, deduplica eventos y conserva justificaciones distintas.
Realiza una ronda inicial para axiomas ground con I. No ejecuta estados, defaults
ni inferencias algebraicas implícitas como contraposition.

**MODEL RESULT M2.** En la familia declarada de 4.016 teorías y 20.080 consultas,
Horn tuvo cero fallos de soundness respecto de FOUR-fusión/filtro de I, pero
680 fallos de completeness. Un contraejemplo es
\(\Gamma=\{\sim Q,P\le Q\}\): todos los modelos satisfacen \(\sim P\), pero
Horn no lo deriva. Los controles por soporte coincidieron en esa familia.
El [informe](four-model-entailment.md) distingue otras interpretaciones,
vacuidad y casos donde la valoración del closure no es un modelo. Ninguno de
estos conteos prueba soundness universal respecto de FOUR. La semántica
operacional elegida sigue siendo el least Horn closure, no todo entailment FOUR.

## 7. ¿Cómo actúa un programa asociado?

En el modo activo se separan las reglas ordinarias O y las definiciones
referenciadas por α. Una instancia positiva p conocida de un verbo Seeded o
Derived activa su definición mediante:

\[
\beta_p=(\vec x\mapsto args(p),\ t\mapsto t_p(p)).
\]

Su cuerpo debe satisfacer Match con esos bindings ya fijados. Conceptualmente:

\[
T_\Pi(F)=T_O(F)\cup
\{C\mid p\in F,\ p\text{ activa }\alpha(verb(p)),
\ C\text{ es su resultado ground justificado}\}.
\]

**THEOREM T6 — activación monotónica.** Para un programa fijo, añadir instancias
o witnesses no retira activaciones ni consecuencias. Con las mismas hipótesis
de matching de T5, T_Π es monótono y admite el cierre por la construcción de T5.
La declaración sola no justifica activación. La categoría elegible no cambia
la política de activación una vez conocida la instancia.

**EXPERIMENTAL OBSERVATION E6.** `close_program` realiza esa política. Excluye
las definiciones asociadas del pool ordinario; aliases explícitos siguen siendo
reglas ordinarias. Conserva llamadas activadas mientras llega evidencia y
agrega el trigger a provenance si el cuerpo no lo cita. Un cuerpo I requiere
el trigger, pero no premisas adicionales. Las asociaciones Effect quedan
inactivas. `close`, residual e incremental conservan sus entradas de reglas
fijas y no descubren esas asociaciones automáticamente.

T_Π describe la modalidad activa ya implementada para un programa fijo. No
reescribe α ni añade reglas al programa al aparecer un evento; la ejecución
mantiene llamadas activadas como estado operacional. No se adopta aquí una
reducción universal de esa modalidad a cambios ΔR. Incrementalizarla mediante
T_Π o mediante una transformación previa del programa sigue siendo una pregunta
de extensión; la derivada productiva continúa usando T_R con reglas fijas.

**MODEL RESULT M3.** La composición P seguido de Q coincidió con activación
conjunta e inline en las familias de
[composition-structure-experiment.md](composition-structure-experiment.md).
El orden \(P\preceq_\Omega Q\iff\forall F\in\Omega,
Known_P(F)\subseteq Known_Q(F)\) produjo `identidad ≼ básico ≼ extra`.
Se verificaron 4.608 comparaciones por argumento de composición, sobre 256
entradas y 384 entradas intermedias para Q. No establece una identificación
con \(\le_L\) ni una composición universal para programas con feedback.

## 8. ¿Qué significa una derivación?

Una derivación es una justificación finita de una conclusión por instanciación
de una regla y witnesses concretos. Su representación separa:

\[
UnstampedDerivation=(p,record),\quad
DerivedEvent=(p,record,t_d).
\]

`record` conserva la regla, los antecedentes y la sustitución. Instanciar no
asigna tiempo de derivación; `stamp` recibe DerivationTime externamente.

**THEOREM T7 — evidencia y provenance.** Cada conclusión inferida debe tener
witnesses y sustitución que permitan justificar ese paso en la modalidad
utilizada. La identidad de p no depende de su registro ni de su sello. Distintos
registros para la misma p deben conservarse sin duplicar p en Known.

**EXPERIMENTAL OBSERVATION E7.** El motor implementa esas separaciones y retiene
registros distintos. Los programas asociados pueden incluir un trigger implícito
en sus witnesses; su replay necesita conocer esa modalidad. El certificado
residual actual sólo reproduce reglas Horn ordinarias, no esa activación general.
La unicidad o el orden de etiquetas t_d es política del reloj suministrado, no
una ley del núcleo. No se identifica t_d con la ronda ni con PropositionTime.

## 9. ¿Qué significa EvidentialNot?

\(\sim p\) es el literal evidencialmente negativo con el mismo verbo, argumentos
y PropositionTime. Su soporte exige ese signo explícito. No significa ausencia
de p, una orden de retractación ni un StateTransform inverso.

**THEOREM T8 — negación evidencial.** Sin un hecho o prueba explícita no se
obtiene \(\sim p\) por ausencia. Añadir \(\sim p\) no retira p. Una oposición no
autoriza por sí sola conclusiones arbitrarias. El dominio puede declarar reglas
que utilicen ambos signos; sólo sus consecuencias explícitas entran al cierre.

**EXPERIMENTAL OBSERVATION E8.** El matcher exige la misma polaridad. Positivos
y negativos coexisten, incluso como conclusiones. No hay privilegio general de
`inconsistent(...)` ni `violation(...)` en el núcleo Horn. `execute_effect`
rechaza un Effect negativo; no transforma una negación en una operación de estado.

**MODEL RESULT M4.** En FOUR, EvidentialNot intercambia (p,n), es involutivo,
monótono en conocimiento y antítono en verdad. Esas propiedades corresponden
al modelo; no implementan negación compuesta universal en el AST.

## 10. ¿Qué significa una consulta residual?

Una consulta recibe un programa o regla, conocimiento disponible y un goal.
Produce condiciones pendientes acompañadas de pruebas de consulta; no añade
evidencia. El conocimiento K y el requisito lógico I son entradas/resultados
de distinta naturaleza.

**THEOREM T9 — frontera de consulta.** El conocimiento real debe permanecer
inmutable. El requirement no es una conclusión derivada ni recibe t_d.
La residuación exige la equivalencia de T3; resolver qué falta mediante búsqueda
Horn requiere un contrato computacional propio, y no demuestra por sí solo ese
adjunto general.

**EXPERIMENTAL OBSERVATION E9.** La consulta local `query_residual(rule,goal,known)`
unifica el head, reduce el body contra evidencia y devuelve requisitos con scope
y sustituciones. Un resultado local vacío puede significar que no hubo match o
que no queda requisito pendiente: esa API no distingue ambos casos.

`query_program_residual(program,evidence,goal,limits)` conserva ramas y pruebas:

| Resultado | Interpretación |
|---|---|
| `Unit` | En esa explicación no falta evidencia |
| `Event(p)` | Hipótesis explícita pendiente |
| `Tensor(H1,H2)` | Requisitos de una misma rama |
| `Join(H1,H2)` | Alternativas de explicación |
| `required: None` | No hay explicación admitida/certificada |
| `Err` | Búsqueda invalidada por scope, ciclo o límite; no resultado parcial completo |

Se requiere grounding y tipos de identidades runtime como precondición del caller;
la API valida firmas, aridad y categoría. Los parámetros de un head compatible
deben quedar determinados para la expansión admitida. Los límites por defecto
son profundidad 32, nodos 4096 y ramas 4096. No son garantías de completitud ni
sustituyen los límites de proceso. El detalle está en
[program-residual-query.md](program-residual-query.md).

La operación computacional tiene forma:

\[
ResidualQuery(\Pi,K,G,limits)
\longrightarrow Result(\{(H_i,proof_i)\},Error).
\]

Cada rama expresa un requisito suficiente según su certificado, y el resumen
Join conserva alternativas. No produce el cierre forward, no ejecuta un Effect
ni incorpora las hipótesis a K. El goal que ya pertenece a la evidencia puede
dar una rama con I, aunque K no sea vacío.

El resultado es relativo a la vista de reglas y al procedimiento de búsqueda:
`required: None` no demuestra imposibilidad en todo L, y `Err` por límite/ciclo
no demuestra ausencia de explicación. Tampoco se infiere suficiencia o
completitud respecto de FOUR a partir del nombre «residual».

## 11. ¿Qué significa una hipótesis abductiva?

H representa evidencia adicional propuesta en una rama, no ya conocida.
\(Leaves(H)\) es el conjunto de sus eventos hipotéticos para una evaluación
forward; el árbol Tensor conserva multiplicidad aunque Known sea un conjunto.

La política actual permite verbos Seeded por defecto en ambas polaridades;
el caller puede restringir los abducibles por verbo y signo. Derived y Effect
requieren prueba, no se inventan como hojas. Esto es una política de admisibilidad,
no una ley del lattice ni sintaxis `abducible` nueva.

**THEOREM T10 — suficiencia certificada.** Una rama devuelta por la consulta
de programa, cuya prueba finita pasa replay forward, debe satisfacer:

\[
G\in Close_{R(\Pi)}(K\cup Leaves(H)).
\]

Justificación condicional por inducción: hojas Evidence pertenecen a K; hojas
Hypothesis a Leaves(H); Unit no aporta eventos; cada nodo Rule aplica una
instancia forward a las conclusiones de sus subpruebas. Si el cierre completo
no termina, la prueba sigue justificando pertenencia al cierre abstracto de T5.
El código aplica ese filtro con el matcher real. Una demostración de su
implementación completa, tipos y scope queda como obligación.

Esta garantía utiliza **R(Π) como reglas explícitas**, igual que la API residual
actual: recoge las declaraciones Rule del módulo, incluidas las definiciones
referenciadas, pero no aplica la política de triggers de close_program. Ese
conjunto no es necesariamente O de la modalidad activa de la sección 7.
La garantía no se extiende sin prueba a T_Π de asociaciones arbitrarias.
Admisibilidad no implica consistencia jurídica: no hay aquí una política general
de rechazo de hipótesis opuestas ni de preferencias.

**THEOREM T11 — alternativas y no preferencia implícita.** La interfaz debe
conservar explicaciones alternativas y sus justificaciones sin forzar un mínimo
único. El resumen Join no elige una rama; no incorpora las alternativas a Known.
No se adopta cardinalidad, minimalidad ni preferencia por \(\le_L\).

**EXPERIMENTAL OBSERVATION E10.** El query conserva pruebas distintas aun cuando
sus requirements coincidan. Deduplica sólo árboles de fórmula idénticos en el
resumen Join; simplifica la unidad del Tensor sin borrar factores repetidos.
No implementa un decisor de equivalencia u orden general en L. Consistencia y
completitud fuera del fragmento certificado siguen pendientes.

**MODEL RESULT M5.** En el experimento finito, tanto `sells(juan,car,pedro) @ tau`
como `sells(charlie,car,pedro) @ tau` justifican `owns(pedro,car) @ after(tau)`.
Son incomparables por inclusión de hipótesis y por los órdenes de entrada FOUR
examinados; no tienen una menor evidencia suficiente común. Esto **no prueba**
\(H_1\not\le_LH_2\) ni la dirección inversa en una semántica general de M2.
La necesidad de conservar ambas alternativas no depende de esa identificación.
El resultado procede de enumerar cierres hipotéticos; no certifica que el query
actual pueda encontrar esas ramas cuando el head deja libre el vendedor.

## 12. ¿Qué garantiza la actualización incremental?

Para R fijo, una entrada S y adiciones ΔS, sea B = Close_R(S). La derivada se
caracteriza como un menor punto fijo sobre eventos fuera de B:

\[
\Phi_{B,R,\Delta S}(D)
=(\Delta S\setminus B)\cup(T_R(B\cup D)\setminus B),
\qquad D_R(S,\Delta S)=\mu D.\;\Phi_{B,R,\Delta S}(D).
\]

**THEOREM T12 — mantenimiento incremental.** Con base cerrada, reglas fijas y
ejecuciones que estabilizan:

\[
Close_R(S\cup\Delta S)=Close_R(S)\cup D_R(S,\Delta S).
\]

Justificación condicional: en el punto fijo, B ∪ D contiene la entrada ampliada
y está cerrado bajo R. Por inducción, cualquier extensión cerrada que contiene
B ∪ ΔS contiene las aproximaciones de D. Así B ∪ D es la menor extensión y:

\[
D_R(S,\Delta S)=Close_R(S\cup\Delta S)\setminus Close_R(S).
\]

La diferencia de cierres es una consecuencia extensional y un oráculo de
comparación, no el algoritmo. Una conclusión fuera de B requiere un witness
nuevo porque B ya está cerrado: eso justifica la evaluación por frontera.
Los axiomas antiguos con I ya se procesaron al construir B.

Además deben conservarse los registros anteriores y añadirse exactamente las
nuevas justificaciones, incluso cuando concluyan un evento ya conocido. No se
exige el mismo schedule ni sellos que una recomputación independiente; los
sellos previos deben mantenerse. El delta lógico no contiene provenance,
DerivationTime ni transformaciones de Store.

La [caracterización posterior por fixpoint de delta](discrete-derivative.md)
deriva esa igualdad sin usar la diferencia de cierres como algoritmo. Explicita
la ley de composición secuencial, la interacción entre adiciones conjuntivas
y una extensión matemática a adiciones de reglas, comprobada sólo en el harness.
Las eliminaciones y la activación incremental general de programas asociados
siguen fuera de la garantía productiva.

**THEOREM condicional — composición del delta.** Si D₁ = D_R(S, Δ₁), entonces:

\[
D_R(S,\Delta_1\cup\Delta_2)
=D_1\cup D_R(S\cup\Delta_1,\Delta_2).
\]

Los dos deltas de la derecha son disjuntos. El segundo utiliza la nueva base
cerrada B ∪ D₁. Con base fija, el delta es monótono en las adiciones; adiciones
vacías o ya conocidas dan delta vacío. No es lineal ni aditivo: una conjunción
puede requerir simultáneamente Δ₁ y Δ₂ y producir una consecuencia que ninguno
produce por separado desde B. El delta canónico tampoco es monótono en B:
excluye eventos que la base mayor ya conocía.

**MODEL RESULT M9.** La fixture y harness de la caracterización verificaron
8.192 actualizaciones con R fijo, incluyendo la menor extensión cerrada y
registros; 20.736 cambios aditivos conjuntos de hechos/reglas, sólo en el
harness; y 8.192 caminos de composición secuencial. Se conservaron
contraejemplos de aditividad y de eliminaciones mediante unión positiva.
Son familias ground de dos átomos con signos y tiempo fijo, no verificación
universal de reglas parametrizadas o activación asociada.

**EXPERIMENTAL OBSERVATION E11.** `IncrementalClosure` construye una base cerrada
y mantiene adiciones mediante matches con al menos un witness en la frontera
nueva. Sus reglas son privadas y fijas. `ClosureDelta.known` y `.derivations`
distinguen nuevos eventos de nuevas pruebas. Vacío y duplicados no reciben sellos;
los axiomas con I ya se procesaron en la base.

**MODEL RESULT M6.** Las 512 actualizaciones del experimento de composición
coincidieron en eventos y registros. La vista incremental se restringió a
asociaciones autoancladas y se comprobó contra `close_program` para cada caso.
No prueba incrementalidad general de activación, borrado, cambios de reglas,
residual, defaults o StateTransform. Las otras familias verificadas y las
restricciones de la API se registran en [incremental-close.md](incremental-close.md).

## 13. Ejecución de estados y adjunción experimental

Derivar un Effect y ejecutarlo son operaciones separadas. Un snapshot es
\(State(t)=(t,Store)\); sus tiempos no son t_d. La interpretación operacional usa
\(State(t_{in})\to State(t_{out})\) y requiere índices resueltos para ejecutar.

**EXPERIMENTAL OBSERVATION E12.** `execute_effect` valida el evento positivo,
firma, cuerpo y transición, y construye un nuevo snapshot sin modificar input.
Para un Effect en `after(t)`, la representación actual resuelve entrada t y
salida after(t); para uno en t puede quedar entrada sin resolver. Es una
convención operacional declarada, no una ley universal del tiempo.
Ni `close` ni `close_program` ejecutan Store. Remover un asset del estado no
contradice monotonía de Known: no se exige monotonía del Store por inclusión.

**MODEL RESULT M7.** El mapa experimental F de 64 evidencias firmadas de sells
a 256 conjuntos de observables Derived tuvo adjunto derecho G:

\[
F(X)\subseteq Y\iff X\le G(Y).
\]

Se comprobaron 16.384 pares en cada orden de entrada (conocimiento y verdad FOUR).
G limita la evidencia permitida para no exceder Y; no es abducción. La dirección
de menor evidencia suficiente falló para 240 objetivos en cada orden. Sus
carriers y contraejemplos constan en
[composition-structure-experiment.md](composition-structure-experiment.md).
No prueba una adjunción Verb ↔ Rule, ni una realización general de \(\multimap\).

## 14. Alcance cerrado de la fase y obligaciones abiertas

La [auditoría final código ↔ especificación](monotonic-core-final-audit.md)
clasifica cada requisito como THEOREM, MODEL RESULT, IMPLEMENTED + TESTED,
IMPLEMENTED + LIMITED o NOT IMPLEMENTED. No encontró contradicciones
operacionales en el fragmento inspeccionado; corrigió la descripción documental
de &/⊗ y conserva las precondiciones y límites de evidencia.

La [auditoría](monotonic-core-audit.md) y su
[seguimiento](monotonic-core-audit-followup.md) documentan las cuatro discrepancias
corregidas y sus regresiones. El puente de modelos y la derivada aditiva tienen
ahora caracterización y resultados propios. No se usa la existencia de un test
como prueba universal de implementación.

Para esta fase, «núcleo monotónico terminado» significa que el siguiente
fragmento tiene semántica operacional y fronteras explícitas, con la evidencia
documentada; no que desaparezcan todas las preguntas de investigación:

| Componente congelado | Garantía y dominio actual | Referencia |
|---|---|---|
| Programa tipado y especialización | Rule-lambda, firmas y sustitución scoped, incluida temporalidad | T1/T2 y auditoría F1–F4 |
| Forward por reglas explícitas | Menor cierre Horn, extensivo/monótono/idempotente; retorno sólo si estabiliza | T5 |
| Programas asociados | Activación positiva Seeded/Derived y composición existente para programa fijo | T6 y M3/M6 |
| Provenance y tiempos | Eventos deduplicados, registros distintos conservados, reloj externo; derive ≠ execute | T7 y sección 13 |
| EvidentialNot | Soporte explícito de ambos signos, sin ausencia ni retractación | T8 |
| Consulta y abducción | Requisitos separados de Known, ramas con certificado suficiente para la vista Horn explícita; límites y errores declarados | T9–T11 |
| Actualización incremental | Adiciones con R fijo, delta por fixpoint y mantenimiento de registros; sin promesa de aditividad | T12 y M9 |
| Puente de teorías/modelos | Cerrar por Horn preserva modelos en las políticas y fragmento declarados; Known no sustituye una clase de modelos | M8 y argumento condicional de sección 4 |

La clausura abstracta puede ser infinita; no se añade una obligación de
terminación para todo programa. El motor no se redefine para obtener todo
entailment FOUR ni toda abducción posible. Las garantías operacionales usan
las precondiciones de tipado, grounding, scope y modalidad especificadas arriba.

Quedan abiertas las siguientes obligaciones de generalidad. «Pendiente» no
autoriza implementar una nueva feature:

| Obligación | Situación conocida | Trabajo que quedaría por justificar |
|---|---|---|
| T3: realización proposicional de L | Traits, modelos, AST parcial y orden puntual en clases finitas | Fijar semántica general de fórmulas, equivalencia y orden; ningún decisor general actual |
| Producto lógico y soporte reusable | El matcher reutiliza witnesses; el puente ground preserva modelos | Generalizar el puente sin inferir weakening, contraction o idempotencia de tensor |
| T5/T6: cierre y activación | Algoritmos y ejemplos positivos; T5 tiene argumento abstracto condicional | Prueba de implementación, scope y límites de terminación |
| Soundness/completeness FOUR | Resultados M2 acotados | No corregir Horn para realizar inferencias algebraicas no declaradas |
| T9: API de consulta local | Vacío ambiguo entre no-match y nada pendiente | Aclarar contrato antes de tratar ese resultado como I |
| T10: replay de requisitos | Certificado Horn explícito; correcciones de grounding/scope F1–F4 con regresiones | Prueba de implementación general; no ampliar la garantía a asociaciones activas |
| Completitud residual | Suficiencia por certificado y cobertura de familias finitas | Fijar la relación objetivo y demostrar completitud; no equiparar Horn, entailment FOUR y búsqueda limitada |
| Variables de regla no determinadas por el goal | El query no enumera dominios; su expansión puede retornar Err | No interpretar el oráculo finito M5 como completitud de búsqueda de esas ramas |
| Asociaciones en residual/incremental | APIs mantienen reglas ordinarias/fijas | Especificar una extensión sólo si se decide incluirla, no inferirla de M6 |
| T11: alternativas | Join y pruebas separados; sin preferencia | No afirmar incomparabilidad en L sin un orden definido |
| T12: mantenimiento de pruebas | Caracterización de delta, argumento de frontera y M9; API con R fijo | Prueba de implementación general; API de cambios de reglas no implementada |
| Eliminaciones | La acción actual es unión de adiciones | Requieren otra semántica de cambios y retractación; no se diseñan en esta fase |
| Activación incremental general | T_Π está descrito para programa fijo; IncrementalClosure mantiene T_R | Decidir el tratamiento formal del estado de activación antes de ampliar la derivada |
| Temporalidad | After estructural; índices de estado pueden quedar pendientes | No confundir tipos temporales ni prometer una política temporal general |
| Monotonía de composición | M3 y M6 | No generalizar a feedback, activación no autoanclada o todos los programas |

El código abductivo y la línea de modelos/composición quedan congelados en el
estado documentado. `Legal` permanece como regresión. Negación por ausencia `!`,
defaults/Reiter, políticas de consistencia/preferencia, residual incremental,
interfaces, Contract y ProgramVerb como construcciones quedan fuera de este
contrato monotónico. Los defaults experimentales ya conservados no se borran ni
se consideran parte de sus garantías.

La verificación futura sigue usando `scripts/test_safe.py`: 1 GiB total, sin swap,
un job de Cargo, un hilo y timeouts. Los controles de proceso contienen una
ejecución; no prueban terminación matemática ni amplían el alcance de las pruebas.
Esta consolidación es documental: se comprobaron referencias locales y formato
del diff; no se ejecutaron compilación, tests ni fixtures. Las evidencias citadas
son las verificaciones anteriores, con sus dominios y registros originales.
