# Contrato del dominio para la revisión semántica

Estado: **CONTRATO PROPUESTO**. Este documento no genera valoraciones, no
añade fixtures al repositorio, no implementa la ablación y no ha hecho
llamadas a modelos. LegalGeneral, el motor, `main` y `v1` no se han tocado.

Complementa `legal-semantic-revision-contract.md` (§0 bis: alcance acordado).

Marcas usadas:
- **DEMOSTRADO**;
- **COMPROBADO**: ejecución concreta, con su id;
- **PROPUESTO**;
- **ABIERTO**: decisión del usuario;
- **OBLIGACIÓN**: verificación que hay que hacer antes de usar el dominio.

## 0. Resumen

- **Dominio:** acceso y uso de espacios ajenos en presencia de terceros.
  - Es un dominio de *situaciones descritas*, no de normas.
  - Se apoya en la forma que LegalGeneral impone a toda posición normativa:
    `(id, subject, conduct, object, beneficiary, authority, effective) @ t`.
- **Vocabulario:** un módulo experimental que importa LegalGeneral sin
  modificarlo. Solo declara entidades, constantes y verbos Seeded
  descriptivos; ninguna regla, ningún verbo Derived o Effect y ningún
  término deóntico o evaluativo. Por construcción, en sus definiciones no hay
  justificaciones ni tratamientos.
- **Las cuatro piezas:**
  - **K**: asignaciones parciales de hechos descriptivos sobre un reparto de
    participantes. Cada átomo es positivo, evidencialmente negativo o
    desconocido.
  - **H**: cláusulas Horn cuya cabeza es una posición normativa existente y
    cuyo cuerpo es descriptivo.
  - **Q**: posiciones normativas proyectadas.
  - **Δ**: desplazamientos de un átomo en K.
- **Conexión:** la evaluación $\operatorname{ev}(h,S)=o_Q(C_{T\oplus h}(S))$.
- **Subespacio de la ablación:** cuerpos de hasta 2 literales más el anclaje,
  lo que da **118.140 hipótesis**. El límite sale de la gramática:
  - 2 es la longitud mínima que permite expresar una propiedad de un tercero
    vinculado por una relación;
  - 1 solo alcanza propiedades de los participantes de la cabeza.
- **Viabilidad COMPROBADA:** un boceto del módulo elabora sobre LegalGeneral,
  y una hipótesis con tercero existencial deriva su posición y el efecto
  `notDo` de LegalGeneral (§12).

## 1. Qué fija LegalGeneral y qué le falta

Lo que fija, leído de `ontology_reference/legal_general.res`:

| Elemento | Contenido | Consecuencia para el dominio |
|---|---|---|
| Firma de posición | `(id : Fact, subject : Person, conduct : Conduct, object : Thing, beneficiary : Person, authority : Authority, effective : PropositionTime)` en `obligation`, `permission`, `prohibition`, `omissionProhibition`, `right`, `obligationFiled`, `permissionFiled` | Los participantes que una posición puede nombrar son un sujeto, una cosa y un beneficiario. Cualquier otro participante solo puede aparecer en el cuerpo. |
| Tipo de verbo | `obligation` y `permission` son Derived; `prohibition`, `omissionProhibition` y `right` son Seeded | Una prohibición solo puede aprenderse con una cabeza Seeded. El elaborador lo permite (rechaza únicamente las cabezas Seeded negativas), y la historia certificada ya lo usa (`ordinaryBan`). |
| Consecuencias | `prohibitionCreatesAbstention` → `notDo @ after t`; `obligationCreatesRelation` → `do @ after t`; `filedPermission`, `filedObligation`; `compliance`/`violation` con `Act`/`~Act`; anulación por autoridad superior y vigencia no anterior | Q puede ampliarse con consecuencias cuando los expedientes contienen actos y normas previas. |
| Disciplina temporal | cuerpos con un único `@ t`; solo los efectos van en `@ after t` | Las hipótesis deben respetarla para poder adoptarse en la historia (L1–L7). |
| Falta | No hay vocabulario descriptivo de situaciones: propiedades de personas o cosas, relaciones entre personas, presencia | Es lo que aporta el módulo de vocabulario. |

## 2. Principios de diseño

| | Principio | Cómo se comprueba |
|---|---|---|
| P1 | **Solo firma.** El módulo declara entidades, constantes y verbos Seeded. No declara reglas, ni verbos Derived o Effect, ni programas, ni defaults, ni experimentos. | Obligación V2: inspección del AST elaborado |
| P2 | **Solo descriptivo.** Ningún nombre expresa una calificación deóntica (permitido, prohibido, debido), evaluativa (peligroso, molesto, legítimo, razonable, necesario) ni una conclusión (riesgo, daño, excepción). | V3: lista de exclusión y revisión humana |
| P3 | **Gramática uniforme.** Rasgos y relaciones son argumentos tipados (`personTrait(p, minor)`), no predicados distintos. Todos los rasgos tienen la misma forma sintáctica, y ninguno recibe un tratamiento gramatical especial. | Por construcción |
| P4 | **Participantes múltiples.** Hay rasgos de cualquier persona, rasgos de cosas, relaciones persona–persona, persona–cosa y persona–rasgo de cosa, y terceros existenciales en las hipótesis. | §4 y §6 |
| P5 | **Tres estados por átomo.** Positivo, evidencialmente negativo (`~`) o desconocido. Lo ausente no equivale a lo negado. | K3 |
| P6 | **Congelación previa.** El vocabulario, H, el generador de situaciones y Q se fijan, se registran con su hash y se archivan antes de generar ninguna valoración. | V10 |

Cómo se eligió el vocabulario:
- Para cada tipo (persona, cosa, relación personal, conducta) se eligen
  constantes de **clases semánticas distintas**: edad, capacidad, estado
  fisiológico, categoría, certificación y registro, tamaño y motorización,
  cuidado, empleo, convivencia, contrato y copresencia.
- Ninguna constante procede de una respuesta esperada.
- La única que el ejemplo del perro hace saliente (la alergia) no figura
  como rasgo. Se sustituye por la relación uniforme `sensitiveTo(person, ThingTrait)`,
  que se aplica por igual a animales, alimentos, aparatos o vehículos.

## 3. Módulo de vocabulario (PROPUESTO)

`docs/legal-geometry/semantic-domain/vocabulary_natural.res`. No se crea en este
entregable.

```text
module SharedSpacesVocabulary
import ".../ontology_reference/legal_general.res"

entity PersonTrait
entity ThingTrait
entity Bond

const minor, elderly, visuallyImpaired, mobilityImpaired, pregnant : PersonTrait
const animal, certifiedAssistance, vaccinated, food, electronic,
      motorized, large, registered : ThingTrait
const guardianOf, employs, livesWith, clientOf, presentWith : Bond
const enterWith, useInside, leaveUnattended : Conduct

seeded verb considered(subject : Person, conduct : Conduct, object : Thing, beneficiary : Person)
seeded verb personTrait(person : Person, trait : PersonTrait)
seeded verb thingTrait(thing : Thing, trait : ThingTrait)
seeded verb sensitiveTo(person : Person, trait : ThingTrait)
seeded verb bond(from : Person, to : Person, kind : Bond)
seeded verb custody(person : Person, thing : Thing)

const hypNorm : Fact
const hypEnacted : PropositionTime
```

Formal no admite declarar varias constantes en una sola línea; el módulo real
tendrá una por línea.

| Símbolo | Lectura descriptiva (glosa permitida) |
|---|---|
| `considered(s, c, o, b)` | «se examina la conducta c de s con la cosa o, en el ámbito de b». Es la pregunta del expediente, no un hecho jurídico. |
| `personTrait(p, r)` | p tiene el rasgo r |
| `thingTrait(o, r)` | la cosa o tiene el rasgo r |
| `sensitiveTo(p, r)` | p tiene una sensibilidad física documentada a las cosas con rasgo r |
| `bond(p, q, k)` | entre p y q existe la relación k, con esa orientación |
| `custody(p, o)` | p tiene o consigo o a su cargo |
| `hypNorm`, `hypEnacted` | identificador y vigencia declarada de la norma *hipotética* evaluada (§5) |

Las glosas son definicionales. Las glosas extendidas siguen las mismas
reglas P2 y pasan la misma revisión humana (V3) antes de la congelación.

### 3.1 Controles de vocabulario

| Variante | Qué se renombra | Qué conserva |
|---|---|---|
| **N** (natural) | nada | todo |
| **O1** (dominio opaco) | todos los símbolos del módulo de vocabulario (entidades, constantes, verbos), mediante una biyección fija con identificadores como `vb3`, `pt2`, `tt7`, `bd1`, `cd2` | los nombres de LegalGeneral (`prohibition`, `permission`, `notDo`, …) |
| **O2** (opaco total) | además, los símbolos de una **copia renombrada** de LegalGeneral (`LegalGeneralOpaque`); el original no se toca | solo la estructura |

- O1 mide si el LLM aprovecha el significado del vocabulario de situaciones.
- O2 mide además su conocimiento previo del vocabulario deóntico.
- Las glosas también se renombran en O1/O2, o se suprimen. Si se
  conservaran, deshacerían la opacidad.

## 4. Dominio de situaciones K

**Reparto de participantes** (para la ablación; PROPUESTO): las personas
$\{s,b,p_1,p_2\}$ y la cosa $\{o\}$, todas constantes distintas.

La base de átomos $\Sigma_0$ son todos los átomos ground del vocabulario
sobre ese reparto, salvo `considered`:

| Familia | Átomos |
|---|---|
| `personTrait` | 4 × 5 = 20 |
| `thingTrait` | 8 |
| `sensitiveTo` | 4 × 8 = 32 |
| `bond` (pares ordenados distintos) | 12 × 5 = 60 |
| `custody` | 4 |
| **Total** | **124** |

**Situación:** $S=(\alpha,\ \kappa)$, donde
- $\alpha:\Sigma_0\to\{+,\sim,?\}$ asigna un estado a cada átomo;
- $\kappa=$ `considered(s, c, o, b)` es el anclaje, con $c$ una de las tres
  conductas.

Las semillas de $S$ son $\{a@t: \alpha(a)=+\}\cup\{\sim a@t:\alpha(a)=\sim\}\cup\{\kappa@t\}$,
y los átomos con estado `?` no se siembran.

**Admisibilidad de situaciones:**

| | Condición |
|---|---|
| K1 | Tipado correcto: el experimento elabora (`elab.rs:344-380`). |
| K2 | Exactamente un anclaje, con $s\neq b$ como constantes distintas. |
| K3 | Nunca $a$ y $\sim a$ a la vez. La evidencia contradictoria es otro fenómeno, ya modelado por `inconsistent`, y se deja para la extensión. |
| K4 | No hay hechos normativos sembrados de LegalGeneral (`prohibition`, `permissionFiled`, `right`, …), ni `Act`, `higherAuthority` o `notEarlier`. Las posiciones proceden solo de las hipótesis. En la extensión (§7.2) esta condición se levanta. |
| K5 | Un solo índice $t$ por situación (el momento del expediente), que es un `PropositionTime` constante del expediente. |

K no lleva **restricciones semánticas** del tipo «un menor no tiene
`employs`» o «solo los animales están `vaccinated`». Si se añadieran, serían
presupuestos sobre el mundo; se declararían aparte y se medirían.

**Desplazamientos contrafactuales Δ:**
- $\delta_{a,v}$ cambia el estado de un átomo $a\in\Sigma_0$ al valor
  $v\in\{+,\sim,?\}\setminus\{\alpha(a)\}$. Además, $\delta_c$ cambia la
  conducta del anclaje.
- Toda situación tiene exactamente $2\cdot124+2=250$ vecinos inmediatos.
- Todos los vecinos son admisibles, porque K3 se conserva por construcción.
- Δ se enumera de forma **uniforme**, sin preferir átomos por su supuesta
  relevancia.

**Semejanza entre situaciones** (para una lectura lewisiana, declarada
aparte): la distancia de Hamming sobre $\alpha$ y $\kappa$.
- La admisibilidad (K1–K5) y la semejanza son dos declaraciones
  independientes.
- La semejanza $s_k$ entre descripciones de hipótesis no se usa como
  semejanza entre situaciones.

**Distribución de expedientes:**
- Las situaciones observadas y reservadas se extraen con un generador
  declarado (probabilidades por estado y semilla fija) **antes** de cualquier
  valoración.
- Ninguna situación se elige por su etiqueta.
- El generador y su semilla son información reservada para el aprendiz.

## 5. Posiciones observables Q

**Q_pos (ablación):** los verbos `prohibition`, `permission` y `obligation`,
con la proyección

$$
o_Q(e)=(\text{verbo},\ subject,\ conduct,\ object,\ beneficiary,\ t)
\quad\text{para } e\in C_{T\oplus h}(S),\ \operatorname{verbo}(e)\in Q_{pos}.
$$

La proyección **descarta** `id`, `authority` y `effective`:
- En el registro de evaluación (𝒫) el evento se conserva completo.
- Solo la comparación con valoraciones usa $o_Q$.
- Así la valoración no depende del identificador con que cada hipótesis
  nombra su norma, y responde a la decisión pendiente de «comparar módulo el
  identificador».

**ABIERTO:** que descartar `authority` y `effective` es adecuado para la
ablación. En la extensión (§7.2), donde intervienen anulación y vigencia,
esos argumentos tienen que observarse.

**`effective` en la ablación:**
- Toda hipótesis usa la constante `hypEnacted` («vigencia declarada de la
  norma hipotética»), distinta del momento $t$ del expediente.
- No se inventa una fecha a partir de $t$, y ninguna regla usa $t$ como
  vigencia.
- Esto sustituye la convención `normEffective` de `dacf1d4` solo para las
  normas hipotéticas.

**Q_ext (extensión):** añade `violation`, `compliance`, `annulled`, `notDo` y
`do`, cuando los expedientes contengan `Act`/`~Act` o normas previas.

**Valoraciones:** $P^\pm$ son conjuntos de posiciones proyectadas. Su
contrato, incluida la negociación simulada, se revisará aparte.

## 6. Espacio de generalizaciones H

### 6.1 Gramática

```text
Clause   ::= considered(s, C, o, b) @ t & Lit* <= Head @ t
Head     ::= K(hypNorm, s, c, o, b, administrative, hypEnacted)
K        ::= prohibition | permission | obligation
C        ::= c | enterWith | useInside | leaveUnattended   (c variable: cabeza con c)
Lit      ::= [~] personTrait(P, PT) | [~] thingTrait(o, TT) | [~] sensitiveTo(P, TT)
           | [~] bond(P, P', BD) | [~] custody(P, o)
P, P'    ::= s | b | x                                     (x: tercero existencial)
TT       ::= constante ThingTrait | φ                      (φ: variable de rasgo)
```

Cuando la conducta es una constante, la cabeza usa esa constante.

### 6.2 Admisibilidad de hipótesis

| | Condición | Origen |
|---|---|---|
| H1 | Elabora con el elaborador real sobre el vocabulario y LegalGeneral | `load_formal_module` |
| H2 | Cabeza segura: todas sus variables ($s,c,o,b$) están ligadas por el anclaje | gramática; `S_UNBOUND_HEAD` |
| H3 | Conexión: si aparece $x$, al menos un literal `bond` lo une con $s$ o $b$ | contrato de conexión (`legal-relational-search-space-contract.md` §4) |
| H4 | No hay un átomo y su negación evidencial en el mismo cuerpo; esa cláusula solo se activaría con evidencia contradictoria, que K3 excluye | coherencia con K3 |
| H5 | Si aparece $\varphi$, aparece al menos dos veces. Una variable sirve para identificar posiciones; un $\varphi$ aislado equivale a una disyunción sobre constantes | operador «identificar variables» (§8 del contrato del espacio) |
| H6 | El cuerpo solo usa verbos del vocabulario: no usa verbos Derived ni normativos de LegalGeneral | precondición de la Proposición 1 (§8) |
| H7 | Disciplina temporal: un único $t$ en cuerpo y cabeza, y nada en `after` | L1–L7 (`legal-temporal-history.md`) |

El cuerpo es un **conjunto**: el orden canónico lo fija `canon`. Dos órdenes
de los mismos literales dan la misma hipótesis.

Sobre $x$ hay que tener en cuenta que Formal no tiene desigualdad: $x$
puede ligarse a $s$ o a $b$ si los hechos lo permiten. Es semántica real del
motor y se documenta. No se añade un predicado de distinción, porque sería
un presupuesto nuevo.

### 6.3 Subespacio finito de la ablación

**Alfabeto de literales:** 168, que se descomponen así:
- `personTrait` 30;
- `thingTrait` 18 (16 más 2 con φ);
- `sensitiveTo` 54 (48 más 6 con φ);
- `bond` 60;
- `custody` 6.

$H_2$ son las cláusulas con como mucho 2 literales además del anclaje, que
cumplen H1–H7:

| | Valor |
|---|---|
| Cuerpos admisibles | 9.845 |
| Cabezas (3 tipos × 4 opciones de conducta) | 12 |
| **$\lvert H_2\rvert$** | **118.140** |
| Referencia: hasta 1 literal | 1.596 |
| Referencia: hasta 3 literales | del orden de $3\cdot10^6$ (sin el refinamiento de φ) |

Recuento calculado con un script de conteo (scratchpad). La obligación V4
exige reproducirlo dentro del generador.

**Justificación de los límites.** Ninguno se apoya en etiquetas:
- **Longitud 2:** es la mínima con la que la gramática expresa una
  propiedad de un tercero vinculado, $\mathtt{bond}(x,\cdot)\wedge\mathtt{trait}(x)$,
  y una coincidencia de rasgo entre participantes, $\mathtt{thingTrait}(o,\varphi)\wedge\mathtt{sensitiveTo}(\cdot,\varphi)$.
  Con longitud 1 solo se alcanzan propiedades aisladas de $s$, $o$ o $b$.
- **Un solo tercero y una sola φ:** es lo mínimo para la exigencia
  multiparticipante. Dos terceros requieren al menos 3 literales para
  quedar conectados.
- **Una sola cláusula:** por la Proposición 1 de §8, y para que cada
  hipótesis tenga un único productor nuevo. Las combinaciones de cláusulas
  pertenecen a la extensión.
- **Coste:** con la Proposición 1, evaluar $H_2$ sobre un expediente es
  instanciar cada cuerpo sobre sus semillas, que es barato. El tamaño cabe
  en el régimen exhaustivo, como pide la Proposición 1 del contrato de
  revisión.

**Estructuras sobre $H_2$** (se calculan, no se identifican entre sí):
- Preorden de cobertura $\preceq_g$. Para la misma cabeza, si el cuerpo de
  $h$ contiene al de $h'$, entonces $h\preceq_g h'$, por el teorema de
  subsunción estructural.
- Partición por protocolo ($\ker$ de las firmas sobre los expedientes).
- Espacio de versiones.

## 7. Transformaciones normativas admisibles

### 7.1 Sobre hipótesis (exploración)

Son los operadores de `legal-relational-search-space-contract.md` §8,
restringidos a la gramática de §6.1:

| Operación | Dirección | En $H_2$ |
|---|---|---|
| Añadir o quitar un literal | especialización / generalización | sí, dentro de la longitud ≤ 2 |
| Cambiar la polaridad de un literal | ninguna de las dos (incomparable) | sí |
| Fijar la conducta, o relajarla a $c$ | especialización / generalización | sí |
| Sustituir un rasgo por $\varphi$ compartida, o al revés | generalización / especialización | sí |
| Cambiar el tipo de cabeza | otra posición (incomparable) | sí |
| Introducir o eliminar el tercero $x$ (con su `bond`) | especialización / generalización | sí |

Una «excepción» no tiene una única forma en un sistema monótono. Puede
expresarse de dos maneras, y el dominio no presupone ninguna:
- especializando con evidencia negativa explícita (`~thingTrait(o, …)`);
- con una posición de otra autoridad que, mediante las reglas de anulación
  de LegalGeneral, marca `annulled` sin retirar nada.

### 7.2 Sobre la teoría y la historia (extensión)

- **Adopción:** $R_k\subseteq H$ pasa a la historia como plantillas
  capturadas por `program Templates`, un `program Rk(t)` y un anclaje
  `rule anchorRkAtTk() = I <= appliesRk() @ tk`. Es exactamente la
  construcción certificada de `dacf1d4`.
  - Cada $R_k$ tiene su identificador de norma y su vigencia, declarados en
    su módulo de etapa.
  - En ese punto $o_Q$ debe observar `id`, `authority` y `effective`.
- **Conflicto y anulación:** al levantar K4 se admiten normas sembradas de
  otras autoridades, `higherAuthority` y `notEarlier`, para que intervengan
  las reglas de anulación existentes.
- **Hipótesis con varias cláusulas, longitudes mayores, más terceros y
  conductas:** se amplían los parámetros de la gramática sin cambiarla.

## 8. Conexión mediante la evaluación

$$
\operatorname{ev}(h,S)=o_Q\big(C_{T\oplus h}(S)\big),\qquad T=\text{LegalGeneral}+\text{vocabulario}.
$$

**Proposición 1 (evaluación estratificada). DEMOSTRADO bajo H6, K4 y Q_pos.**

*Enunciado.* Sea $I_h(S)$ el conjunto de instancias de la cabeza de $h$
sobre las semillas de $S$. Entonces

$$
o_{Q_{pos}}\big(C_{T\oplus h}(S)\big)=o_{Q_{pos}}\big(C_T(S)\big)\cup o_{Q_{pos}}\big(I_h(S)\big),
$$

y bajo K4, $o_{Q_{pos}}(C_T(S))=\varnothing$.

*Prueba.*
- El cuerpo de $h$ solo contiene verbos Seeded del vocabulario (H6).
- Ninguna regla de $T$ produce esos verbos, porque el vocabulario no tiene
  reglas (P1). Por tanto, en $C_{T\oplus h}(S)$ sus átomos son exactamente
  las semillas, y las instancias de $h$ son $I_h(S)$.
- Las reglas de LegalGeneral que producen verbos de $Q_{pos}$ son:
  - `filedObligation` y `filedPermission`, que necesitan `obligationFiled`
    o `permissionFiled` sembrados, excluidos por K4;
  - `statutoryObligation`, que necesita `law` y, por tanto, `Act`,
    `legislativePower` y `legislationText`, también excluidos por K4.
- Ninguna regla de LegalGeneral concluye `prohibition`. Las cabezas de $h$
  solo alimentan reglas cuyas conclusiones no están en $Q_{pos}$:
  - efectos `notDo` y `do`;
  - `violation` o `compliance`, que exigen `Act`/`~Act`, excluidos por K4;
  - `annulled`, que exige `higherAuthority` y `notEarlier`, excluidos por K4.

∎

*Uso:* la Proposición 1 permite precalcular $\operatorname{ev}$ sobre $H_2$
sin hacer un cierre por par. **OBLIGACIÓN V6:** certificar que el atajo
coincide con `close_program` en todos los pares de la tabla de puntuación,
o en una muestra declarada si la tabla completa excede el presupuesto. Ante
cualquier diferencia, manda el motor. Fuera de H6, K4 y Q_pos no se usa el
atajo.

**Relación con las otras estructuras:**
- **K** es una estructura de situaciones. Sobre ella, $C_T$ es un operador
  de cierre.
- **Sobre H** se calculan el preorden de cobertura, la partición y el
  espacio de versiones.
- **Δ** es la vecindad de Hamming en K.
- **ev** es la única conexión entre K y H.
- Ninguna de estas estructuras se deduce de la semejanza entre
  descripciones.

## 9. Preservación de la historia temporal

1. **Sin cambios:** `dacf1d4` no se toca. Sus fixtures, su vocabulario
   (`TemporalHistoryVocabulary`) y sus tests siguen como están. El dominio
   nuevo es un módulo distinto y no reutiliza nombres que entren en conflicto
   (el error `duplicate symbol` es global en el grafo).
2. **Aislamiento:** toda evaluación de hipótesis es una ejecución
   $T\oplus h$ aislada, en un único momento. La equivalencia
   $\pi_{t_k}\operatorname{Run}_{H_n}\cong\operatorname{Run}_{T_k}(S_k)$
   justifica no construir nunca $H_n$ para evaluar hipótesis.
3. **Requisito de adopción:** H7 garantiza que toda $h\in H_2$ cumple la
   forma temporal que exigen los detectores L1–L7 de
   `temporal_history_tests.rs`. Sigue siendo una OBLIGACIÓN (V9) comprobarlo
   antes de adoptar cualquier hipótesis.
4. **Cuatro historias:** formalizaciones (Formal), descripciones y
   embeddings, hipótesis y evaluaciones, y valoraciones y pérdidas (ledger).
   La iteración de revisión es un número de secuencia del ledger, no un
   índice de la ontología.

## 10. Información semántica disponible para el LLM

| Disponible | No disponible |
|---|---|
| Nombres y firmas del vocabulario (variante N), o identificadores opacos (O1/O2) | Valoraciones reservadas y cualquier agregado calculado con ellas |
| Glosas definicionales congeladas (solo en N) | Generador de situaciones, su semilla y las situaciones reservadas |
| Fuente de LegalGeneral (N y O1), o de la copia opaca (O2) | Intereses, utilidades o instrucciones de los negociadores simulados |
| Semillas de los expedientes observados | Tabla de puntuación offline |
| Salidas del motor sobre los expedientes observados (según el nivel I0–I3) | Estado de compatibilidad de hipótesis no evaluadas por el propio episodio |
| Valoraciones observadas (según el nivel de información de la condición) | Biyección de renombrado (en O1/O2) |
| Su propia descripción $d_k$ y su historial | Descripciones de otros nodos fuera de lo que define la condición |

**Comprobación léxica en O1/O2:** las descripciones no pueden contener nombres
naturales del vocabulario, porque indicarían una filtración de la biyección.
Las afirmaciones `hipótesis` pueden usar lenguaje natural libre, pero en O1/O2
no pueden asignar significado a un identificador opaco salvo como
`hipótesis` explícita. Esa asignación es precisamente lo que se mide.

## 11. Obligaciones de verificación

Todas se ejecutarán antes de generar valoraciones, con tests individuales vía
`scripts/test_safe.py`.

| | Obligación |
|---|---|
| V1 | El vocabulario N, O1 y O2 elabora. El hash del archivo de LegalGeneral no cambia. |
| V2 | El módulo de vocabulario solo contiene `Entity`, `Const` y `Verb` Seeded (comprobación sobre el AST elaborado y sus orígenes). |
| V3 | Revisión humana de P2: lista de nombres y glosas firmada antes de la congelación. Esto lo decide el usuario, no el harness. |
| V4 | El generador de $H_2$ enumera exactamente 9.845 cuerpos × 12 cabezas, sin duplicados según `canon`, con la misma fórmula de conteo independiente. |
| V5 | Toda $h\in H_2$ elabora con el elaborador real y cumple H2–H7. Hay contraejemplos negativos para cada código de rechazo. |
| V6 | El atajo de la Proposición 1 coincide con `close_program`. |
| V7 | Isomorfismo de renombrado: $\operatorname{Run}(\rho T,\rho S)=\rho\operatorname{Run}(T,S)$ en todos los pares comprobados, para O1 y O2. |
| V8 | Comprobador de K1–K5. Δ genera exactamente 250 vecinos admisibles. El generador de situaciones es reproducible por semilla. |
| V9 | Una muestra declarada de $H_2$ pasa los detectores L1–L7. Además, una adopción de prueba en una historia de dos etapas conserva las proyecciones, con el patrón de `projection_of_the_history_equals_each_isolated_run`. |
| V10 | Congelación: el hash de vocabulario, glosas, gramática, $H_2$, generador y Q queda registrado en el ledger antes de cualquier valoración. |

## 12. Comprobación de viabilidad realizada

Fuera del repositorio, en el scratchpad, elaboré un boceto reducido del
vocabulario: dos rasgos por tipo, `bond` y `custody`, con una relación de
alergia todavía como rasgo, anterior a la decisión de §2. Contiene una
hipótesis con tercero existencial y un expediente:

```text
rule h(s, c, o, b, x, t) =
    considered(s, c, o, b) @ t & bond(x, b, presentWith) @ t
    & personTrait(x, allergicToAnimals) @ t
    <= prohibition(hypNorm, s, c, o, b, administrative, hypEnacted) @ t
```

Ejecución `--forward` con `test_safe.py`, informe
`20261018T215441674171Z-942749`. **COMPROBADO:**
- elabora;
- `x` se liga al tercero `eva`;
- se deriva `prohibition(hypNorm, ana, enterWith, rex, host, administrative, hypEnacted) @ t0`;
- LegalGeneral deriva `notDo(ana, enterWith, rex, host) @ after(t0)`;
- `~thingTrait(rex, certifiedAssistance)` permanece como evidencia sin
  efecto, porque ninguna regla la usa.

Esta comprobación es solo de viabilidad sintáctica y semántica. No es la
obligación V1 sobre el módulo definitivo.

## 13. Decisiones abiertas

1. **Lista de vocabulario y glosas (V3).** ¿Apruebas las clases semánticas y
   las constantes de §3, o prefieres otro dominio de situaciones con la misma
   estructura?
2. **Proyección $o_Q$.** ¿Es aceptable descartar `authority` y `effective`
   en la ablación?
3. **Reparto de participantes de K.** ¿Bastan cuatro personas y una cosa
   para la ablación? Una segunda cosa duplicaría `thingTrait`/`custody` y
   abriría relaciones cosa–cosa.
4. **Conductas.** ¿Bastan tres? Las conductas son constantes y no cambian la
   gramática.
5. **Generador de situaciones.** Probabilidades por estado, número de
   expedientes observados y reservados, y splits. Se fijarán junto con el
   contrato de valoraciones, antes de ellas.
