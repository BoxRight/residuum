# Contrato de experiencia recuperable: memoria, aplicabilidad y comparación contrafáctica

Estado: **diseño, versión 2, no congelado**. No hay implementación. No modifica
código, motor, LegalGeneral, contratos congelados ni registros de 004-R.

Parte de dos documentos:
- `semantic-counterfactual-learning-feasibility.md` (en adelante, *informe de
  viabilidad*);
- la revisión del usuario de la versión 1, que la aprueba como base y pide
  incorporar cinco distinciones antes de implementar (§16 resume los cambios).

Objetivo del siguiente avance: cuando llega un expediente nuevo, el sistema
recupera experiencias anteriores para orientarse, comprueba si sus
formalizaciones se aplican, las contrasta mediante Residuum y, cuando
encuentra una insuficiencia, construye mundos que permiten investigar qué
diferencias importan. Después conserva lo aprendido.

Fuera de alcance: ampliar LegalGeneral, buscar casos nuevos, medir
rendimiento, cambiar el motor, adoptar una semántica modal completa.

## 0. Resumen

$$
E_i=(S_i,\ \Gamma_i,\ R_i,\ N_i,\ d_i,\ z_i,\ \mathcal H_i)
$$

- **Núcleo inmutable** `(S_i, Γ_i, R_i, N_i)`. Se fija al cerrar la visita a
  un expediente documentado y se identifica por contenido. `N_i` contiene solo
  consecuencias sobre la formalización documental `Γ_i`.
- **Anotaciones versionadas** `(d_i, z_i)`. Nunca entran en Γ, en la clausura
  ni en el verificador.
- **Historia** `𝓗_i`, solo de añadido.
- **Mundos hipotéticos** `W`. Son objetos **separados** de las experiencias,
  construidos por intervención sobre un `Γ_i`. Pueden recuperarse como
  antecedentes de investigación, nunca como expedientes.

Reglas que sostienen el resto:

1. **Recuperar no es aceptar.** Una hipótesis recuperada se readmite y se
   reevalúa en el expediente nuevo. Su estado anterior no se transfiere.
2. **Semejanza no es aplicabilidad.** Los embeddings proponen candidatos; la
   aplicabilidad de una formalización al expediente nuevo se comprueba
   simbólicamente (§7).
3. **El embedding no decide un contrafactual.** Propone qué mundos o
   experiencias examinar. Qué se sigue en cada mundo lo decide la teoría; qué
   mundos son pertinentes lo decide la pregunta (§8).
4. **Revisar una descripción no es revisar una formalización.** La primera
   cambia `d` y `z`. La segunda crea una `Γ'` con identidad propia y enlace a
   la anterior (§11).
5. **Disciplina prequencial** (§6.2).

## 1. Alcance mínimo de D1-D6

| | Decisión mínima | Consecuencia en el contrato |
|---|---|---|
| D1 | **Los curadores formalizan los hechos**, como en 004-R. El aprendiz puede *proponer* una nueva formalización, que queda registrada y sin efecto hasta que un curador la acepta. | `Γ_i` tiene linaje (`revises`). El diseño no impide que más adelante el aprendiz formalice (§11.2). |
| D2 | Intervenciones `δ=(D,A)` sobre constantes y verbos sembrados del vocabulario. Un mundo con `inconsistent` se **marca**, no se rechaza. Los grupos de semillas que solo se intervienen juntos los declara el curador. | Objeto `W` separado (§4) y pregunta `q` con hechos fijos y variables (§8). |
| D3 | Seis categorías: no aplicabilidad, insuficiencia deductiva, insuficiencia representacional, discrepancia valorativa, dependencia y fuera de alcance (§9). Solo algunas se calculan. | Registro de señales con evidencia en `𝓗`. |
| D4 | Unidad almacenada = experiencia completa. Embeddings solo para **recuperar candidatos**. Aplicabilidad simbólica. Línea base estructural sin embeddings. Proveedor sin fijar. | §7. |
| D5 | Los contrastes se nombran "dependencia bajo T". Sin lenguaje causal. | Comprobación léxica en descripciones (§6.3). |
| D6 | No se buscan casos nuevos. Con el corpus actual, la discrepancia valorativa solo puede aparecer en Brookside. | Ninguna. |

## 2. Objetos que no se mezclan

| Objeto | Qué es | Quién lo produce | ¿Puede recuperarse como experiencia? |
|---|---|---|---|
| Expediente documentado `S` | hechos establecidos del caso, con fuentes | corpus | sí, dentro de `E` |
| Formalización `Γ` | semillas Formal de `S`, con linaje | curador (aprendiz: propuesta) | sí, dentro de `E` |
| Hipótesis `h`, `Γ_h` | reglas candidatas con estado | aprendiz | sí, dentro de `R` de `E` |
| Mundo hipotético `W` | `Γ` intervenida por `δ`, sin respaldo documental | aprendiz o harness, por una pregunta | **no**: solo como antecedente de investigación, etiquetado |
| Consecuencias | clausuras, evaluaciones, soportes, diferencias | Residuum | solo junto al objeto sobre el que se calcularon |
| Valoración `ε` | juicio externo sobre una posición, con fuente | scorer o fuente externa | solo si fue observada (P3) |
| Descripción `d`, embedding `z` | texto y vector sobre una experiencia | renderizador o LLM, embedder | no; son índices |

Un mundo nunca adquiere estatus documental, aunque coincida con las semillas
de otro expediente. Si coincide, se registra la coincidencia como relación,
no como identidad.

## 3. Componentes de una experiencia

### 3.1 Notación y granularidad

En la Etapa 0 `Γ` era la configuración de una hipótesis. Aquí `Γ_i` es la
formalización del expediente, y la de una hipótesis se escribe `Γ_h`, con
`id(Γ_h)` de la Etapa 0 §2.2.

Una experiencia es **una visita a un expediente dentro de una trayectoria**.
Dos trayectorias sobre Casa Fiesta dan dos experiencias con el mismo `S` y la
misma `Γ`, y distinto `R`.

### 3.2 `S_i`: expediente documentado

- referencia al corpus (`pilot-004-documentary-corpus.md`) y grado de la
  fuente;
- texto de hechos establecidos, sin alegaciones;
- `id(S_i)=SHA256(texto canónico ‖ referencias)`.

No contiene valoraciones.

### 3.3 `Γ_i`: formalización

$$
\Gamma_i=(\tau,\ \Sigma_i,\ G_i,\ a_i,\ \rho_i)
$$

- `τ`: identidad de la teoría de referencia (Etapa 0), con el vocabulario.
- `Σ_i`: semillas en Formal elaborado, canonicalizadas con `formal_event`,
  ordenadas y sin duplicados.
- `G_i`: grupos de semillas que solo se intervienen juntas.
- `a_i`: autor y estado (`curada`, `propuesta`, `aceptada`, `rechazada`).
- `ρ_i`: linaje. `revises: id(Γ)` anterior, si lo hay, y el motivo.

`id(Γ_i)=SHA256(τ ‖ canon(Σ_i) ‖ canon(G_i))`. Autor, estado y linaje no
entran en la identidad: dos formalizaciones iguales tienen la misma `id`,
vengan de donde vengan.

Un expediente sin semillas representables (Flowers) tiene `Σ_i=∅` y produce
experiencia igualmente.

### 3.4 `R_i`: hipótesis asociadas

Todas las hipótesis de la trayectoria durante la visita, con su estado al
cerrar:

- `id(Γ_h)`, texto, nombre de regla, racional;
- estado: `committed`, `draft`, `discarded` (razón), `retired` (momento),
  `rejected_at_admission` (etapa);
- `parent` si es revisión; `reused_from=(id(E_j), id(Γ_h'))` si llegó por
  recuperación;
- `R_k` al entrar y al salir.

Hoy en `trajectory-report.json` (`hypotheses[]`, `commit_traces`); faltan
`id(Γ_h)` y `reused_from`.

### 3.5 `N_i`: consecuencias sobre `Γ_i`

Solo sobre las semillas documentales:

- `C_O(Σ_i)` y `C_{O∪R_k}(Σ_i)`;
- por evaluación: base `B`, `added_events`, justificaciones, soportes con
  hojas = semillas, certificación y omisiones;
- aplicabilidad de cada hipótesis de `R_i` (§7.2);
- contrastes entre hipótesis (particiones, regiones discriminantes) **sobre
  `Σ_i`**;
- huellas del motor y de `τ`.

Las consecuencias en mundos van con el mundo (§4). `N_i` solo los enlaza.

### 3.6 `d_i` y `z_i`

| | Determinista, no se revisa | Interpretativa, versionada |
|---|---|---|
| **Hechos** | `f^S`: renderizado de `Σ_i` | `h^S` |
| **Formalización con resultado** | `f^Γ`: por hipótesis, `canon`, verbos leídos y escritos, aplicabilidad, señales | `h^Γ` |

Versiones interpretativas: `(texto, autor, versión, momento, afirmaciones[])`,
con procedencia (`formal`, `consecuencia`, `valoración`, `hipótesis`) como en
el contrato de revisión §3. Cada `z` es `(vector, embedder, versión,
dimensión, sha256(texto))`, uno por versión y por vista.

### 3.7 `𝓗_i`: historia

| Tipo | Contenido | Autor |
|---|---|---|
| `closed` | cierre del núcleo | harness |
| `valuation_observed` | valoración y momento | scorer |
| `scored` | pérdida, faltantes, excesos | scorer |
| `signal` | categoría y evidencias (§9) | harness, scorer o aprendiz según la categoría |
| `world_built` | `id(W)` construido desde esta experiencia y su pregunta | harness |
| `description_revised` | versión nueva, motivo, aceptada o rechazada | LLM más validador |
| `formalization_proposed` | `id(Γ')` propuesta, motivo, estado | aprendiz, curador |
| `retrieved` | por qué consulta, vista, puntuación, rango, `used`/`unused` | harness |
| `applicability` | nivel alcanzado en el expediente nuevo (§7.2) | harness |
| `reuse_outcome` | resultado de la reutilización (§11.3) | harness |
| `replayed` | reproducción de `N_i` | harness |

## 4. Mundos hipotéticos

$$
W=(\operatorname{id}(\Gamma_\text{base}),\ q,\ \delta,\ \Sigma',\ N_W,\ m)
$$

- `id(Γ_base)`: la formalización documental de partida. Un mundo siempre
  parte de una formalización documentada, no de otro mundo. Las
  intervenciones compuestas se escriben como un solo `δ`.
- `q`: la pregunta que lo motivó (§8.1). Un mundo sin pregunta no se
  construye.
- `δ=(D,A)`, `Σ'=(Σ_base∖D)∪A`, respetando `G_base`.
- `N_W`: para cada teoría evaluada, `C_T(Σ')` y su diferencia con
  `C_T(Σ_base)`; soportes si se pidieron.
- `m`: marcas. Siempre `hipotético`; además `inconsistent` si aplica.
- `id(W)=SHA256(id(Γ_base) ‖ canon(δ))`. La pregunta no entra en la
  identidad: dos preguntas pueden examinar el mismo mundo.

Un mundo puede recuperarse en una investigación posterior como
**antecedente**: "en una experiencia anterior se examinó qué pasaba si…".
Se presenta con la marca `hipotético` y nunca alimenta la vista de hechos de
la recuperación.

## 5. Identidad e inmutabilidad

$$
\operatorname{id}(E_i)=\mathrm{SHA256}\big(\operatorname{id}(S_i)\ \|\ \operatorname{id}(\Gamma_i)\ \|\ \operatorname{canon}(R_i)\ \|\ \operatorname{hash}(N_i)\ \|\ \text{trayectoria}\ \|\ \text{posición}\big)
$$

- El núcleo no se edita. Un cambio de `R_k` o de `Γ` produce otra
  experiencia en la siguiente visita.
- `d`, `z` y `𝓗` se direccionan por `id(E)`.
- Relaciones calculadas (mismo `S`, misma `Γ`, `Γ'` revisa `Γ`, hipótesis
  α-equivalentes, `reused_from`, mundo con las mismas semillas que otro
  expediente) son aristas. No fusionan identidades.

## 6. Invariantes

### 6.1 Separación

| | Invariante |
|---|---|
| I1 | `d` y `z` no entran en `Γ`, `Γ_h`, ninguna clausura ni el verificador. |
| I2 | Revisar `h^S` o `h^Γ` no cambia `id(S)`, `id(Γ)`, `R` ni `N`. |
| I3 | Recuperar no cambia `R_k`; lo recuperado se readmite y se reevalúa. |
| I4 | Toda consecuencia se refiere a una teoría y a unas semillas concretas. |
| I5 | Las valoraciones se observan con fuente; no se infieren. |
| I6 | Un mundo `W` no es un expediente: no tiene `S`, no recibe valoraciones como si fuera un caso, no puntúa y no entra en la vista de hechos. |
| I7 | Una formalización propuesta no sustituye a la vigente hasta que un curador la acepta (D1). Aceptarla no edita experiencias pasadas. |

### 6.2 Disciplina prequencial

En el expediente `k` de la trayectoria `T`:

- **P1.** Solo son recuperables experiencias y mundos cerrados antes del
  inicio de `k`.
- **P2.** De cada uno, solo las versiones de `d` y `z` anteriores al inicio
  de `k`.
- **P3.** Una valoración solo es visible después de su `valuation_observed`.
  Nunca se lee `reserved/`.
- **P4.** Lo que se escribe sobre `k` antes de su scoring no contiene
  valoraciones de `k`. La versión posterior se marca `post_feedback` y solo es
  visible para expedientes posteriores.
- **P5.** Si la memoria contiene experiencias de los mismos expedientes, se
  declara en el manifiesto y ninguna pérdida medida con ella se presenta como
  prospectiva.

### 6.3 Admisión de descripciones

- `h^Γ`: condiciones R1-R6 del contrato de revisión §4. Sobre todo R3: ninguna
  constante ground.
- `h^S`: puede mencionar constantes de su propio `Γ_i`; no valoraciones no
  observadas. Se mide con el control de vocabulario opaco.
- Todas: sin lenguaje causal (D5) ni autovaloración (R4).

## 7. Recuperación y aplicabilidad: dos relaciones distintas

### 7.1 Semejanza: qué experiencias se parecen

$$
\operatorname{sim}_S(S_k,S_i)
$$

Solo propone candidatos. Vistas:

| Vista | Puntuación | Uso |
|---|---|---|
| hechos | `cos(z^S_k, z^S_i)` | casos que se cuentan parecido |
| formalización | `cos(z^S_k, z^Γ_i)` | formalizaciones descritas como útiles para hechos así |
| estructural (sin embeddings, control) | Jaccard entre verbos con polaridad de `C_O(Σ_k)` y de `C_O(Σ_i)` | casos con el mismo material formal |

Devuelve las `m` mejores experiencias por vista, con empates por id. Todas
se anotan como `retrieved`, usadas o no.

La semejanza global es una **heurística de recuperación**. No es el orden
adecuado para un contrafactual concreto (§8.3).

### 7.2 Aplicabilidad: qué formalización puede aplicarse

$$
\operatorname{app}(\Gamma_h,\ \Sigma_k\mid B)
$$

Se comprueba simbólicamente, con niveles acumulativos, para cada hipótesis de
una experiencia recuperada:

| Nivel | Condición | Cómo se calcula hoy |
|---|---|---|
| `A0` firmas | todo verbo con polaridad del cuerpo de `h` aparece en `C_B(Σ_k)` | comparación de verbos sobre la clausura |
| `A1` átomos | cada átomo del cuerpo tiene alguna instancia en `C_B(Σ_k)` por separado | unificación átomo a átomo |
| `A2` disparo | existe una sustitución conjunta del cuerpo: la regla produce al menos una justificación | evaluación de 004-R (`contributed_justifications ≠ ∅`) |
| `A3` aporte | además añade eventos que `B` no derivaba | `added_events ≠ ∅` |

Si no se alcanza un nivel, se guarda **qué falta**: los verbos ausentes
(`A0`), los átomos sin instancia (`A1`) o, para un objetivo concreto, los
requisitos residuales (`begin_residual` del worker, que ya existe).

El primer nivel no alcanzado es la señal de **no aplicabilidad** (§9). No
dice nada sobre si la formalización del expediente es buena.

Ejemplo de la simulación: h1 sobre Casa Fiesta no alcanza `A0` (falta
`obligation`); h3 sobre Casa Fiesta alcanza `A3`; h3 sobre Brookside
alcanza `A0` pero no `A1` (no hay ningún `Act` de `adoptPolicy` ni de
`post`).

### 7.3 Qué ve el aprendiz

Por cada experiencia recuperada:

- `f^S`, `h^S`, `f^Γ`, `h^Γ` visibles según P2;
- `R_i` con estados, razones y señales;
- la **aplicabilidad de cada hipótesis en el expediente nuevo**, ya calculada,
  con lo que falta;
- mundos examinados desde esa experiencia, marcados `hipotético`;
- valoraciones solo según P3;
- la etiqueta "experiencia anterior: hipótesis a reevaluar, no respuesta".

## 8. Comparación contrafáctica

### 8.1 La pregunta

Siguiendo la idea de Lewis de que evaluar un contrafactual requiere elegir
los mundos pertinentes, sin adoptar su semántica:

$$
q=(\Sigma,\ T,\ \alpha,\ \beta,\ F_q,\ V_q)
$$

- `Σ`: semillas documentales de partida.
- `T`: la teoría bajo examen (`O`, `O∪R_k`, `B∪{h}`).
- `α`: el antecedente, como cambio de hechos: un conjunto de átomos que deben
  estar y otro que no deben estar.
- `β`: el consecuente que se pregunta, un evento o una posición.
- `F_q`: hechos que la pregunta **mantiene fijos**.
- `V_q`: hechos que la pregunta **permite variar**, además de los de `α`.

`F_q` y `V_q` son lo que vuelve la pregunta contextual: qué diferencias deben
mantenerse y cuáles pueden variar. Los propone el aprendiz y los valida el
harness con D2 (grupos `G`, verbos sembrados, marca `inconsistent`).

### 8.2 Mundos pertinentes

Un mundo `W` con `δ=(D,A)` es **admisible para `q`** si:

- cumple `α`;
- no toca `F_q`;
- solo toca átomos de `α` y `V_q`;
- respeta los grupos `G` (si toca un miembro, toca todo el grupo).

Proximidad: `W ≤_q W'` si `D ⊆ D'` y `A ⊆ A'`. Es un **orden parcial** por
inclusión del cambio, no un orden total de semejanza global. Los mundos
pertinentes son los minimales admisibles.

Lewis usa un orden de semejanza total entre mundos. Aquí basta un orden
parcial porque los mundos son finitos y se construyen; es una elección del
contrato, no una tesis sobre Lewis.

Sobre esos mundos, con `T` monótona:

- **"sería"**: `β` se deriva en todos los mundos pertinentes;
- **"podría"**: en alguno;
- **"no"**: en ninguno.

Los tres se calculan con un solo cálculo de soportes sobre el pool
`Σ ∪ A_max`: `β ∈ C_T(Σ')` si algún soporte mínimo de `β` está contenido en
`Σ'`. Es el argumento del informe de viabilidad §4.4. Si `T` deja de ser
monótona, esto ya no vale y hay que cerrar cada mundo por separado.

Un resultado se enuncia siempre como **dependencia bajo `T`**, relativo a
`F_q` y `V_q` (D5).

### 8.3 Qué papel tienen los embeddings

- **No deciden** si `β` se sigue: lo decide la clausura.
- **No deciden** qué mundos son pertinentes: lo deciden `α`, `F_q`, `V_q` y
  `G`.
- **Proponen** de dónde sacar la pregunta: experiencias y mundos anteriores
  que se parecen, cuyas preguntas y diferencias pueden sugerir qué fijar y qué
  variar. Esa sugerencia se registra como origen de `q`.

La semejanza condicionada por la pregunta, `sim(S_i, S_j | q, T)`, queda
reconocida y **no se implementa** en la primera iteración. Una aproximación
simbólica posible y sin embeddings: comparar dos expedientes solo en los
átomos que aparecen en algún soporte mínimo de `β` bajo `T`. Dos casos
globalmente parecidos pueden diferir justo ahí, y dos casos lejanos pueden
coincidir.

### 8.4 Ejemplo de la simulación, reescrito como preguntas

| `q` | `α` | `F_q` | Resultado bajo `O∪{h3}` |
|---|---|---|---|
| ¿Seguiría la obligación de cfA1 sin el acuerdo de usa? | quitar `Act(saCF_u, usa, agree, …)` | actos cfA1, cfA2, acuerdo de casaFiesta | no: se pierden las cuatro consecuencias |
| ¿Habría violación si cfA1 no se hubiera hecho? | reemplazar `Act(cfA1, …)` por `~Act(cfA1, …)` | acuerdos | no: tampoco hay obligación, porque h3 la deriva del acto |

La segunda es el hallazgo "obligación que solo existe si se cumple". Bajo
este contrato es una dependencia bajo `O∪{h3}` con `F_q` declarado. Decidir
si es un defecto sigue siendo un juicio jurídico.

## 9. Categorías de señales

| Categoría | Qué significa | ¿Se calcula? | Evidencia |
|---|---|---|---|
| **No aplicabilidad** | los antecedentes de `h` no encuentran correspondencia en `Σ_k` | sí: primer nivel `A0`-`A3` no alcanzado (§7.2) | lo que falta |
| **Insuficiencia deductiva** | `β` es expresable en `Γ_k` (verbo del vocabulario, constantes de `Σ_k`), es **buscada**, y `β ∉ C_T(Σ_k)` | sí, cuando hay un `β` buscado: por una pregunta, por el aprendiz o por una valoración observada | requisitos residuales de `β` |
| **Insuficiencia representacional** | un contenido relevante de `S_k` no puede expresarse fielmente con las firmas y composiciones disponibles | **no**: no se puede determinar inspeccionando una clausura | afirmación del aprendiz o del curador, con el pasaje de `S_k` y el intento de expresión; queda abierta hasta revisión |
| **Discrepancia valorativa** | una predicción contradice una valoración observada | sí, por el scorer, solo tras el commit | entrada `scored` |
| **Dependencia** | un mundo pertinente cambia las consecuencias de `h` | sí, desde `N_W` | `q`, `δ` y la diferencia |
| **Fuera de alcance** | la evaluación se rechaza o no se analiza | sí | motivo literal |

Notas:

- No aplicabilidad no es insuficiencia de nadie. Dice que esa regla no habla
  de ese caso. h1 en Casa Fiesta es no aplicabilidad, no un defecto de la
  formalización de Casa Fiesta.
- Insuficiencia deductiva requiere un objetivo. Sin un `β` buscado no se
  declara.
- Una señal representacional puede desembocar en una propuesta de nueva
  formalización (§11.2).
- Dependencia es información para juzgar, no una insuficiencia por sí misma.

## 10. Ciclo

1. **Recuperar candidatos** por semejanza (§7.1).
2. **Comprobar aplicabilidad** de cada hipótesis recuperada en el expediente
   nuevo (§7.2).
3. **Elegir y readmitir** `h'` con `reused_from`. Si el aprendiz la adapta,
   otra `id(Γ_h')`, mismo `reused_from`.
4. **Evaluar** con `B=O∪(R_k∖{h'})` y soportes con hojas = semillas (la
   evaluación de 004-R, no la del worker).
5. **Señales** (§9).
6. **Preguntar** si hay algo que investigar: formular `q`, construir los
   mundos pertinentes, calcular "sería / podría / no" (§8).
7. **Decidir** sobre `h'`: commit, descarte con razón, revisión o borrador.
8. **Proponer una formalización nueva** si hubo una señal representacional
   (§11.2). Opcional.
9. **Cerrar** `E_k` y los `W` construidos; anotar `retrieved`,
   `applicability` y `reuse_outcome` en las experiencias de origen.
10. **Describir** `E_k` (`pre_scoring`, luego `post_feedback`).
11. **Revisar descripciones pasadas** si la reutilización enseñó algo (§11.1).

## 11. Revisión

### 11.1 Revisar descripciones

Cambia `d` y `z` de una experiencia pasada. Pasa §6.3. No cambia nada más.

Sirve para corregir **qué se recupera**. Por ejemplo, tras el fallo de h1 en
Casa Fiesta: "lee `obligation`; no aplica a expedientes con solo actos".

### 11.2 Revisar formalizaciones

Si el aprendiz encuentra que `Γ_i` omite algo que sí puede expresarse con las
primitivas existentes, cambiar la descripción no lo arregla. Entonces propone
`Γ'`:

- con las mismas `τ` y vocabulario: ninguna firma nueva;
- `ρ'=(revises: id(Γ_i), motivo, señal que la origina)`;
- estado `propuesta` hasta que un curador la acepta o la rechaza (D1).

Una `Γ'` aceptada se usa en visitas futuras y produce experiencias nuevas.
Las experiencias con `Γ_i` siguen en la memoria tal como estaban.

Esto es la vía para que el aprendizaje produzca composiciones nuevas de lo
existente sin tocar la ontología fija. En la primera implementación solo se
registra la propuesta.

### 11.3 Criterio de utilidad, por caso y sin estadística

Cambiar el coseno no es aprender. Para cada recuperación se registra, en
`reuse_outcome`:

1. qué experiencia se recuperó, por qué vista y con qué versión de `d`;
2. el nivel de aplicabilidad alcanzado (`A0`-`A3`);
3. si produjo un **contraste formal informativo**, que es cualquiera de:
   - `added_events ≠ ∅` en el expediente nuevo;
   - un mundo pertinente con diferencia no vacía;
   - una región que discrimina entre la hipótesis recuperada y otra de
     `R_k`;
   - una señal de no aplicabilidad que el aprendiz cita al decidir o al
     revisar una descripción.

Una revisión de descripción se considera **útil en un caso** si, en una
recuperación posterior, la experiencia revisada:
- se recupera donde alcanza `A2` o produce un contraste informativo; o
- deja de recuperarse donde no alcanzaba `A0`.

En esta iteración solo se registra, caso a caso. No se exige mejora
estadística.

## 12. Almacenamiento y reproducción

Formato propuesto:

- `memory/objects/<sha256>.json`: núcleos de experiencia, formalizaciones y
  mundos, direccionados por contenido;
- `memory/annotations.jsonl`: versiones de `d` y `z`;
- `memory/history.jsonl`: entradas de `𝓗`;
- `memory/manifest.json`: `τ`, huella del motor, embedder, trayectorias de
  origen, declaración P5.

La memoria vive fuera de `docs/legal-geometry/pilot-004r/`. Un adaptador lee
004-R y la simulación del informe de viabilidad en solo lectura.

Todo `N_i` y todo `N_W` se pueden recalcular desde sus semillas, la teoría y
la huella del motor. Lo que no se reproduce se marca `stale` y no se recupera
hasta reproducirse.

## 13. Correspondencia con lo que existe

| Componente | Fuente hoy | Estado |
|---|---|---|
| `S_i` | `pilot-004-documentary-corpus.md` | existe; falta `id(S)` |
| `Γ_i` | `dossiers.res`, `vocabulary.res`, `formal_event` | existe; faltan `canon(Σ)`, `G`, linaje e `id(Γ)` |
| `R_i` | `trajectory-report.json` | existe; faltan `id(Γ_h)` y `reused_from` |
| `N_i` | `hypotheses[].evaluations[]` | existe y certificado |
| aplicabilidad `A0`-`A3` | clausuras, `contributed_justifications`, `added_events` | ingredientes existen; falta la función |
| requisitos residuales | `begin_residual` del worker | existe |
| mundos `W` y preguntas `q` | `contrast/`, simulación | ejecutado fuera del arnés; faltan `propose_world`, `F_q`, `V_q` |
| "sería / podría / no" | `frontiers` sobre un pool | existe para teorías monótonas |
| `f^S`, `f^Γ` | `formal_event`, plantilla de `generate_catalog.py` | piezas existen |
| `h^S`, `h^Γ` y validador | contrato de revisión §3-4, `claims.py` | especificado; `claims.py` tiene el defecto de `contains_event` |
| `z` y recuperación | Etapa 0 §2.1 | no existe |
| `𝓗` | ledger, prequential y findings de 004-R | existe dentro de una trayectoria |

## 14. Verificación de una implementación futura, sin modelo

Con la memoria inicial construida desde run-013 y la simulación:

1. **Identidad estable.** Construir la memoria dos veces da los mismos ids y
   bytes.
2. **Reproducción.** Cada `N_i` y `N_W` se recalcula y coincide.
3. **I2.** Una versión nueva de `h^Γ` cambia el rango de recuperación en un
   caso construido y deja iguales `id(Γ)`, `R` y `N`.
4. **I6.** Un mundo con las mismas semillas que un expediente no aparece en la
   vista de hechos.
5. **Prequencialidad.** P1-P4 filtran lo que deben; nada de `reserved/`
   aparece en la memoria.
6. **Aplicabilidad.** h1 en Casa Fiesta: no alcanza `A0`, falta `obligation`.
   h3 en Casa Fiesta: `A3`. h3 en Brookside: `A0`, no `A1`.
7. **Preguntas.** Las dos de §8.4 dan "no" con los `F_q` indicados, y
   coinciden con cerrar cada mundo por separado.
8. **Propuesta de formalización.** Una `Γ'` propuesta queda `propuesta`, con
   `revises`, y no altera ninguna evaluación hasta aceptarse.

## 15. Lo que sigue abierto

- **Proveedor de embeddings.** API (pagada, la ejecutaría el usuario) o modelo
  local dentro de 1 GiB.
- **`h^S` y constantes.** Permitir las del propio expediente o describir solo
  por roles.
- **Quién formula `F_q` y `V_q`.** Propuesta: el aprendiz, validado por D2.
  Alternativa: plantillas fijas por tipo de intervención.
- **Semejanza condicionada.** Si la aproximación simbólica de §8.3 entra en la
  primera iteración o se deja para después.
- **Memoria inicial y P5.** Construirla desde 004-R sirve para verificar el
  mecanismo, no para medir aprendizaje.
- **Descripciones `h` de las experiencias de 004-R.** Escribirlas
  retrospectivamente requiere una llamada al modelo; si no, quedan solo con
  `f`.

## 16. Cambios respecto a la versión 1

| Distinción pedida | Cambio |
|---|---|
| No aplicabilidad frente a insuficiencia representacional | §9: la ausencia de un verbo del cuerpo pasa a ser **no aplicabilidad**. La insuficiencia representacional se redefine como no calculable desde la clausura. La deductiva exige un objetivo buscado. |
| Semejanza frente a aplicabilidad | §7: `sim_S` solo propone candidatos; `app` se comprueba con niveles `A0`-`A3` y registra qué falta. La vista estructural compara ahora hechos con hechos. |
| Semejanza global frente a condicionada | §8: pregunta `q` con `F_q` y `V_q`, mundos pertinentes por un orden parcial de cambio mínimo, "sería / podría / no". La semejanza global queda como heurística de recuperación. |
| Experiencias frente a mundos | §2, §4 e I6: los mundos salen de `N_i` y son objetos separados, marcados `hipotético`, nunca expedientes. |
| Revisión de descripciones frente a formalizaciones | §11.1-11.2 e I7: `Γ` tiene linaje y el aprendiz puede proponer `Γ'`, aceptada por un curador. |
| Criterio de utilidad | §11.3: registro por caso de recuperación, aplicabilidad y contraste informativo. |
