# Contrato del espacio de configuraciones jurídicas

Investigación en `experiment-legal-order-geometry`. Este documento define un
fragmento de búsqueda y sus obligaciones de verificación. No implementa un
generador, subsunción, Candidate Elimination ni una decisión efectiva nueva.

Referencia de ejecución: `main`/HEAD
`7b14cb8ac86d07c8d101dc36ca9f1161580f6c22`. La ontología, las fixtures y el motor
se mantienen intactos. Continuación de
[la formalización del modelo](legal-recalcitrance-mathematical-model.md).

Se distinguen cuatro estados: **IMPLEMENTED**, comportamiento presente en el
código; **FINITE RESULT**, comprobación acotada; **THEOREM**, argumento matemático
bajo hipótesis explícitas; **PROPOSED CONTRACT**, decisión experimental que
todavía no está implementada o certificada.

## 1. Extracción de las signaturas

La fuente es el `Module` elaborado por `load_formal_module`, no una inferencia a
partir de los nombres. El test existente
`cli::tests::legal_geometry::correspondence_and_typed_inventory` exporta el AST,
las firmas y los orígenes. Se volvió a ejecutar individualmente mediante
`scripts/test_safe.py`; su resultado actual figura en el apartado 11.

Los inventarios completos previos están en
[general-inventory.tsv](legal-geometry/data/general-inventory.tsv) y
[general-verbs.tsv](legal-geometry/data/general-verbs.tsv). La extracción actual
del fragmento se conserva en
[search-space/signatures.json](legal-geometry/search-space/signatures.json),
con las posiciones, los tipos y las fuentes elaboradas.

Los tres efectos tienen estas firmas exactas, además del índice `@ t`:

```text
give(subject : Person, object : Thing, recipient : Person)
do(subject : Person, conduct : Conduct, indirectObject : Thing?, beneficiary : Person)
notDo(subject : Person, conduct : Conduct, indirectObject : Thing?, beneficiary : Person)
```

Los siguientes ocho verbos comparten la secuencia exacta de argumentos:

```text
(id : Fact, subject : Person, conduct : Conduct, object : Thing,
 beneficiary : Person, authority : Authority, effective : PropositionTime)
```

| Verbo | Categoría | Función en este fragmento |
|---|---|---|
| `Act` | Seeded | Evidencia histórica |
| `obligationFiled` | Seeded | Canal documental de obligación |
| `obligation` | Derived | Obligación obtenida por reglas |
| `right` | Seeded | Derecho |
| `permissionFiled` | Seeded | Canal documental de permiso |
| `permission` | Derived | Permiso obtenido por reglas |
| `prohibition` | Seeded | Prohibición |
| `omissionProhibition` | Seeded | Prohibición de omitir |

Las cinco calificaciones son obligación, derecho, permiso, prohibición y
prohibición de omitir. Obligación y permiso conservan sus dos canales; no se
fusionan sus declaraciones. El vocabulario contiene once verbos y doce
literales firmados al añadir `~Act`.

`~Act` es `Act` con `Polarity::EvidentialNot`; no tiene una signatura distinta.
El perfil inicial permite ambos signos para `Act` y signo positivo para los
otros diez verbos. Esta última limitación es **PROPOSED CONTRACT** del fragmento,
no una prohibición general del lenguaje.

Todo evento tiene `PropositionTime` como índice. `effective` es otro argumento
de ese tipo y puede tener un valor diferente. `DerivationTime` no es un
parámetro jurídico de estas firmas. `StateTransitionType` acompaña a los efectos
como metadata operacional, fuera de la identidad lógica del evento.

`Arg` conserva nombre y tipo. No existe un tipo nominal independiente para
«vendedor», «acreedor» o «beneficiario». Los nombres identifican posiciones;
las identidades se establecen mediante bindings concretos. Dos variables de
tipo `Person` pueden coincidir al hacer matching; escribir variables distintas
no impone desigualdad.

## 2. Conexiones proporcionadas por la teoría

`LegalGeneral` tiene 27 reglas. La referencia `LegalMonotonicCase` añade
`complianceAsset`; la extensión patrimonial incorpora esa regla y las constantes
`deliver`, `molest`. Los cuerpos coinciden según la partición, pero los módulos
y archivos de origen se comprueban por separado.

Las conexiones relevantes ya declaradas son:

| Antecedentes | Consecuencia | Identidades y tiempos |
|---|---|---|
| `obligationFiled` | `obligation` | Todos los argumentos y `@ t` se conservan |
| `permissionFiled` | `permission` | Todos los argumentos y `@ t` se conservan |
| `obligation` | `do` | Sujeto/conducta/objeto/beneficiario compartidos; `@ after t` |
| `prohibition` | `notDo` | Mismos cuatro bindings; `@ after t` |
| `Act` con `conduct = transfer` | `transferred` | Se conserva el identificador del acto y `@ t` |
| `transferred` | `give` | Sujeto/objeto/destinatario; `@ after t` |
| `obligation` o `right`, junto con `Act` | `compliance` | Sujeto/conducta/objeto/beneficiario e índice compartidos |
| `obligation` y `~Act` | `violation` | Los mismos vínculos entre norma y acto |
| `right` y `~Act` | `enforceability` | Los mismos vínculos entre derecho y acto |
| `prohibition` y `Act` | `violation` | Los mismos vínculos entre prohibición y acto |
| `omissionProhibition` y `~Act` | `violation` | Los mismos vínculos entre prohibición y acto |

El identificador de la norma y el del acto permanecen separados. En las reglas
de cumplimiento/violación/exigibilidad, la autoridad y la vigencia del acto son
binders separados de las de la norma. No se exige que sean iguales. Esa igualdad
sería una especialización candidata, no una condición original de la teoría.

Ejecutar `do` añade un record `Relation` con `mode = Doing` a obligaciones del
sujeto y derechos del beneficiario. Ejecutar `notDo` hace lo mismo con
`Abstaining`. Ejecutar `give` quita el objeto del conjunto de activos del sujeto
y lo añade al del destinatario. `close_program` deriva efectos, sin ejecutarlos
ni convertirlos automáticamente en `Act`/`~Act`.

La gramática podrá reutilizar estas conexiones; no presupone que todo patrón
tipado sea equivalente a una de ellas ni que todo patrón constituya una razón
normativa suficiente.

## 3. Objeto candidato y gramática

Fijamos una teoría inmutable `T`, un módulo de referencia, un vocabulario de
constantes `C`, una interfaz de salida `Q` y un presupuesto `b`. Los cinco se
declaran antes del ciclo. Para este contrato, `Q` se elige entre firmas Derived
o Effect ya existentes. Un objetivo sin firma existente requiere otro incremento
explícito; aquí no se añade una declaración de impedimento.

Una configuración conserva:

\[
\Gamma=(id,parameters,rules,RuleUses,module,activation,origins).
\]

Para que el perfil inicial esté completamente instanciado, fijamos
`T = LegalGeneral` y la interfaz positiva `Q0 = compliance`, cuya firma es:

```text
compliance(norm : Fact, act : Fact, subject : Person, object : Thing,
           beneficiary : Person, authority : Authority,
           effective : PropositionTime) @ t
```

Esta elección retoma una consecuencia cuya frontera ya está certificada y
delimita una prueba de representación. No asigna nuevas valoraciones externas
ni decide qué significa una decisión jurídica efectiva. Otros objetivos
requieren un manifiesto propio.

`rules` es una lista de cláusulas individuales. `RuleUses` referencia reglas
existentes o esas cláusulas experimentales. Este registro describe un objeto
del harness; sus componentes ejecutables utilizan `Rule`, `Program`, `RuleUse`
y los bindings que ya tiene Residuum. No propone un segundo AST del motor.

La gramática abstracta es:

```text
Term[T]     ::= variable tipada | constante declarada asignable a T
              | none, cuando T admite ausencia
              | after(Term[PropositionTime])
              | record declarado con campos tipados
Time        ::= constante temporal | parámetro temporal | after(Time)
Literal     ::= verbo(Term[T1], ..., Term[Tn]) @ Time
              | ~Act(Term[Fact], ..., Term[PropositionTime]) @ Time
Antecedent  ::= I | Literal | Antecedent & Antecedent
Clause      ::= Antecedent <= Head[Q]
RuleUse     ::= referencia a Rule + bindings parciales tipados
Program     ::= parámetros compartidos + secuencia acotada de RuleUses + módulo
```

Para programas con consecuencias intermedias, `Head[Q]` se amplía por un
manifiesto explícito de firmas Derived/Effect existentes, incluidas las
calificaciones `obligation`, `permission` cuando corresponda. La interfaz final
sigue fija. No se inventan verbos auxiliares ilimitados.

El perfil básico utiliza cuerpos Horn. `RuleBody::ResidualClause` ya existe:
una cláusula `A <= (B -o C)` puede proyectarse como `A & B <= C` mediante
`unresiduate`. Su forma original debe conservarse si se incorpora al catálogo.
La gramática inicial no enumera ambas presentaciones como si fueran distintas
operaciones jurídicas; su cobertura puede coincidir y su formulación diferir.

El AST tipado admite records y `after` anidados. No posee un constructor de
literales de conjuntos en `Term`; el perfil no lo inventa. La admisibilidad de
un record exige todos los controles reales de `lower_term`. Por ejemplo, una
estructura recursiva `Person -> Patrimony -> Set Relation` no implica que exista
un literal finito de `Person` expresable con cualquier combinación de campos.
El perfil inicial usa constantes y variables de esas entidades.

Los bindings omitidos en `RuleUse` permanecen locales a su regla. Los parámetros
compartidos se sustituyen en todos los usos; `compile_program` renombra los
locales higiénicamente. La aplicación parcial fuente actual aplica un prefijo
de constantes declaradas; no ofrece una unificación simbólica arbitraria entre
dos programas. Las transformaciones más generales de patrones pertenecen al
harness futuro.

## 4. Admisibilidad estructural

Definimos `Struct(T, Γ)` mediante las siguientes obligaciones comprobables:

1. Cada tipo, constante, verbo y referencia existe en el ámbito del módulo.
   Importar conserva los orígenes; no suministra valores nuevos automáticamente.
2. La aridad y la posición de cada argumento corresponden a la firma. Las
   asignaciones siguen las reglas de `rule_type_assignable`, incluidos opcionales
   y subtipado declarado. Las fuentes jurídicas aquí examinadas no declaran
   subtipos.
3. Cada binder tiene un único tipo compatible con todos sus usos. Constantes y
   parámetros conservan identidades distintas, incluso si son homónimos.
4. Los tiempos son términos `PropositionTime`. `after` construye un sucesor
   simbólico; no calcula una fecha ni una desigualdad. Comparaciones jurídicas
   de autoridad/fecha requieren las relaciones explícitas de la teoría.
5. Todo parámetro de la cabeza debe obtener su valor por matching del cuerpo,
   bindings aplicados o trigger de activación. Un parámetro de salida sin fuente
   de valor no constituye una enumeración existencial. Esta seguridad es una
   restricción del generador propuesto: la elaboración actual no demuestra por
   sí sola que cada regla produzca una cabeza ground.
6. Los parámetros locales y compartidos tienen ámbitos distintos. La sustitución
   debe recorrer constructores, tiempos y records sin capturar binders locales.
7. La instalación de reglas y la activación son explícitas. Una regla asociada
   exige el trigger positivo correspondiente, también cuando su cuerpo es `I`.
   Las reglas de defaults sólo entran mediante la selección existente.
8. La conclusión candidata del perfil pertenece a la interfaz Derived/Effect.
   No se aprende evidencia histórica mediante cabezas `Act` o `~Act`. Esta es
   una política de investigación; no se atribuye al elaborador un rechazo
   universal de todas las cabezas Seeded positivas.

Para delimitar configuraciones relacionadas con `Q`, se define además un perfil
conectado: el grafo de incidencia contiene nodos para átomos y para identidades
de variables/constantes compartidas. Cada átomo del cuerpo debe estar conectado
a una cabeza observable mediante esos bindings, referencias entre reglas o una
dependencia inferencial explícita de `T`. El origen de cada enlace se registra.
Una coincidencia de tipo no crea un enlace; una constante compartida sí puede
crearlo. Esto comprueba conexión estructural, no relevancia jurídica suficiente.

La condición de conexión es **PROPOSED CONTRACT**, no una obligación global del
lenguaje. El catálogo completo y su subcatálogo conectado deben distinguirse;
ninguna conclusión de completitud de uno se transfiere al otro.

No se añaden presupuestos tácitos como exclusividad de permisos/prohibiciones,
identidad de toda autoridad, orden cronológico por nombre, desigualdad de
personas, consistencia automática ni prioridad de una conclusión. Si una regla
original deriva `annulled` o `inconsistent`, se conserva su significado actual:
un hecho Derived con las consecuencias expresamente declaradas.

## 5. Presupuesto finito

El presupuesto es un vector, no un límite de recursos del runner:

\[
b=(a,r,u,v,d_t,d_r,C,M,P).
\]

`a`: átomos por cuerpo; `r`: nuevas cláusulas por configuración; `u`: RuleUses;
`v`: binders; `d_t`: profundidad de `after`; `d_r`: profundidad de records;
`C`: vocabulario finito de constantes; `M`: módulos autorizados;
`P`: perfiles de instalación/activación permitidos. Los tipos y firmas se toman
del ámbito fijo. `Program` no tiene miembros que sean otros `Program`: los usos
actuales apuntan a `Rule`; no se inventa nesting de programas.

Perfil inicial propuesto:

| Parámetro | Cota |
|---|---:|
| Nuevas cláusulas | 1 |
| Átomos en su antecedente | 4 |
| RuleUses, al estudiar agrupación posterior | 4 |
| Binders totales | 16 |
| Profundidad de `after` en candidatos | 2 |
| Profundidad de records en candidatos | 0 |
| Perfil de ejecución inicial | Adición explícita de una cláusula ordinaria |
| Cabeza inicial | `compliance`, positiva, con su firma existente |

`C` es la unión de las constantes generales declaradas y las constantes
sintéticas ya presentes en `GeometryGeneral`, con sus tipos elaborados. Se
conserva en el manifiesto adjunto. No se incorporan datos de `12hectareas`.
Las constantes `deliver`, `molest` requieren elegir expresamente la extensión
patrimonial; `pay`, `Plain`, `Retention` pertenecen a `Sales` y no están en este
perfil general.

La cota de binders permite expresar las reglas de cumplimiento/violación ya
inspeccionadas, que separan norma, acto y sus autoridades/vigencias. No se afirma
que baste para reconstruir toda la legislación ni la teoría sucesoria.

**THEOREM — Enumerabilidad finita.** Con firmas y constantes finitas, profundidad
de términos acotada, aridades finitas, cotas de átomos/cláusulas/usos y nombres
de variables canonizados por renombrado consistente, existe un número finito
de AST candidatos. Prueba: inducción en la profundidad de términos tipados;
cada nivel elige entre un número finito de constructores/records y productos
finitos de términos anteriores. Los átomos, cuerpos y secuencias acotadas son
entonces finitos. Los filtros estructurales producen un subconjunto finito.

Las identidades arbitrarias de candidatos no forman parte de la enumeración:
son índices del catálogo. Se conservan formulaciones originales y procedencias
como fibras asociadas a cada estructura, sin crear infinitas variantes por
renombrar `x` como `x1`, `x2`, etc. No se borran variantes con distintos
antecedentes ordenados, reglas, programas u orígenes.

Esta prueba no proporciona una cota práctica pequeña. Tampoco demuestra
terminación de cada cierre: incluso `A(x) <= A(after(x))` puede producir una
cadena infinita con un AST finito. Fin de presupuesto, timeout y salida del
universo ground acotado son estados de evaluación sin certificar; no etiquetas
jurídicas negativas. Los límites del runner permanecen intactos.

## 6. Dominio de evidencia y evaluación

Separa el catálogo de candidatos `H_b` del dominio de expedientes `A`.
Una primera elección reproducible es el conjunto de subconjuntos de la evidencia
Seeded del experimento `performance` de `GeometryGeneral`: siete eventos ground,
incluidos ambos signos de `Act`, con todos sus sujetos, objetos, identificadores
y tiempos. Hay 128 entradas posibles. Este es un dominio finito del experimento;
no es el dominio jurídico universal ni el expediente histórico de Riggs.

El cierre se calcula con toda `LegalGeneral`, no sólo con los once verbos del
vocabulario candidato. Así permanecen los productores originales y todas sus
consecuencias patrimoniales, normativas y legislativas que lleguen a habilitarse.

Cada candidato del perfil inicial produce una copia experimental
`T_Γ = T ⊕ Γ`, donde `⊕` significa exclusivamente adición de cláusulas ordinarias.
El origen de cada nueva regla se distingue del de `T`. `T`, los seeds originales
y sus justificaciones no se editan. Los experimentos existentes de Riggs ya
utilizan copias hipotéticas y cierres independientes con esta clase de operación.

En programas posteriores se requiere registrar también el plan de instalación:
miembros compilados introducidos conjuntamente, asociación a un trigger existente
o selección de defaults ya definida. Estas alternativas no son intercambiables.
En `prepare_programs` se excluyen del cierre ordinario las definiciones de
comportamientos asociados. Declarar un `Program` por sí solo no crea esa
exclusión para cualquier regla referenciada; el flujo de defaults aplica sus
propios controles de selección. No se debe convertir un paquete pasivo en reglas
obligatorias por simplificar el comparador.

La evaluación tiene dos capas:

\[
F_\Gamma(S)=C_{T_\Gamma}(S),\qquad
N_\Gamma(S)=D(F_\Gamma(S),Q).
\]

`D` permanece fijo durante el ciclo. Observación:
`ε = (S, Q, y*, fuente, fundamento, alcance, estado)`; la etiqueta externa no
se inserta en Known. El criterio explícito `κ_ε` compara la decisión con la
valoración; cuando la valoración queda pendiente, no elimina hipótesis.

Una evaluación debe conservar los eventos completos, sustituciones canónicas,
antecedentes ordenados, productores y sus módulos, y multiplicidad de
justificaciones. Se permite que cambie el valor concreto de `derivedAt` entre
ejecuciones independientes. El estado de completitud del cierre se registra.

El contrato de decisión efectiva para exclusión sucesoria sigue pendiente.
El experimento existente aprende productores de un impedimento hipotético:
añadir ese evento no retira automáticamente la atribución. Este documento no
declara un `D` nuevo para convertir el marcador en exclusión.

Además, el perfil de adición de productores positivos de `compliance` sólo
puede conservar o ampliar su región. No puede corregir una valoración que
exija retirar un cumplimiento ya derivado por `T`. Esa observación requiere
otra operación hipotética o un contrato de decisión explícito; no se cambia
la etiqueta para hacerla compatible con este perfil.

## 7. Orden de generalidad y equivalencias

Fijados `T`, `A`, interfaz observable `Q` y plan de ejecución, definimos cobertura
relacional, que conserva valores y objetivos ground:

\[
Cov(\Gamma)=\{(S,q):S\in A,\ q\in Q_{ground},\ q\in F_\Gamma(S)\}.
\]

No se sustituye un expediente por indicadores O/R/P/A. Esos símbolos pueden
nombrar eventos concretos certificados, pero los conjuntos contienen eventos
con identidad, argumentos, polaridad y tiempo.

La orientación elegida es:

\[
\Gamma_s\preceq_g\Gamma_g\iff Cov(\Gamma_s)\subseteq Cov(\Gamma_g).
\]

«Más específica» está debajo de «más general». Es un preorden relativo al
dominio y objetivo; no se identifica con `≤_L`, con inclusión de declaraciones
ni con refinamiento de particiones. Para programas con distintas condiciones de
activación, éstas participan en la evaluación que define `Cov`.

Dos equivalencias se anotan separadamente:

* Cobertura: `Cov(Γ1) = Cov(Γ2)`.
* Clausura: `F_Γ1(S) = F_Γ2(S)` para todas las entradas declaradas, comparando
  eventos. Es más exigente que observar sólo una interfaz seleccionada.

Ninguna exige igualdad de las explicaciones. Una tercera comparación puede
examinar sus multiconjuntos de provenance y sus formulaciones. Esas fibras se
conservan aunque las dos primeras equivalencias coincidan.

**THEOREM — Subsunción estructural suficiente.** Para cláusulas positivas Horn
ordinarias, con mismo ámbito tipado, y negaciones evidenciales tratadas como
literales firmados, `r_g = A_g <= h_g` subsume `r_s = A_s <= h_s` si existe una
sustitución tipada `θ` tal que `h_g θ = h_s` y los átomos de `A_g θ` están
contenidos en los de `A_s`. Entonces la cobertura de `r_s` está contenida en la
de `r_g` bajo el mismo entorno positivo. Prueba: todo testigo ground de `A_s`
es testigo de `A_g θ`; el resultado de la cabeza coincide. La simulación se
extiende por inducción a las rondas del cierre Horn ordinario.

La prueba compara soporte, no identidad de justificaciones. La implementación
del comparador de patrones no existe todavía: el matcher actual enfrenta
patrones con eventos ground, no resuelve subsunción simbólica entre cláusulas.
La inclusión de átomos tampoco es necesaria para subsunción relativa a `T`:
si `T` contiene `B(x) <= A(x)`, exigir `B(x)` puede ser más específico que
exigir `A(x)` aunque los nombres de los predicados sean distintos.

**THEOREM — Comparación por fronteras.** Si las coberturas de una conclusión
ground fija son regiones ascendentes sobre el mismo dominio `P(Σ)`, entonces
`R_s ⊆ R_g` si y sólo si para cada soporte mínimo `B_s` de `R_s` existe un
soporte mínimo `B_g` de `R_g` con `B_g ⊆ B_s`. Prueba: todo estado de una región
ascendente finita contiene un soporte mínimo; aplicar esa propiedad en ambas
direcciones. No se aplica a resultados efectivos arbitrarios de `D` sin probar
que sus regiones sean ascendentes.

Los certificados de fronteras actuales trabajan con eventos ground y reglas
fijas. No certifican todas las equivalencias de configuraciones nuevas.

## 8. Operaciones de búsqueda y obligaciones pendientes

| Operación propuesta | Dirección en el fragmento Horn | Condición |
|---|---|---|
| Añadir antecedente | Especialización | Misma cabeza y ámbito; seguridad conservada |
| Quitar antecedente | Generalización | Bindings necesarios de la cabeza conservados |
| Identificar variables | Especialización | Sustitución tipada consistente |
| Separar usos de una variable | Generalización posible | Preservar la cabeza y el vínculo con variables restantes |
| Fijar una variable a constante/constructor | Especialización | Recorrer todos sus usos y respetar tipo |
| Relajar constante a variable | Generalización posible | Elegir coherentemente usos compartidos y seguridad |
| Desplegar un productor de `T` | Especialización o equivalencia | Incluir cuerpo y bindings del productor escogido |
| Plegar cuerpo a su consecuencia en `T` | Generalización posible | Otros productores pueden ampliar cobertura |
| Agrupar RuleUses | Unidad ejecutable | Mismo cierre; plan de instalación/activación conservado |

Las direcciones son teoremas del fragmento positivo con las condiciones
indicadas, no una demostración de todos los Program dinámicos. Separar variables
o relajar constantes admite varias transformaciones incomparables; no es una
operación inversa única. Quitar y añadir reglas cambia también la clausura de
otras consecuencias. La evaluación debe conservar esas diferencias.

Las fronteras y provenance permiten proponer despliegues, bindings y soportes:
se extraen los productores y los testigos que realmente llevaron a una
consecuencia. No prueban que todo candidato válido deba aparecer en una prueba
ya observada. Restringir la generación a esa provenance sería otro sesgo explícito.

Antes de mantener fronteras incrementalmente se requieren operadores completos
de generalización mínima y especialización máxima dentro de `H_b`, con todas
las variantes tipadas, temporales y dependencias relevantes. Un recorrido de
añadir/quitar átomos no demuestra esa completitud. Algunas transformaciones
necesitan varios cambios simultáneos para conservar la seguridad del resultado.

## 9. Convexidad y retículas de particiones

Para una observación, sea

\[
A_\varepsilon=\{\Gamma:\kappa_\varepsilon(D(F_\Gamma(S),Q))=1\}.
\]

El espacio de versiones es la intersección de esos filtros. Para representarlo
exactamente por extremos se debe demostrar convexidad de orden: si
`Γ1 ≼ Γ2 ≼ Γ3` y los extremos pertenecen al filtro, también debe pertenecer
`Γ2`. En un poset finito, una región convexa se reconstruye a partir de sus
elementos mínimos y máximos. Puede necesitar varios extremos incomparables;
no implica un único intervalo.

Para observaciones binarias de derivabilidad y orden por inclusión de cobertura,
las observaciones positivas definen filtros ascendentes y las negativas
descendentes; su intersección es convexa. Las actualizaciones clásicas de
Candidate Elimination necesitan además esa orientación y operadores completos.
No se ha demostrado que las valoraciones jurídicas efectivas tengan esa forma.

**Contraejemplo matemático a «D fijo garantiza convexidad».** Consideremos
configuraciones con coberturas anidadas que producen, en un expediente,
respectivamente cero, una y dos consecuencias observables. Un `D` fijo que
acepte cardinalidad par acepta los extremos y rechaza el medio. La región
compatible con «aceptar» tiene un hueco. Este ejemplo no propone una decisión
jurídica; refuta la suficiencia matemática de fijar cualquier `D`.

La retícula de particiones ya implementada en el harness organiza hipótesis por
sus predicciones sobre casos. No genera configuraciones ni reemplaza el orden
de generalidad. Una función que admite todos los casos y otra que excluye todos
inducen la misma partición de un bloque, pero resultados opuestos. Se necesitan
etiquetas de salida y realizaciones tipadas conservadas.

La aciclicidad y antiintercambio de la teoría de referencia no se heredan por
cualquier candidato. Una regla puede introducir ciclos. Esa propiedad se
comprueba en la teoría hipotética y el universo ground declarado; no se utiliza
como filtro de adecuación jurídica.

## 10. Qué aporta Sales y qué no cubre el fragmento

`sells` es una relación nueva declarada con identidad propia:
`sells(Person, Thing, Person)`, asociada a la regla `givingProgram`.
El trigger ground aporta tres argumentos y su tiempo; la regla `I <= give @
after t` deriva el efecto con ese trigger incluido en la provenance. El cuerpo
no reconstruye por sí solo obligaciones ni calificaciones de compraventa.

`sale` es otro Seeded, con nueve argumentos explícitos en `Sales`.
`normalSaleProgram` agrupa `normalSale`, `reciprocalTransferPermission` y
`plainTransferObligation`. `retentionSaleProgram` comparte la regla de permiso
y utiliza `paidRetentionObligation`. `obligationCreatesRelation` permanece en
el entorno general. Los bindings del programa coordinan vendedor, comprador,
objeto, autoridad, vigencia e índice de expediente.

La representación y ejecución de esa composición ya están implementadas.
La elección de las relaciones y reglas fue realizada por el programador.
El fragmento de once verbos no expresa literalmente esos programas porque sus
cuerpos usan `sale`, `selectedReciprocity`, `paymentFor` y la conducta `pay`.
Reproducirlos exigiría ampliar el manifiesto de vocabulario o definir y verificar
una traducción; no se declara una equivalencia automática.

Del mismo modo, Riggs necesita relaciones sucesorias, adquisición, identidad
del beneficiario y vínculos causales que no están en este fragmento. La
restricción inicial sirve para investigar un espacio enumerable; no certifica
que represente toda compraventa o sucesión ni redefine esos expedientes.

## 11. Correspondencia y verificación

| Objeto | Implementación concreta | Estado |
|---|---|---|
| Signaturas, categorías, campos | `typed::Verb`, `Arg`, `Type`; `elab::lower_rule`, `check_rule_event` | IMPLEMENTED + inventario finito verificado |
| Identidad temporal y constructores | `typed::TimeExpr::{At,Var,After}`, `Term`, conversiones estructurales | IMPLEMENTED |
| Cláusula individual | `typed::Rule`, `RuleBody` | IMPLEMENTED |
| Programa y bindings | `typed::Program`, `RuleUse`; `programs::lower_program`, `specialize_program`, `compile_program` | IMPLEMENTED |
| Aplicación parcial | Sustitución de prefijo de constantes; parámetros pendientes conservados | IMPLEMENTED; tests existentes |
| Activación y fixpoint | `derive::close_program`, `prepare_programs`; `IncrementalProgramClosure` | IMPLEMENTED; termina bajo condiciones del programa |
| Oráculo de comparación | `derive::close_program_reference` | IMPLEMENTED |
| Efectos de estado | `runtime::execute_effect` | IMPLEMENTED; separado de cierre |
| Procedencia | `Module::origins`, `DerivationRecord`, `program_rule_origin` | IMPLEMENTED |
| Fronteras de eventos | `legal_geometry_tests::finite_frontiers`, `insert_minimal` | HARNESS; certificados finitos |
| Particiones y meet | `legal_geometry_partition_tests`, `legal_geometry_conversation_tests` | HARNESS; resultados finitos |
| Gramática acotada y perfiles | Este contrato y manifiesto | PROPOSED CONTRACT |
| Enumerabilidad | Argumento del apartado 5 | THEOREM bajo cotas explícitas |
| Subsunción simbólica de candidatos | No existe comparador implementado | THEOREM del fragmento; implementación pendiente |
| Operadores mínimos completos | No existen | Pendiente de prueba y de implementación |
| Filtros convexos de decisión efectiva | Contrato `D` todavía incompleto | Pendiente |

Test individual vuelto a ejecutar:

```text
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 python3 scripts/test_safe.py \
  --test-target cli \
  --test cli::tests::legal_geometry::correspondence_and_typed_inventory
```

Resultado: 1 test aprobado, 0 ignorados. Ejecución 1,2222 s; pico RSS 15,55 MiB.
Compilación incremental 0,1671 s; pico RSS 25,17 MiB. El runner verificó los
cgroups con 1 GiB total, sin swap, un trabajo de Cargo y un hilo. El primer
intento sin acceso a systemd se detuvo por fallo de aislamiento; el reintento
autorizado conservó los controles. No se ejecutó la suite completa.

La extracción JSON selecciona del inventario elaborado actual los once verbos
y los tipos/constantes generales. Las constantes sintéticas y las siete semillas
del dominio inicial se transcriben de la fixture fuente que ese test carga y
elabora; el exportador existente omite esas declaraciones experimentales.
Se identifica expresamente esa diferencia de procedencia. Conserva el registro
tipado original y hashes de sus fuentes. Es una exportación de datos, no un nuevo
parser del lenguaje ni un generador de candidatos.

## 12. Orden de trabajo autorizado por el contrato

1. Validar el alcance del fragmento y fijar la interfaz de salida y el significado
   jurídico del contrato `D` para el ciclo concreto.
2. Contrastar las operaciones de generalidad con patrones tipados, incluyendo
   constantes, identificaciones, constructores y condiciones de activación.
3. Demostrar o refutar completitud de los operadores para el presupuesto elegido.
4. Comprobar los filtros inducidos por valoraciones efectivas. Si no son convexos,
   conservar una representación capaz de registrar sus huecos.
5. Implementar sólo entonces la exploración del harness y decidir si las
   fronteras de Candidate Elimination son una representación apropiada.

La finitud del catálogo, su ejecutabilidad y su adecuación normativa son tres
obligaciones diferentes. Este contrato define las dos primeras como objeto de
investigación y mantiene la tercera sometida a valoraciones fundamentadas.
