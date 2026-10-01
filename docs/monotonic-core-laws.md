# Contrato algebraico del núcleo monotónico

La [especificación provisional completa](monotonic-core-specification.md) sitúa
estas leyes dentro del núcleo y distingue THEOREM, MODEL RESULT y EXPERIMENTAL
OBSERVATION. Este documento conserva el contrato algebraico detallado; sus
property tests no certifican una realización proposicional general de M2.

Este documento fija la estructura que M2 debe realizar. Es una especificación
del núcleo, no una afirmación de que el AST o los algoritmos actuales ya realizan
todas sus leyes. `Legal` permanece congelado. Las consultas operacionales se
amplían bajo este contrato, declarando sus fragmentos y garantías por separado.

## Portador, orden e igualdad

La estructura prometida es el lattice residuado conmutativo
`(L, ∨, ∧, ⊗, I, ≤, ⊸)`. Las fórmulas tienen un preorden; `L` es su cociente por
equivalencia semántica. Contiene las interpretaciones de fórmulas bien tipadas:
átomos proposicionales, unidad, joins, meets, productos y residuales. El objetivo
algebraico exige cierre bajo esos operadores; la gramática implementada sólo
representa un fragmento de ese portador. En las representaciones usamos `≃`;
en el cociente estas equivalencias son igualdades.

Para una teoría fija de reglas, `A ≤ B` expresa consecuencia lógica. No es
subtipado de entidades ni de clases proposicionales. La jerarquía
`Effect(t) <: Derived(t) <: Prop(t)` corresponde al chequeo de tipos; no declara
por sí misma una desigualdad lógica entre eventos distintos.

El orden es un preorden:

```text
A ≤ A                                      reflexividad
A ≤ B y B ≤ C  implican  A ≤ C              transitividad
```

Definimos equivalencia semántica por:

```text
A ≃ B  si y sólo si  A ≤ B y B ≤ A
```

No exigimos antisimetría en las representaciones. `≃` no es igualdad Rust del
AST, igualdad textual, identidad de eventos ni igualdad de provenance. Una
lambda de regla es un esquema parametrizado de desigualdades; no es un evento
del mundo ni una justificación ya instanciada.

## Lattice y monoide ordenado conmutativo

Join y meet se definen mediante sus propiedades universales:

```text
A ∨ B ≤ C  si y sólo si  A ≤ C y B ≤ C     least upper bound
C ≤ A ∧ B  si y sólo si  C ≤ A y C ≤ B     greatest lower bound
```

Son asociativos, conmutativos e idempotentes y satisfacen absorción. `∧` no se
identifica con `⊗`. No exigimos que el lattice sea completo ni que tenga altura
finita; esas condiciones pertenecen a contratos adicionales para fixpoints.

El producto y su unidad satisfacen:

```text
(A ⊗ B) ⊗ C ≃ A ⊗ (B ⊗ C)                asociatividad
I ⊗ A ≃ A ≃ A ⊗ I                         unidad bilateral
A ⊗ B ≃ B ⊗ A                             conmutatividad
A ≤ A' y B ≤ B'  implican  A ⊗ B ≤ A' ⊗ B'  monotonía
```

Formal escribe el producto con `&`; `⊗` es su notación matemática en este
contrato, no un alias aceptado por el lexer actual. La monotonía se exige en
ambos argumentos. El AST puede conservar agrupación y unidades explícitas; su
interpretación debe respetar estas equivalencias.

`I` es la unidad del producto lógico, incluido el producto de cero factores.
No representa ausencia de conocimiento. No es un evento atemporal, no
entra en `Known` y no introduce un binding ni un tiempo. Tampoco se identifica
con una proposición máxima: no postulamos `A ≤ I` para todo `A`. Una regla ground
`I ≤ C` permite derivar `C` sin premisas porque la teoría contiene esa regla,
no porque la unidad implique cualquier conclusión.

## Adjunción y residual derecho

Para cada `B`, el operador `(-) ⊗ B` tiene adjunto derecho `B ⊸ (-)`:

```text
A ⊗ B ≤ C  si y sólo si  A ≤ B ⊸ C
```

`-o` y `⊸` denotan este residual. Ambas direcciones son obligatorias: currificar
y descurrificar deben preservar la misma desigualdad, con los mismos parámetros,
átomos, polaridades y expresiones temporales.

En particular, `B ⊸ C` caracteriza el mayor elemento, salvo `≃`, que puede ocupar
el lugar de `X` en `X ⊗ B ≤ C`. No significa por sí mismo «el conjunto mínimo de
hechos ausentes». Estas dos nociones requieren justificaciones distintas.

De la adjunción se siguen las siguientes leyes de aceptación:

```text
A ≤ B ⊸ (A ⊗ B)                           unidad de la adjunción
(B ⊸ C) ⊗ B ≤ C                           counidad / evaluación
I ⊸ C ≃ C                                 residual de la unidad monoidal
B ≤ B' y C ≤ C'  implican  B' ⊸ C ≤ B ⊸ C'  variancia
(B ⊗ D) ⊸ C ≃ B ⊸ (D ⊸ C)                currificación iterada
```

El residual es antítono en el requisito y monótono en el resultado. La unidad
de la adjunción y la unidad monoidal son conceptos distintos, aunque la segunda
permite deducir `I ⊸ C ≃ C`.

Ahora adoptamos explícitamente conmutatividad. Esto también permite resolver
el factor derecho mediante el mismo residual:

```text
A ⊗ B ≤ C  si y sólo si  B ≤ A ⊸ C
```

La adjunción relaciona también los operadores del lattice con el producto:

```text
A ⊗ (B ∨ C) ≃ (A ⊗ B) ∨ (A ⊗ C)
(A ∨ B) ⊗ C ≃ (A ⊗ C) ∨ (B ⊗ C)
A ⊸ (B ∧ C) ≃ (A ⊸ B) ∧ (A ⊸ C)
(A ∨ B) ⊸ C ≃ (A ⊸ C) ∧ (B ⊸ C)
```

Estas distribuciones son consecuencias de la adjunción, no mecanismos
procedurales independientes. No postulamos que `⊗` distribuya sobre meet ni que
coincida con él. La conmutatividad tampoco convierte un requisito encontrado por
matching en el residual general: aún hay que demostrar su caracterización universal.

## Sustitución, tipos y tiempo

Las leyes se aplican a fórmulas bien tipadas y en un scope declarado. Una
sustitución admisible respeta tipos y bindings, y conmuta con la estructura:

```text
θ(I) = I
θ(A ⊗ B) = θ(A) ⊗ θ(B)
θ(B ⊸ C) = θ(B) ⊸ θ(C)
θ(A ∨ B) = θ(A) ∨ θ(B)
θ(A ∧ B) = θ(A) ∧ θ(B)
A ≤ C  implica  θ(A) ≤ θ(C)
```

La especialización de una lambda por β-reduction debe preservar esas propiedades.
La sustitución del objetivo restringe una consulta; no aporta evidencia de sus
antecedentes.

Los átomos conservan su `PropositionTime`. Un contexto puede reunir átomos de
tiempos distintos sin imponerles un índice común. Ninguna ley introduce
`after(after(t)) ≃ after(t)`, un sucesor discreto ni coerciones entre tiempos.
`DerivationTime` pertenece al registro de una inferencia efectuada, no al orden
lógico ni a un requisito pendiente.

## Obligaciones computacionales

La transformación estructural debe satisfacer, en el fragmento representable:

```text
unresiduate(residuate(Horn(A ⊗ B, C))) = Horn(A ⊗ B, C)
residuate(unresiduate(ResidualClause(A, B, C))) = ResidualClause(A, B, C)
```

Aquí puede exigirse igualdad del cuerpo AST porque se conserva el corte exterior
y la agrupación. La equivalencia de evaluación forward es otra obligación:
currificar no cambia los eventos derivados ni los antecedentes que los justifican.
Esos round trips no prueban por sí solos que se haya implementado toda la adjunción.

Una respuesta pendiente de consulta debe aportar una desigualdad justificable:

```text
E ⊗ Q ≤ θ(C)
```

`E` es el producto lógico de los factores de evidencia efectivamente utilizados,
no el conjunto `Known` ni su valoración en FOUR; `Q` es el requisito pendiente
y `θ` la sustitución compatible con el objetivo y esa evidencia. El orden y la
ubicación de los factores deben estar justificados por las leyes adoptadas.
Asociatividad y conmutatividad permiten reagrupar y permutar factores. No permiten
descartarlos ni duplicarlos sin justificación: esas propiedades necesitan leyes
adicionales o una interpretación explícita de evidencia reutilizable.

Si no se utilizó evidencia, el producto de cero factores es `E = I`; esto no
identifica conocimiento vacío con unidad lógica. Devolver el antecedente instanciado
de una regla es entonces un test válido de corrección local. Si `Q ≃ I`, no queda
requisito lógico pendiente. Eso tampoco autoriza a la consulta a afirmar el goal:
la derivación forward y su provenance siguen siendo operaciones separadas.

Si una API usa todo `Known` como `E`, debe justificar también el descarte de
evidencia irrelevante. Alternativamente debe registrar el contexto concreto usado.
Corrección local, completitud de búsqueda y minimalidad de requisitos son
obligaciones separadas. Una búsqueda parcial puede ser correcta sin ser completa;
no puede presentarse como cálculo general del adjunto sin demostrarlo.

Con reglas fijas, el cierre forward conserva la monotonía de evidencia:

```text
S ⊆ S'  implica  close_R(S).known ⊆ close_R(S').known
```

Esto es una propiedad del conjunto de hechos conocidos, distinta de la monotonía
de `⊗` en el preorden. `EvidentialNot` requiere evidencia explícita: la ausencia
de `P` no deriva `~P`. Derivar un `Effect` no ejecuta su transformación de estado.

## Lo que no se deduce de este contrato

No se adoptan por implicación idempotencia del producto, weakening, contraction,
negación por ausencia, explosión desde `P` y `~P`, completitud abductiva, ni una
política de defaults. Cualquiera de esas extensiones necesita una decisión explícita.
Tampoco se deduce una igualdad entre tiempo proposicional, tiempo de derivación
y tiempos de entrada/salida de una transición.

El matcher actual permite reutilizar un evento de `Known` para varios átomos y
encontrarlos sin consumirlo. Eso es una política operacional de evidencia
reutilizable, más fuerte que lo que especifica un monoide residuado sin otras
leyes. Antes de usarla como interpretación general de `⊗`, habrá que fijar qué
leyes estructurales se adoptan o cómo se distingue esa evidencia del contexto
monoidal. No afirmamos que el motor actual implemente una lógica de recursos lineal.

## Interfaz algebraica y property tests

[src/algebra.rs](../src/algebra.rs) especifica `Preorder`, `Lattice`, `Monoid` y
`CommutativeResiduatedLattice`. Comparten un carrier y el mismo orden; no son
implementaciones independientes de cinco operadores del lenguaje.
`b.residual(c)` denota `B ⊸ C`. `equivalent` usa las dos desigualdades, sin exigir
igualdad estructural de representaciones. Los traits expresan una obligación,
no una prueba: Rust no verifica por sí mismo estas leyes.

[src/algebra/tests.rs](../src/algebra/tests.rs) comprueba las propiedades
exhaustivamente sobre dos carriers finitos completos:

- El powerset del grupo conmutativo `Z/2Z`, ordenado por inclusión. Join es unión,
  meet es intersección, tensor es suma de elementos del grupo y `I = {0}`.
  `B ⊸ C = {x | para todo b en B, x + b está en C}`. Sus cuatro elementos
  distinguen producto, meet, join y unidad; `I` no es máximo ni mínimo.
- La cadena de Łukasiewicz `{0, 1, 2}`, con orden numérico, join máximo, meet mínimo,
  `a ⊗ b = max(0, a + b - 2)`, `I = 2` y `b ⊸ c = min(2, 2 - b + c)`.
  Aquí `I` sí es máximo, pero el producto sigue siendo distinto del meet.

Son modelos de referencia del contrato, no una elección de semántica para las
proposiciones de Residuum. El primero no es `Known`, aunque ambos usen conjuntos:
su tensor es una operación de grupo y su unidad no es el conjunto vacío.
El segundo no impone integrality al contrato de M2.

Los [modelos de pares de evidencia](evidence-pair-models.md) añaden FOUR con
orden de conocimiento, orden de verdad y negación por intercambio de componentes.
Meet de verdad y fusión son dos productos que satisfacen el contrato residuado;
los tests distinguen sus unidades y preservan los contraejemplos de compatibilidad
con acumulación de conocimiento. No se identifica ese carrier con el AST de L
ni se cambia el puente de satisfacción o el closure actual.

El [experimento de comparación de requisitos](requirement-order-experiment.md)
conecta esos modelos con fórmulas obtenidas por la consulta sobre programas.
`interpret_requirement` es un adaptador con valoración explícita;
`Preorder::compare` usa las dos direcciones del mismo orden. No es un decisor
general de desigualdades entre fórmulas de M2. Los contraejemplos muestran que
añadir un factor no implica necesariamente bajar en el orden: se necesita
justificar `B ≤ I` para obtener `A ⊗ B ≤ A`. La integralidad no se adopta.

Las propiedades incluyen cierre de operaciones, reflexividad/transitividad,
bounds universales, leyes de lattice, monoide conmutativo y monotonía, ambas
direcciones de la adjunción, unidad/counidad, variancia, distribución y
currificación iterada. Se recorre cada tupla del carrier; no se añaden dependencias
ni generadores sin límite. Son exhaustivos para esos modelos, no una prueba
para todo modelo posible ni para el AST proposicional actual.

El contraejemplo `monotone_map_need_not_have_a_right_adjoint` usa la función
constante `f(X) = {0}` del primer modelo. Es monotónica, pero para el target vacío
ningún candidato a `f*(vacío)` cumple la adjunción. Por tanto, `ProgramVerb`
necesitará contratos separados para monotonía y para tener adjunto derecho.
En lattices completos, preservar todos los joins, incluido el vacío, caracteriza
la existencia de ese adjunto; preservar sólo joins binarios no basta.

## Frontera con conocimiento, satisfacción y ejecución

`K` sigue siendo el dominio de conjuntos de eventos ground bien tipados,
ordenados por inclusión; `∨_K` es unión. No se identifica con `∨_L`. El contrato
algebraico no incorpora snapshots, provenance ni `DerivationTime` a sus elementos.
Su mínimo de conocimiento es `⊥_K=∅`. En la valoración atómica FOUR, ese conjunto
asigna U a cada átomo; no identifica todo `K` con un solo valor. La unidad `I`
es B en el modelo de fusión y T en el modelo de meet. El bottom lógico de ambos
modelos es F. Son tres papeles diferentes; ni `I=B` ni tener un bottom lógico
se impone todavía a todas las realizaciones de M2.
Los átomos lógicos conservan tiempo proposicional y polaridad; los registros de
derivación acompañan inferencias sin determinar equivalencia algebraica.

La identidad lógica de `Event` es `(verb, arguments, propositionTime, polarity)`;
su igualdad y orden no incluyen `StateTransitionType`. La clasificación del verbo
se chequea por separado y no añade un componente a esa clave. Una interpretación
operacional parcialmente resuelta debe poder coincidir con un requisito lógico.
La ejecución sí valida los tiempos de estado. El cierre conserva la primera
representación de un evento lógico, sin fusionar interpretaciones, y las
transformaciones que prometan preservar metadata deben comprobarla explícitamente.

El matcher es un candidato a puente parcial con testigos `Match(F, A) = {(θ, π)}`.
Para una regla `A ≤_L B`, la corrección forward debe distinguir evidencia inicial
de conocimiento cerrado: `F ⊨ A` permite exigir `Close_R(F) ⊨ B`, no que `B` ya
pertenezca a `F`. La satisfacción completa de joins, meets y residuales queda
pendiente. Dar a tensor y meet la misma satisfacción booleana con evidencia
reutilizable puede ocultar su diferencia; no demuestra una realización fiel de `L`.

Una transformación `f_L` tampoco determina automáticamente cómo producir evidencia
mediante `f_K`. Esa realización, su monotonía y su derivada incremental tienen
obligaciones propias. No se implementa todavía `ProgramVerb`, un decisor general
de desigualdades, un nuevo evaluador de satisfacción ni una migración del runtime.

Las leyes algebraicas siguen la definición y los lemas de
[Galatos y Ono](https://www.jaist.ac.jp/~galatos/research/slai.pdf).
El criterio para adjuntos en órdenes completos se formaliza en
[Mathlib, Galois connections](https://leanprover-community.github.io/mathlib4_docs/Mathlib/Order/GaloisConnection/Basic.html).

## Cobertura actual y revisión pendiente

| Obligación | Evidencia actual | Pendiente |
| --- | --- | --- |
| Lattice residuado conmutativo abstracto | Traits y property tests exhaustivos en dos modelos de referencia | Realización proposicional de M2 |
| Dos direcciones del corte `A ⊗ B ≤ C` / `A ≤ B ⊸ C` | Transformaciones estructurales y tests de round trip del fragmento | Interpretación general del preorden |
| Unidad bilateral | `Antecedent::Unit`, matching y tests de identidad | Extender las leyes a todas las fórmulas de `L` |
| Comportamiento forward del corte | Normalización a Horn y tests de cierre/provenance | No confundir estos tests con completitud algebraica |
| Asociatividad y monotonía del producto | Árboles conjuntivos y matching | Criterios semánticos explícitos y tests de leyes |
| Reflexividad y transitividad | Inferencia forward encadenada | No existe todavía un decisor general de desigualdades |
| Unidad/counidad, variancia y residual iterado | Consecuencias del contrato anterior | El AST actual no representa todos sus términos |
| Sustitución y β-reduction | Tests del fragmento tipado | Cobertura de fórmulas generales |
| Consulta desde el goal con `Known` vacío | Tests de unificación, evidencia parcial, scope y metadata independiente | Corrección general como realización del adjunto |
| LegalAbduction de dos reglas | Harness acotado de tres escenarios y validación hipotética | Búsqueda general, preferencia en `L` y consistencia programable |
| Consulta sobre programas con alternativas | `RequirementFormula::Join`, ramas con testigos, tres escenarios Legal y oráculo forward finito | Variables no determinadas por el head, ciclos, preferencia y consistencia |
| Join/Tensor de requisitos y comparación | Adaptador hacia modelos explícitos, bounds del join y contraejemplos de orden | Realización proposicional de `L`, leyes estructurales y preferencia admisible |
| Suficiencia de la consulta abductiva | Seeds abducibles por defecto, filtro final y certificado con matcher forward compartido | Completitud fuera del fragmento certificado; sin política de preferencia |

`Residual { required: Antecedent, consequent: Event }` y `RuleBody` son actualmente
representaciones de un fragmento, no un AST cerrado bajo residuales arbitrarios.
Los cuatro tests algebraicos pasan individualmente bajo `scripts/test_safe.py`.
Eso certifica las propiedades comprobadas en los dos modelos finitos, no las
del pipeline proposicional. Los tests de consultas se actualizaron para unificación
desde el goal y reducción con evidencia parcial; ese fragmento y el harness
`delivery → sales` se verifican individualmente bajo el runner seguro. La
reducción local de «átomos faltantes» todavía debe revisarse contra este contrato,
especialmente cuando hay factores intermedios o variables que el goal no fija.

La [consulta sobre programas](program-residual-query.md) añade búsqueda acotada
con requisitos ground y alternativas explícitas. El oráculo comprueba cobertura
en una fixture finita; no prueba completitud sobre `L` ni sobre programas cíclicos.
El join del resultado es estructura lógica, independiente de la unión de `Known`.

El bloque de abducción monotónica queda congelado después de incorporar el
certificado forward para cada resultado. `Seeded ⇒ Abducible` es política de
declaración por defecto, no una ley de `L`. El álgebra no selecciona explicaciones;
no se adopta integralidad ni se crea otro orden. El
[puente de valoración y satisfacción](satisfaction-bridge-experiment.md) ya compara
soporte positivo con el matcher y el cierre ground en ambos modelos FOUR. Esa
coincidencia no se extiende a la desigualdad puntual de verdad: un cierre Horn
estable puede incumplirla. Queda precisar la interpretación de la teoría antes
de una migración o de incrementalidad del cierre forward.

Cada ampliación debe declarar qué fragmento realiza, probar ambas
direcciones de residuación en ese fragmento y separar las garantías de corrección,
completitud y minimalidad. No requiere modificar ahora gramática, runtime,
defaults ni `Legal`.
