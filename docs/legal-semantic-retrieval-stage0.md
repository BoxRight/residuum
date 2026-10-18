# Etapa 0. Nodos Formal–semánticos: inventario y contrato

Estado: **INFORME DE CONTRATO**. No se ha implementado ningún código, ningún
embedding, ninguna recuperación ni ningún ciclo LLM, y no se ha hecho ninguna
llamada a modelos.

Rama: `experiment-legal-order-geometry` (worktree `/tmp/residuum-legal-order-geometry`,
HEAD `dacf1d4`). Se han consultado `experiment-llm-normative-exploration-spec`
(`8edcc30`, explorador y Piloto 003) y `main` (`7af8d39`).

Marcas usadas: **EXISTE** (código presente, con su ubicación), **TEST** (lógica
que solo existe dentro de los tests), **PROPUESTO** (contrato de este informe) y
**ABIERTO** (decisión que no corresponde al harness).

## 1. Hallazgos que condicionan el contrato

1. **Las ramas no coinciden.**
   - Esta rama tiene la historia temporal certificada, pero no tiene sesiones
     paso a paso (`src/residual/steps.rs`, `src/derive/steps.rs`), ni bindings
     de consulta (`src/residual/bindings.rs`), ni el trabajador
     (`src/experiments/{worker,proposal,exploration}.rs`, `boundary/`).
   - La rama del explorador tiene todo eso, pero no tiene `e5fbecb..dacf1d4`.
   - `git merge-tree` entre las dos ramas no da conflictos textuales. Que
     compilen juntas y pasen sus tests no está comprobado.
2. **Todo el análisis geométrico de esta rama es TEST:** fronteras, bases y
   extremos, particiones, identificabilidad, revisión de regiones,
   recalcitrancia, versiones y generador estructural. La biblioteca solo ofrece
   parseo y elaboración, cierre con provenance y consultas residuales. En la
   rama del explorador, `exploration.rs` sí convierte en biblioteca `install`,
   `frontiers`, `kernel`/`meet`/`finer`, `Partition`, `discriminants`,
   `classify` y `certify_addition`.
3. **No hay identidad canónica de reglas ni de teorías:**
   - no hay `Hash` en el AST tipado;
   - no hay impresor Formal canónico;
   - la identidad de una regla es su nombre (`RuleRef{name}`, `typed.rs:308`;
     `DerivationRecord.rule: String`).

   El único impresor (`render_rule`, `legal_geometry_structural_generation_tests.rs:343`)
   es TEST y solo cubre Horn sin records.
4. **No hay un constructor programático validado.** Los campos del AST son
   `pub`, y los tests que construyen reglas como literales se saltan
   `elab.rs`. La única vía validada es generar texto Formal, elaborarlo
   (`load_formal_module`, o `admit` en la rama del explorador) y leer el AST
   resultante.
5. **El elaborador no comprueba la seguridad de la cabeza** (que toda variable
   de la cabeza esté ligada en el cuerpo). Esa comprobación la hacen los
   validadores de perfil: `classify` emite `S_UNBOUND_HEAD`, sea en el TEST de
   generación estructural o en `exploration.rs:931` del explorador.
6. **El Piloto 003 no puede reutilizarse tal cual.**
   - Su `D` observa `hypotheticalSuccessionImpediment`, y el candidato
     propuesto deriva exactamente esa cabeza. Es la cabeza artificial de
     impedimento que el encargo prohíbe dar al LLM.
   - Tiene una sola valoración documentada (`historicalRiggs`); los otros cinco
     expedientes no tienen $y^*$.
   - El dispatcher descarta `parent` y `move` de `propose_candidate`
     (`dispatcher.py:80`).
   - Las afirmaciones de `finish_episode` no se comprueban, porque la función
     vuelve antes de llegar a esa comprobación (`controller.py:174`).

   Del piloto se puede reutilizar la infraestructura: canal, ledger
   encadenado, presupuestos, reproducción byte a byte y etapas de admisión.
   Su configuración experimental no sirve.
7. **No existe ninguna función de pérdida** en ninguna de las ramas. En el
   explorador la decisión es un booleano `compatible`
   (`exploration.rs:149-168`).

## 2. El nodo Formal–semántico

### 2.1 Definición

Un nodo es un registro con una parte inmutable, que lo identifica, y anotaciones
versionadas:

$$
v=(\Gamma,\ d,\ z,\ \mathcal P,\ \mathcal V),\qquad
\operatorname{id}(v)=\operatorname{id}(\Gamma).
$$

**Γ (configuración formal, inmutable).** PROPUESTO, a partir de
`legal-relational-search-space-contract.md` §3 (Γ = id, parámetros, reglas,
RuleUses, módulo, activación, orígenes), reducido a lo que las etapas 1–6
necesitan:

$$
\Gamma=(\tau,\ R,\ Q,\ \iota)
$$

- $\tau$: identidad de la teoría de referencia. Es el nombre del módulo raíz
  más el SHA-256 del contenido de cada archivo del grafo de módulos que carga
  `load_formal_module`. LegalGeneral original no se modifica.
- $R$: lista finita de cláusulas Horn ordinarias, elaboradas por el elaborador
  real (`Rule` con `RuleBody::HornClause`). Las cabezas pertenecen a firmas
  Derived/Effect ya existentes en $\tau$, y no se declaran verbos nuevos. Así
  se excluye la cabeza artificial, igual que `S_INSTALL`/`P_HEAD_INTERFACE`.
  La especialización (`rule b = r(c)`) entra ya elaborada, con lo que A y B de
  la historia temporal dan la misma $R$.
- $Q$: la interfaz observada, una lista de firmas existentes. Forma parte de la
  identidad de un nodo como *hipótesis evaluada*, no de la regla.
- $\iota$: plan de instalación. En las etapas 1–6 solo existe `add`, es decir,
  $T_\Gamma=T\oplus R$ (`exploration::install`) en un módulo privado. El plan
  `captured(t_k)` (programa más anclaje `I`, certificado en `dacf1d4`) está
  reservado a la adopción histórica y queda fuera de este ciclo (§3).

**d (descripción, versionada, nunca normativa):**
`d = (texto, autor, versión, afirmaciones[])`.
- El autor es un modelo o un renderizador determinista.
- Las afirmaciones apuntan a elementos de Γ (§2.3).
- Un nodo puede tener muchas descripciones.
- `d` no entra en Γ, en Known, en `D` ni en el verificador.

**z (embedding, versionado):**
`z = (vector, embedder, versión, dimensión, sha256(d.texto))`.
- Es un vector por descripción, no por nodo.
- Cambiar de embedder crea otra versión; no cambia el nodo.

**𝒫 (evaluaciones):** registros independientes con clave
`(id(Γ), id(S), τ, presupuesto)`. Cada registro guarda:
- el `Closure` completo: eventos, `DerivedEvent`, `DerivationRecord` con regla,
  antecedentes ordenados y sustitución;
- el estado de completitud;
- el número de rondas.

`derived_at` puede variar de una ejecución a otra; el resto debe reproducirse.

**𝒱 (valoraciones y verificaciones):**
- valoraciones externas $\varepsilon$ con fuente y fundamento (§4);
- resultados que calcula el harness a partir de 𝒫 y 𝒱: pérdida, afirmaciones
  comprobadas y bloques de partición.

Ninguno de estos resultados lo produce el LLM.

### 2.2 Identidad

PROPUESTO. $\operatorname{id}(\Gamma)=\mathrm{SHA256}(\tau\ \|\ \mathrm{canon}(R)\ \|\ Q\ \|\ \iota)$, donde $\mathrm{canon}$:

1. parte del `Rule` **elaborado**, nunca del texto fuente;
2. ignora el nombre de la regla (es una etiqueta);
3. renombra los parámetros por orden de primera aparición (antecedente en
   recorrido izquierda-derecha, después la cabeza) y conserva su tipo;
4. conserva el orden y la repetición de los conjuntos del cuerpo, porque
   determinan el orden de `antecedents` en la provenance;
5. distingue las constantes de los parámetros, aunque tengan el mismo nombre;
6. conserva polaridad, `TimeExpr` (`At` / `Var` / `After` anidado) y records;
7. ordena las cláusulas de $R$ por su forma canónica y elimina duplicados
   idénticos.

Al instalar, cada cláusula recibe el nombre `h_<prefijo de canon>`. El
productor que aparece en la provenance pasa a ser función de la identidad
formal. Esto amplía el contrato de identidad de productor de `dacf1d4` y evita
el error `duplicate symbol` cuando dos candidatas comparten una cláusula.

Hay que validar dos puntos en la Etapa 1:
- **No verificado:** que dos árboles `And` con la misma secuencia de hojas
  producen la misma lista `antecedents`. Si no fuera así, la forma del árbol
  pasaría a formar parte de `canon`.
- La identidad es α-equivalencia sintáctica de cláusulas elaboradas. Es más
  fina que las equivalencias semánticas. Dos nodos con distinta id pueden tener
  igual cierre, igual cobertura o caer en el mismo bloque. Eso se **calcula**
  y se registra como relación entre nodos; nunca fusiona identidades.

Jerarquía, de más fina a más gruesa:

| Relación | Definición | Fuente |
|---|---|---|
| $=_{id}$ | igual `canon` | harness (identidad) |
| $\equiv_{prov}$ | iguales multiconjuntos de justificaciones para todo $S\in A$ | motor (`close_program`) |
| $\equiv_{cl}$ | $C_{T_{\Gamma_1}}(S)=C_{T_{\Gamma_2}}(S)$ para todo $S\in A$ | motor |
| $\equiv_{cov}$ | $Cov_Q$ igual sobre $A$ | motor más proyección $Q$ |
| $\equiv_{D}$ | mismo bloque de la partición de hipótesis inducida por un protocolo $D$ | motor más `kernel` |

Cada relación implica la siguiente:
- con nombres `h_<canon>`, $=_{id}$ implica $\equiv_{prov}$;
- igual provenance con iguales semillas implica igual cierre;
- $\equiv_{cov}$ implica $\equiv_D$ cuando $D$ observa solo $Q$.

Las implicaciones inversas fallan en general. Por ejemplo, dos reglas
sintácticamente distintas pueden dar igual cierre y distinta provenance. Las
cinco relaciones se registran por separado.

### 2.3 Descripción auditable

El LLM produce `texto` y `afirmaciones`. PROPUESTO: cada afirmación es
`{ref, atributo, valor}`, siguiendo el contrato `claims/1` del explorador
(`claims.py`), donde `ref` apunta a un elemento de Γ:
- `head`;
- `body[i]`;
- `body[i].arg[slot]`;
- `param[x]`;
- `time[i]`.

Los atributos verificables son:
- `verb`, `polarity`, `role→binder` y `binder-shared-with`;
- `constant`, `time-pattern` (`@ t` frente a `@ after t`);
- `head-interface`.

El validador comprueba:

- **Corrección:** toda afirmación es verdadera en el Γ elaborado.
- **Cobertura:** cada átomo, cada variable compartida y cada constante de Γ
  aparece en al menos una afirmación.
- **No invención (comprobación léxica):** todo identificador del vocabulario
  de $\tau$ (verbo, constante, entidad) que aparezca en el texto debe estar en
  Γ, y toda afirmación negativa debe tener su átomo `~` en Γ.

La fidelidad semántica de la prosa más allá de esos identificadores queda
**sin verificar**, y así se marca. Los criterios de la Etapa 1 se aplican a las
afirmaciones verificadas, no a la prosa.

## 3. Representaciones y APIs reutilizables

| Necesidad | API | Ubicación | Estado |
|---|---|---|---|
| Elaborar Γ | `parse_formal`, `load_formal_module`, `elaborate_formal` | `formal.rs:39`, `modules.rs:17`, `elab.rs:57` | EXISTE |
| Admisión por etapas (parsed → … → admitted) | `proposal::admit`, `check_scope` | `8edcc30:src/experiments/proposal.rs:228,340` | EXISTE en el explorador; acepta **solo reglas** (`S_DECLARATION`) |
| Validador de perfil (cabeza segura, aridad, conexión) | `exploration::classify`, `classify_candidate` | `8edcc30:exploration.rs:896,1001` | EXISTE en el explorador |
| $T\oplus R$ sin tocar $T$ | `exploration::install`, `theory_preserved`, `certify_addition` | `8edcc30:exploration.rs:36,204,258` | EXISTE en el explorador |
| Cierre con provenance | `close_program`, `IncrementalProgramClosure` | `derive.rs:192`, `derive/program_incremental.rs:35` | EXISTE |
| Cierre acotado, sin certificar ausencia | `ForwardSession::for_module`, `step`, `trace` | `8edcc30` / `main`: `src/derive/steps.rs:33` | EXISTE fuera de esta rama |
| Evaluación de candidata | `EvaluationSession::{begin,step,finish}` | `8edcc30:proposal.rs:440-495` | EXISTE en el explorador |
| Requisitos de un objetivo ausente | `query_active_program_residual`, `begin_active_residual_query_with_bindings` | `residual/active.rs:70`; `8edcc30:residual/bindings.rs:164` | EXISTE |
| Origen de productores | `program_rule_origin`, `Module.origins` | `programs.rs:237`, `typed.rs:2` | EXISTE |
| Fronteras mínimas, particiones, discriminantes | `frontiers`, `kernel`, `meet`, `finer`, `Partition`, `discriminants` | `8edcc30:exploration.rs:417-626` | EXISTE en el explorador; en esta rama solo como TEST |
| Ledger reproducible, presupuestos, canal | `ledger.py`, `episode.py`, `channel.py`, `manifest.py` | `8edcc30:tools/normative_explorer/` | EXISTE |
| Comprobador de afirmaciones | `claims.py` (`claims/1`) | ídem | EXISTE, con los defectos de §1.6 |
| Clave de justificación (productor, antecedentes, sustitución) y proyección | `justification`, `project`, detectores L1–L7 | `temporal_history_tests.rs` | TEST (`dacf1d4`) |
| Generador exhaustivo de referencia | base 12, 10.850 cláusulas `compliance` admitidas | `legal_geometry_structural_generation_tests.rs` | TEST |

**Qué se reutiliza de la historia temporal certificada:**
1. El teorema $\pi_{t_k}\operatorname{Run}_{H_n}\cong\operatorname{Run}_{T_k}(S_k)$.
   Justifica evaluar cada hipótesis en una ejecución aislada de un único
   momento, sin construir nunca $H_n$.
2. El contrato de identidad de productor, con la clave de justificación.
3. Los detectores L1–L7, como precondición de una adopción futura: una
   hipótesis aceptada solo pasará a la historia si cumple localidad.
4. La separación probada por `hypothetical_r1_on_dossier0`.

Adoptar una hipótesis en la historia (convertir $R$ en una plantilla capturada
más un anclaje) queda **fuera** de las etapas 1–7.

## 4. Valoración parcial y pérdida

Una valoración es
$\varepsilon=(S,\ Q,\ P^+,\ P^-,\ \text{fuente},\ \text{fundamento},\ \text{estado})$.

**Condiciones de buena formación:**
- $P^+$ y $P^-$ son conjuntos finitos de eventos ground cuyos verbos están en
  $Q$, con identidad lógica completa (verbo, argumentos, tiempo, polaridad).
- $P^+\cap P^-=\varnothing$.
- Las posiciones no valoradas no cuentan.
- Que $X\in P^-$ no implica $\sim X$, y que $\sim X\in P^+$ es una posición
  evidencial distinta.
- La valoración no entra en Known ni en $S$.
- Solo cuentan las valoraciones cuyo estado es `documentada`. Las pendientes
  no eliminan hipótesis.

**Resultado:** $N_h(S,Q)=\{e\in C_{T_h}(S):\operatorname{verbo}(e)\in Q\}$,
calculado por el motor.

**Ejecución completa:**
$$
\ell(h,\varepsilon)=|P^+\setminus N_h(S,Q)|+|P^-\cap N_h(S,Q)|.
$$

**Ejecución incompleta.** Tras $k$ rondas sin punto fijo, con $N^{(k)}$
creciente: lo derivado ya no se retira, porque el cierre Horn es inflacionario.
Por tanto
$$
\ell\in\big[\,|P^-\cap N^{(k)}|,\ \ |P^-\cap N^{(k)}|+|P^+\setminus N^{(k)}|\,\big].
$$
Las posiciones de $P^+$ que faltan quedan **pendientes**, no ausentes, y se
informa del intervalo. Una ejecución incompleta solo certifica los errores por
exceso.

**Criterio por posición:** cada $e\in P^\pm$ queda como `satisfecha`,
`violada` o `pendiente`. El booleano `compatible` del explorador es el caso
$\ell=0$ con ejecución completa.

**Qué no es la pérdida:**
- no es una medida de adecuación jurídica;
- una pérdida cero sobre los casos conocidos no prueba aprendizaje;
- la pérdida prospectiva se mide sobre expedientes valorados que la hipótesis
  no usó (Etapa 7).

## 5. Vecindad semántica frente a particiones formales

| | Vecindad del embedding | Particiones formales |
|---|---|---|
| Objeto | descripciones $d$ (varias por nodo) | nodos Γ, mediante sus ejecuciones |
| Fuente | modelo $E$ externo, versionado | `close_program` sobre un dominio $A$ y un canal $Q$ |
| Estructura | puntuación $\cos\in[-1,1]$; ni transitiva ni relación de equivalencia | relación de equivalencia ($\ker$); retícula con `meet` y `join` |
| Estabilidad | cambia con el texto de $d$ y con la versión de $E$ | cambia solo con $A$, $Q$ o $\tau$ |
| Garantías | ninguna sobre cierre, cobertura, subsunción ni causalidad | igualdad exacta de predicciones en $A$ |
| Uso permitido | ordenar qué nodos inspeccionar o proponer | filtrar, discriminar y elegir preguntas |

Dos descripciones del mismo nodo pueden quedar lejos, y dos nodos con bloques
distintos pueden quedar cerca. Ambos casos se registran como datos de
evaluación, no como errores del verificador.

La comparación de la Etapa 2 mide si el ranking por $\cos$ concuerda con:
- la proximidad estructural: firma de verbos, cabeza, grafo de incidencia de
  variables compartidas y constantes;
- la proximidad de cobertura: Jaccard sobre $Cov_Q$ en $A$.

La similitud nunca decide un bloque ni una pérdida.

## 6. Interfaces mínimas por etapa

Todas las interfaces viven en la capa experimental (`src/experiments`, `tools/`
o tests). Ninguna modifica el motor ni LegalGeneral.

| Etapa | Interfaz | Base |
|---|---|---|
| 1 | `node(text, τ, Q) → Node \| Rejection{etapas, códigos}` | `admit`, `classify`, elaborador |
| 1 | `canon(rules) → bytes`; `id(Γ)` | **nuevo** (harness; §2.2) |
| 1 | `render(Γ) → d₀` determinista, de referencia | nuevo; extiende `render_rule` |
| 1 | `check_description(d, Γ) → {correctas, incorrectas, sin cubrir, inventadas, no verificables}` | `claims.py` ampliado |
| 2 | `embed(d) → z` con caché por `(embedder, versión, sha256(d))` | nuevo; proveedor ABIERTO |
| 2 | `retrieve(z, k) → [(id, cos)]`, por fuerza bruta sobre un archivo local | nuevo |
| 2 | `structural_neighbors(Γ, k)`, `coverage_neighbors(Γ, A, k)` | `exploration::classify`, cierres |
| 3 | `verify(Γ, S, Q, presupuesto) → Evaluation{closure, provenance, complete}` | `install` + `EvaluationSession` / `ForwardSession` |
| 3 | `loss(Evaluation, ε) → (intervalo, estado por posición)` | nuevo |
| 4 | `discrepancy(Γ, ε) → {faltantes con requisitos residuales, excesos con productores y antecedentes, propiedades del expediente}` | `query_active_program_residual`, provenance |
| 4 | `revise(parent, Γ', motivo) → Node` que exige $\operatorname{id}(\Gamma')\neq\operatorname{id}(\Gamma)$ | ledger; corrige el descarte de `parent`/`move` |
| 5 | `vary(S, δ) → S'` con δ = añadir o quitar un evento Seeded tipado; se comprueba elaborando el experimento | elaborador (`elab.rs:344-380`) |
| 6–7 | episodio en ledger: hipótesis, d, z, recuperados, consultas, evaluaciones, ε usadas, pérdida, motivo y sucesora | `ledger.py`, `episode.py` |

En la Etapa 4, una revisión cuyo Γ' tiene la misma id que su padre es una
reformulación, no una revisión. Se registra como descripción nueva del mismo
nodo.

## 7. Decisiones abiertas

1. **Base de código.** Las etapas 3–6 necesitan el trabajador, la admisión y
   las sesiones paso a paso de `8edcc30`, que no están en esta rama. Hay tres
   opciones:
   - (a) integrar localmente `experiment-llm-normative-exploration-spec` en
     esta rama. El merge textual está limpio, y no implica push ni tocar
     `main`.
   - (b) copiar selectivamente los archivos necesarios.
   - (c) llegar a la Etapa 2 solo con `load_formal_module` y `close_program`,
     y decidir después.

   Recomiendo (a). El encargo prohíbe hacer merge, así que necesito tu
   autorización expresa.
2. **Dominio, interfaz $Q$ y fuente de las valoraciones independientes.** Hay
   tres opciones:
   - El dominio de Riggs del Piloto 003 usa una cabeza artificial de
     impedimento y tiene una sola valoración.
   - El dominio de la historia temporal (perros, `prohibition`/`obligation`)
     no tiene valoraciones externas: las consecuencias esperadas de sus tests
     las escribí yo para certificar la equivalencia y no son $y^*$.
   - El subperfil `compliance` (10.850 cláusulas) es el único con enumerador
     exhaustivo, que la Etapa 7 exige como referencia, pero no tiene
     valoraciones.

   Quien aporte $P^+/P^-$ no puede ser el LLM ni el harness.
3. **Comparación de posiciones.** Hay que decidir si $P^\pm$ se comparan con
   identidad completa o módulo el identificador de norma, y de dónde sale
   `effective` en las generalizaciones aprendidas. Es la decisión pendiente de
   `dacf1d4`, y condiciona directamente $\ell$.
4. **Proveedor de embeddings.** Hay dos opciones:
   - Una API externa: cada llamada es de pago y necesita tu autorización y un
     presupuesto.
   - Un modelo local: añade una dependencia y tiene que caber en 1 GiB total
     sin swap, lo que puede excluir muchos modelos.

   También hay que decidir qué texto se embebe: la prosa del LLM, sin
   verificar más allá de lo léxico, o una descripción mixta que incluya $d_0$.
5. **Alcance de la verificación de la prosa.** Tal como está, la comprobación
   léxica solo detecta identificadores del vocabulario. Las paráfrasis en
   lenguaje natural ("va acompañado de" por `accompanies`) solo serían
   verificables con un glosario por verbo y por constante, que LegalGeneral no
   tiene. Hay que decidir si se escribe ese glosario, y quién lo escribe.
6. **Lenguaje de hipótesis del LLM.** Recomiendo cláusulas Horn ordinarias con
   cabezas en $Q$, sin verbos nuevos ni programas, que es el perfil P1 del
   explorador. Si se quieren hipótesis con programas o con records, el
   validador de admisión tiene que ampliarse.

Lo que no considero abierto, porque se sigue del encargo y de lo certificado:
- la identidad del nodo depende solo de Γ;
- las hipótesis se evalúan en ejecuciones aisladas $T\oplus R$;
- la historia no se toca;
- la pérdida es un intervalo cuando la ejecución es incompleta;
- la similitud coseno solo ordena.

## 8. Verificación de este informe

- No se ha modificado código ni se han ejecutado tests.
- Las ubicaciones de `8edcc30` se han leído con `git show`.
- Se han comprobado directamente:
  - `exploration::install` (`exploration.rs:36`);
  - las firmas públicas de `proposal.rs`;
  - el retorno anticipado de `finish_episode` (`controller.py:174`);
  - el envío exclusivo de `text` en `propose_candidate` (`dispatcher.py:80`);
  - la ausencia de `src/residual/steps.rs` en esta rama;
  - la fusión sin conflictos textuales (`git merge-tree --write-tree`, salida 0).
