# Correspondencia entre el dominio propuesto y la ontología primitiva

Estado: **MAPA PARA REVISIÓN**.
- No modifica fixtures, catálogo de hipótesis, motor ni LegalGeneral.
- La única ejecución nueva es una prueba de expresividad en el scratchpad
  (§2), fuera del repositorio.
- Corrige la dirección de construcción de
  `legal-semantic-domain-contract.md` §3. Las fixtures de ese contrato
  quedan como estaban hasta que se decida este mapa.

## 0. Dirección y criterio

$$
\text{ontología primitiva}\rightarrow\text{construcciones tipadas}\rightarrow
\text{configuraciones relacionales admisibles}\rightarrow\text{dominio experimental}.
$$

La doctrina (clasificación alemana) es una **capa interpretativa** de las
construcciones resultantes (§5). No decide la estructura.

Cada propiedad del dominio se clasifica en una de tres clases:

| Clase | Criterio |
|---|---|
| **A. Composición** | Se expresa con verbos, tipos, records y efectos existentes. Solo puede introducir **constantes nuevas de tipos existentes** (`Conduct`, `Fact`, `Person`, `Thing`, `PropositionTime`), que son instancias y no amplían la ontología. |
| **B. Extensión genuina** | Necesita un verbo o un tipo nuevo. Se justifica solo si no hay una construcción fiel, y se documenta qué significado nuevo introduce. |
| **C. Sin representación fiel** | Ninguna composición conserva el significado, y una extensión mínima tampoco lo haría sin introducir contenido explicativo. Se deja fuera o se declara la pérdida. |

Una composición que obliga a rellenar un argumento sin correspondencia (por
ejemplo, una cosa ficticia en `object`) se marca **A con argumento forzado**.
No cuenta como fiel.

## 1. Primitivas disponibles

**Tipos y records** (`legal_general.res`):

| Tipo | Contenido | Dónde vive |
|---|---|---|
| `Person { patrimony : Patrimony }` | solo el patrimonio | constantes en Known; patrimonio en State |
| `Thing`, `Conduct`, `Authority`, `Fact`, `PerformanceMode` | sin campos | constantes |
| `Relation { debtor : Person, thing : Thing?, conduct : Conduct?, creditor : Person, mode : PerformanceMode }` | vínculo obligacional | en State, producido por los efectos `do` y `notDo` |
| `Patrimony { assets : Set Thing, rights : Set Relation, obligations : Set Relation }` | patrimonio | en State |

**Constantes:** `Doing`, `Abstaining`, `transfer`, `enact`, `use`, y las
autoridades `private`, `legislative`, `judicial`, `administrative` y
`constitutional`.

**Verbos.** Las posiciones tienen todas la firma de 7 argumentos
`(id, subject, conduct, object : Thing, beneficiary, authority, effective)`.

| Papel | Verbos |
|---|---|
| Acto registrado | `Act` (Seeded), con la misma firma de 7 argumentos; `~Act`, evidencia de su no realización |
| Posiciones sembradas | `right`, `prohibition`, `omissionProhibition`, `obligationFiled`, `permissionFiled` |
| Posiciones derivadas | `obligation`, `permission` |
| Fuentes | `legislationText(act, norm, …)`, que vincula un acto con el contenido normativo que produce; `powerGranted`, `extinguished` |
| Orden entre fuentes | `higherAuthority`, `notEarlier`, que son datos sembrados |
| Patrimonio | `patrimonialHolding` (Seeded), que deriva `patrimonialAsset`, luego `owns` y luego `capacity(·, use, ·)`; `transferred` |
| Consecuencias | `compliance`, `violation`, `enforceability`, `inconsistent`, `disputedPerformance`, `patrimonialLiability`, `annulled` |
| Efectos | `give`, `do`, `notDo`, que modifican State; no son evidencia histórica |

**Construcciones de Residuum:** records como argumentos de términos,
`Optional`, programas asociados a verbos, especialización
(`rule b = r(c)`) y efectos sobre State.

**Qué pueden leer las reglas:** solo eventos de Known. **State no es legible
desde los cuerpos de reglas.** Por eso las `Relation` que crean `do` y `notDo`
no están disponibles para las hipótesis.

## 2. Hallazgo de expresividad: el record `Relation` puede usarse en Known

**COMPROBADO** con la ejecución `--forward` vía `test_safe.py` del informe
`20261018T230657027971Z-992962` (`scratchpad/domain/records.res`).

Un verbo Seeded con un argumento de tipo `Relation` admite:
- sembrar literales de record con campos constantes y `thing = none`;
- emparejar en un cuerpo de regla un patrón de record con variables en sus
  campos;
- usar esas variables en la cabeza.

```text
seeded verb relationRecorded(r : Relation)
rule probe(x, y, c, t) =
    relationRecorded(Relation { debtor = x, thing = none, conduct = c, creditor = y, mode = Doing }) @ t
    <= obligation(hypNorm, x, c, obj, y, administrative, hypEnacted) @ t
```

La siembra de `Relation { debtor = ana, thing = none, conduct = care, creditor = bea, mode = Doing }`
derivó `obligation(hypNorm, ana, care, obj, bea, …)`, y LegalGeneral derivó
después `do(ana, care, obj, bea) @ after(t0)`.

**Consecuencia.** Las relaciones jurídicas entre personas pueden
representarse con el **record existente**, con su campo `thing` opcional, en
lugar de con predicados nuevos (`guardianOf`, `employs`, `clientOf`). La
extensión se reduce a **un solo verbo** que registra en Known un `Relation`
del expediente, idealmente con su fuente (§4).

**No comprobado todavía:** el emparejamiento de variables en campos
opcionales con valor (`thing = o`), y `mode` como variable. Son obligaciones
para cuando se rehagan las fixtures.

## 3. Mapa propiedad por propiedad

| Propiedad propuesta | Construcción primitiva candidata | Clase | Fidelidad y efectos en el cierre |
|---|---|---|---|
| `considered(s, c, o, b)` (anclaje) | **retrospectivo:** `Act(a, s, c, o, b, private, fecha)`, la conducta realizada; **prospectivo:** ninguna, porque una pregunta no es un hecho del mundo | A en retrospectivo / B en prospectivo | Con `Act`, LegalGeneral deriva `violation` o `compliance` cuando hay posiciones con los mismos (s, c, o, b), y así Q puede ampliarse a consecuencias. El anclaje prospectivo se mantiene como extensión declarada («conducta examinada»), sin valor fáctico. |
| Conductas `enterWith`, `useInside`, `leaveUnattended` | constantes de `Conduct`; `use` ya existe | A | Son instancias. `useInside` puede sustituirse por `use`, aunque `capacity(·, use, ·)` derivada de `owns` interactúa con ella (§6). |
| `custody(p, o)` (tenencia) | (i) `patrimonialHolding(id, p, o, …)`; (ii) `Act(id, p, carry, o, p, private, fecha)` | (i) **no fiel**; (ii) A con argumento forzado | (i) deriva `owns` y `capacity(use)`: convierte la tenencia en título. (ii) registra un acto de llevar, no un estado continuado, y obliga a elegir un beneficiario (el propio p). La tenencia de hecho como **estado** no tiene primitiva: si se necesita, es **B**. |
| `guardianOf(p, q)` | `relationRecorded(Relation{debtor=p, thing=none, conduct=care, creditor=q, mode=Doing})`, más opcionalmente el acto de nombramiento `Act(n, juez, appoint, ?, p, judicial, fecha)` | A + 1 verbo B (§2); el nombramiento es A con argumento forzado (no tiene cosa) | Representa el **deber** de cuidado, no la tutela completa (representación, administración). Es una relación jurídica preexistente del expediente: es contenido normativo, pero de otra relación distinta de la conducta examinada. |
| `employs(p, q)`, `clientOf(p, q)` | el acto: `Act(k, p, contract, ?, q, private, fecha)`; el vínculo: `relationRecorded(Relation{debtor=…, conduct=service, creditor=…, mode=Doing})` | acto: A con argumento forzado; vínculo: A + verbo B | La calificación como **negocio** no se sigue de `Act` (§5). El contenido del contrato **no** puede ser una posición sobre la conducta examinada: eso daría el tratamiento. Restricción estructural propuesta: las conductas de los vínculos registrados son disjuntas de las conductas examinables. |
| `livesWith(p, q)` | ninguna: no es acto, ni relación obligacional, ni título | **B** | Es una situación de hecho primitiva respecto de esta ontología. |
| `presentWith(p, q)` | `Act(id, p, enter, ?, q, private, fecha)` (entrar) | A con argumento forzado, o **B** | El acto de entrar no es el estado de copresencia y obliga a dar una cosa. La copresencia como estado es **B**. |
| `personTrait(p, minor)` | ninguna: `Person` no tiene campos descriptivos; no hay aritmética de fechas, y `notEarlier` no puede comparar el índice del expediente con una fecha jurídica | **B** | Condición de la persona: primitiva respecto de la ontología. |
| `personTrait(p, visuallyImpaired \| mobilityImpaired \| pregnant \| elderly)` | ninguna | **B** | Como el anterior. |
| `sensitiveTo(p, r)` | ninguna | **B** | Depende además de cómo se clasifiquen las cosas (fila siguiente). |
| `thingTrait(o, animal \| food \| electronic \| motorized \| large)` | ninguna: `Thing` no tiene campos | **B** | La clase o descripción de una cosa es primitiva respecto de la ontología. |
| `thingTrait(o, registered)` | `Act(r, registrador, register, o, titular, administrative, fecha)` | A (con el titular como beneficiario) | La inscripción es un acto de autoridad sobre la cosa, y el hecho relevante es que el acto ocurrió. |
| `thingTrait(o, vaccinated)` | `Act(v, veterinario, vaccinate, o, titular, private, fecha)` | A | Acto material cuya ocurrencia es el hecho. |
| `thingTrait(o, certifiedAssistance)`: **el acto** | `Act(cert, emisor, certify, o, titular, administrative, fecha)` | A | Representa que hubo una certificación. **No** representa qué se certificó. |
| `certifiedAssistance`: **el contenido** | (i) una constante de conducta `certifyAssistance` en el `Act`; (ii) `permissionFiled(cert, titular, enterWith, o, …)`; (iii) `relationRecorded(Relation{…})` | (i) **no fiel**; (ii) **prohibida**; (iii) **no aplicable** → **C**, o B con un verbo de contenido | (i) mete el contenido en el nombre de una constante. (ii) convierte la certificación en un tratamiento sobre la conducta examinada. (iii) el contenido (una aptitud de la cosa respecto de una persona) no es un vínculo obligacional. La única opción fiel es **B**: un verbo de contenido ligado al acto por su id (§4), que necesita su propio vocabulario de aptitudes. |

**Resumen:**

| Clase | Propiedades |
|---|---|
| A | conductas; registro, vacunación y acto de certificar (con `Act`); anclaje retrospectivo |
| A + 1 verbo B | relaciones de guarda y negociales (con `Relation`) |
| A con argumento forzado (no fiel) | tenencia como acto, nombramiento, contrato y entrada (falta de objeto) |
| B | condiciones de personas, clase de cosas, sensibilidad, convivencia, copresencia, tenencia como estado, anclaje prospectivo |
| C (o B con vocabulario propio) | contenido de la certificación |

## 4. Acto y contenido

Una certificación, un contrato o un nombramiento tienen dos componentes:
**que el acto ocurrió** y **qué contenido tiene**. `Act` solo representa el
primero.

- **Precedente en la ontología:** `legislationText(act, norm, subject, conduct, object, beneficiary, authority, effective)`
  vincula un acto de autoridad con el contenido normativo que produce.
- **Para actos privados o administrativos no hay un verbo equivalente.**
  Compartir el `id : Fact` entre un `Act` y una posición archivada es una
  convención que la ontología no interpreta.
- **`Relation` no tiene campo de fuente.** Un vínculo registrado no puede
  remitir al acto que lo constituye.

Extensión mínima candidata (B), que sustituiría a `relationRecorded`:

```text
seeded verb constitutes(act : Fact, r : Relation)   // el acto act constituye o atesta el vínculo r
```

- Conserva la distinción entre acto y contenido: un `Act(k, …)` sin
  `constitutes(k, …)` es un acto de contenido desconocido.
- Un `constitutes` sin `Act` correspondiente sería un contenido sin fuente.
  La admisibilidad de K puede exigir el acto, y eso se declararía.
- Cubre guarda, empleo y servicio.
- **No** cubre contenidos que no son vínculos, como la aptitud certificada de
  una cosa. Para esos haría falta otro verbo de contenido con su propio
  vocabulario, y precisamente ahí es donde se introduciría significado
  explicativo. Por eso lo clasifico como C.

## 5. Capa interpretativa: revisión de las identificaciones

| Identificación propuesta antes | Juicio | Motivo |
|---|---|---|
| `Act(…, private, …)` ≡ negocio jurídico | **retirada** | `Act` es un registro genérico de un acto humano, con conducta, autoridad y vigencia. LegalGeneral lo usa para actos de cumplimiento (`obligationCompliance`, cuya naturaleza negocial es discutida), traslaciones (`transferOccurrence`) y actos de autoridad (`enact`). No representa una declaración de voluntad ni la intención de producir efectos, que son lo que define el negocio. La autoridad `private` solo dice bajo qué autoridad se realiza el acto. |
| `~Act` ≡ omisión | **retirada** | `~Act` es evidencia explícita de que **ese** acto, con su id, autoridad y vigencia, no se realizó. Una omisión jurídicamente calificada presupone un deber de actuar. En LegalGeneral esa calificación solo aparece **derivada**: `obligation & ~Act → violation`, o `omissionProhibition & ~Act → violation`. |
| `give`/`do`/`notDo` ≡ prestaciones de dar, hacer y no hacer | **defendible como interpretación** | Los efectos crean vínculos `Relation` con `mode` `Doing` o `Abstaining`, o mueven activos. No son evidencia de que la conducta ocurriera. |
| `obligation`/`permission` ≡ situaciones jurídicas concretas | **defendible** | Se derivan con participantes concretos. Las hipótesis con variables corresponden a la regla, es decir, a la situación abstracta en el sentido de Bonnecase. |
| `annulled` ≡ nulidad | **retirada** | Marca la invalidez de una norma inferior por jerarquía y orden temporal. No es la nulidad de un acto, y no retira nada. |
| Condiciones de la persona y clase de cosas ≡ hechos naturales | **interpretación posible** | Solo si se añaden como B (§3). La categoría no justifica por sí sola añadirlas. |

La capa doctrinal sirve para **describir** las construcciones en las glosas
futuras, siempre con la cautela de esta tabla. No sirve para crear predicados.

## 6. Consecuencias para el contrato del dominio (todavía sin aplicar)

1. **K deja de ser una colección de etiquetas.** Pasa a ser una configuración
   relacional tipada de:
   - `Act`, que identifica cada acto con su id, autor, conducta, objeto,
     beneficiario, autoridad y fecha;
   - vínculos `Relation` con su fuente;
   - las extensiones B que se aprueben;
   - el anclaje.
2. **La Proposición 1 (evaluación estratificada) cambia.** Si los
   expedientes contienen `Act` o `patrimonialHolding`, LegalGeneral deriva
   `owns`, `capacity`, `transferred`, `compliance` o `violation`.
   - Si las hipótesis pueden leer verbos derivados (por ejemplo,
     `capacity(s, use, o, …)`), deja de valer H6, y la evaluación necesita el
     cierre completo.
   - Esto es deseable, porque las hipótesis interactúan con la ontología real,
     pero encarece la tabla de puntuación y obliga a revisar la
     Proposición 1.
3. **K4 se refina.** Admitir vínculos `Relation` registrados es admitir
   contenido normativo preexistente. Se mantendría la exclusión de posiciones
   sobre las **conductas examinables**, que es lo que daría el tratamiento.
4. **La firma de las posiciones limita la expresividad.** `object : Thing` no
   es opcional en las posiciones (sí lo es en `Relation` y en `do`/`notDo`).
   Por eso los deberes sin cosa solo se expresan fielmente con `Relation`, no
   con `obligation`. Es un límite de LegalGeneral, no del motor. No propongo
   cambiarlo, porque la ontología original no se toca.
5. **Más límites:**
   - Formal no tiene desigualdad (un tercero existencial puede ligarse a un
     participante);
   - no hay aritmética de fechas;
   - State no es legible desde las reglas.

## 7. Extensiones genuinas candidatas (B), con el significado que introducen

| Extensión | Significado nuevo | Necesaria para |
|---|---|---|
| `constitutes(act : Fact, r : Relation)` | el acto act constituye o atesta el vínculo r | guarda, empleo, servicio |
| `condition(person : Person, k : PersonCondition)` | condición fáctica de una persona (edad, salud) que consta en el expediente | menores, discapacidad, embarazo |
| `kind(thing : Thing, k : ThingKind)` | clase descriptiva de una cosa | animal, alimento, vehículo, aparato |
| `sensitiveTo(person : Person, k : ThingKind)` | sensibilidad física de una persona a una clase de cosas | interacción entre participantes y cosas |
| `cohabit(p : Person, q : Person)`, `copresent(p : Person, q : Person)` | situaciones de hecho entre personas | convivencia, copresencia |
| `holding(p : Person, o : Thing)` | tenencia de hecho como estado, sin título | tenencia |
| anclaje prospectivo | conducta examinada, sin valor fáctico | preguntas prospectivas |

Las listas de constantes de `PersonCondition` y `ThingKind`, y sus glosas,
quedan pendientes. Se elegirían describiendo construcciones generales, sin
referirse a los ejemplos de evaluación.

## 8. Decisiones

1. **Relaciones jurídicas.** ¿Se adopta `Relation` con un verbo de registro
   ligado a la fuente (`constitutes`) en lugar de `guardianOf`, `employs` y
   `clientOf`?
2. **Contenido de la certificación.** ¿Se deja fuera del dominio (C), o se
   admite un verbo de contenido aceptando que introduce vocabulario de
   aptitudes?
3. **Anclaje.** ¿Expedientes retrospectivos, con `Act` y por tanto con
   violación y cumplimiento derivados; prospectivos, con un anclaje B; o
   ambos?
4. **Extensiones B de §7.** ¿Cuáles se admiten, sabiendo que cada una es un
   significado nuevo documentado y no una consecuencia de la ontología?
5. **Composiciones con argumento forzado** (el acto de entrar o el de llevar
   sin cosa). ¿Se excluyen, o se admiten con la pérdida declarada?

## 9. Correcciones tras la consulta del usuario (2026-10-18)

Pruebas en el scratchpad (`scratchpad/domain/`), ejecutadas con
`test_safe.py --forward`:

| Pregunta | Resultado | Informe |
|---|---|---|
| ¿Existe `Thing?`? | Sí. `Optional` existe y lo usan `Relation.thing` y `do`/`notDo`. El límite está en que LegalGeneral declara `object : Thing` en las posiciones: es un límite de la **ontología**, no del lenguaje. Se evitaría con una variante renombrada de LegalGeneral, o con `Relation`, que ya tiene la cosa opcional. | — |
| ¿Sirven los subtipos (`entity Animal <: Thing`) como descripciones? | Se declaran, pero el elaborador asigna **un solo tipo por variable**. Usar `o` como `Thing` en el anclaje y como `Animal` en otro átomo da el error `conflicting inferred types for o`. Además, el tipo de una constante es fijo (un contrafactual sobre él cambiaría la identidad del participante), y el parser admite un único supertipo por declaración. | `20261018T231716550010Z-1000013` |
| ¿Fechas con `after`? | Sí. `after(after(epoch))` se usa como argumento de fecha, y una regla empareja el patrón `born(p, after(after(d)))`. Con las construcciones Peano ya certificadas en el repositorio (`peano_after.res`, `peano_multiplication.res`), el orden y la aritmética unaria sobre fechas son **expresables**. | `20261018T231733091051Z-1000211` (`after_arg.res`) |
| ¿Desigualdad? | No se obtiene con subtipos ni con `after`. El emparejamiento solo comprueba igualdad, y no hay negación por fallo. Solo puede aportarse como dato explícito. | — |

**Rectificaciones de §3 y §6:**
- Retiro «no hay aritmética de fechas». Lo correcto es que no hay aritmética
  **predefinida**, pero se puede **componer** con `after` y programas.
- La minoría de edad y la edad avanzada dejan de ser extensiones B.
  Se construyen componiendo `born(p, d)`, una fecha del expediente como
  argumento (nunca el índice `@ t`) y un programa definicional de orden.
- Los subtipos sirven para distinciones de clase **invariantes** de una
  entidad. No sirven para propiedades que varían entre situaciones ni para
  propiedades que tenga que leer una hipótesis que liga el objeto desde una
  posición de tipo `Thing`.
- Si se componen descripciones con reglas, P1 («solo firma») se relaja así:
  se admiten **programas definicionales** cuyas cabezas son verbos
  descriptivos, nunca posiciones normativas, y que se certifican por
  separado (terminación, sin cabezas de Q).
