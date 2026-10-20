# Piloto 004-R: aprendizaje generativo de reglas sobre LegalGeneral (contrato v1, propuesta)

Estado: **propuesta, no congelada**.
- Rama: `experiment-pilot004-documentary`.
- Sustituye al contrato mínimo (`docs/pilot-004-documentary-minimal-contract.md`) en el espacio de hipótesis y el protocolo. Conserva de él las reglas de representación R1–R8 (§1) y las configuraciones de §2, con el cambio en Brookside que se explica en §3.1.
- El Piloto 004 v2 sigue congelado y queda como ablación separada (§9).

Nada de esto está implementado. No se ha ejecutado el explorador ni se han
hecho llamadas pagadas.

## 0. Criterio del piloto

Permitir que un LLM genere generalizaciones sobre la ontología fija, que se
registren como hipótesis, que se apliquen a experiencias sucesivas, y
observar si la interacción entre derivación, recalcitrancia y revisión
mejora su capacidad predictiva.

$$T_k = O \cup R_k$$

| Símbolo | Significado |
|---|---|
| $O$ | LegalGeneral original. Fija: ni el LLM ni el piloto la modifican. |
| $R_k$ | Reglas aprendidas hasta el paso $k$, cada una con origen, alcance y estado. No forman parte de $O$ ni tienen validez jurídica general. |
| $S_i$ | Configuración establecida del expediente $i$. |
| $N_k(S_i)$ | Posiciones proyectadas de `close_program`$(T_k, S_i)$. Las deriva Residuum, no el LLM. |

**Lo que no se exige:**
- **No hace falta una refutación para admitir una regla.** Una regla se admite si es formalmente válida (§2).
- **La ausencia de recalcitrancia es un resultado experimental.** Indica que la capacidad de revisión no ha podido ponerse a prueba, no que haya fallado.

## 1. Tres resultados distintos

| Resultado | Definición | Qué no implica |
|---|---|---|
| **V: formalmente válida** | La regla supera el lenguaje admisible (§2), se elabora con el elaborador real y su cierre sobre cada $S_i$ evaluado termina con estado verificado (§4). | Nada sobre su corrección jurídica ni sobre su ajuste a los documentos. |
| **C: compatible con las experiencias evaluadas** | Hay al menos una posición valorada en algún expediente ya visto en la que la regla influye (está expuesta), y la pérdida en las posiciones a las que está expuesta es 0 después de la revisión permitida. | Si la regla se ajustó con la discrepancia de ese mismo expediente, C no es predicción. Si la regla no está expuesta, no hay C, sino «no expuesta». |
| **P: respaldo prospectivo** | La regla quedó comprometida antes de ver el expediente $j$ y ninguna discrepancia de $j$ había llegado al aprendiz. La regla estuvo expuesta en $j$ y $\ell_j$ sobre sus posiciones expuestas fue 0. | No demuestra validez universal. Se informa el número de expedientes y de posiciones expuestas. |

**Estados del registro** para cada regla, que pueden combinarse:
- `valid`;
- `compatible(n, e)`;
- `prospective(n, e)`;
- `refuted(dossier, posiciones)`;
- `unexposed`;
- `superseded_by(id)`;
- `pending(motivo)`.

`n` es el número de expedientes y `e` el número de posiciones valoradas
expuestas.

**Exposición:** una regla $r\in R_k$ está expuesta en la posición valorada
$p$ del expediente $i$ si se cumple una de estas dos condiciones:
- $p\in N_k(S_i)$ y alguna derivación de $p$ usa $r$, según la provenance;
- $p\in P_i^+$ falta, y $r$ tiene cabeza del mismo verbo y argumentos compatibles con $p$.

La segunda condición se fija como predicado mecánico sobre la cabeza
canónica: la regla «podía» producir $p$ y no lo hizo.

## 2. Lenguaje de hipótesis admisibles $\mathcal L_{adm}$

| | Restricción | Cómo se comprueba |
|---|---|---|
| A1 | El texto propuesto contiene solo reglas Formal. Nada de verbos, entidades, constantes, imports ni módulos. | Existe: `proposal::admit`, códigos `S_DECLARATION` y `S_IMPORT`. |
| A2 | Se elabora contra $O$ importada sin cambios, y $O$ se conserva. | Existe: `load_formal_module` y `theory_preserved` (`S_ORIGIN`). |
| A3 | Cabezas permitidas: `obligation`, `permission`, `prohibition`, `omissionProhibition`, `right`, `compliance`, `violation`, `enforceability`, `patrimonialLiability`. | Comprobación nueva sobre el AST, que cuesta poco. |
| A4 | Cabezas prohibidas: `Act` y las entradas documentales (`obligationFiled`, `permissionFiled`, `legislationText`, `powerGranted`, `patrimonialHolding`, `extinguished`, `higherAuthority`, `notEarlier`), porque fabricarían hechos o documentos. También se prohíben los efectos (`give`, `do`, `notDo`), porque la ejecución de State queda fuera, y las marcas `inconsistent`, `disputedPerformance` y `annulled`. | Comprobación nueva sobre el AST. |
| A5 | Solo se permite la instalación por adición. Nada de excepciones, defaults, retracción ni sustitución de reglas de $O$. | Existe: `exploration::install` añade y no reescribe. |
| A6 | Toda constante de la regla pertenece a $O$ o a algún $S_j$ ya presentado. Una regla con constantes de un único expediente se marca `case_specific`, y su C o P no cuenta como generalidad. | Comprobación nueva. Impide transcribir resultados aún no vistos. |
| A7 | Perfil acotado: número de cláusulas, literales, binders y profundidad. | Existe: `Profile` y `max_clauses` en `classify_candidate`. |
| A8 | Seguridad y tipado: toda variable de la cabeza está ligada en el cuerpo y no hay argumentos inventados. La laguna de `Relation.thing` se evita como en v2 (H6): sin records ni campos opcionales. | El elaborador, más la comprobación H6 de v2. |

Una regla rechazada por A1–A8 no se cuenta como refutada: queda registrada
con su código.

## 3. Experiencias iniciales

Las configuraciones son las de §2 del contrato mínimo. Siguen las reglas
R1–R8: constantes de instancia permitidas, objeto individualizado,
beneficiario con correspondencia jurídica, acuerdos `private` y alegaciones
fuera de $S$.

### 3.1 Cambio en Brookside

El acuerdo firmado es un hecho establecido, y su contenido también lo es. Por
eso ahora entra en $S_B$:

```text
Act(saB_j, jant, agree, saDocB, usa,  private, d20210216) @ tB
Act(saB_u, usa,  agree, saDocB, jant, private, d20210216) @ tB
obligationFiled(saB_j, jant, adoptPolicy,   policyA, usa, private, d20210216) @ tB
obligationFiled(saB_j, jant, distribute,    policyA, usa, private, d20210216) @ tB
obligationFiled(saB_j, jant, post,          policyA, usa, private, d20210216) @ tB
obligationFiled(saB_j, jant, keepAvailable, policyA, usa, private, d20210216) @ tB
obligationFiled(saB_j, jant, post,          signB,   usa, private, d20210216) @ tB
prohibition(saB_u, usa, sue, premisesB, jant, private, d20210216) @ tB
```

**Valoraciones de Brookside:**

| Posiciones | Etiqueta | Fuente y papel |
|---|---|---|
| `obligation` B1–B5, `prohibition` B6 | required | $O$ ya las deriva desde $S_B$. Son **controles**: verifican la proyección y no discriminan. |
| **B7–B11:** `right(jant, c, o, usa)` para cada una de las cinco obligaciones | required | Cláusula de no renuncia: «will not be construed a waiver of the United States' right to enforce other deadlines and provisions of this Agreement»; y «reserves the right to file a civil lawsuit to enforce this Agreement». **Propuestas: requieren tu revisión antes de congelar.** El paso de «right to enforce provisions» a pretensión de cumplimiento de cada estipulación es una interpretación mínima. |

La cláusula de no uso probatorio se respeta: B7–B11 solo puntúan en $S_B$.

### 3.2 Casa Fiesta y Flowers

- **Casa Fiesta:** sin cambios. $S_{CF}$ tiene los acuerdos y los dos actos realizados (con reserva sobre el beneficiario), y no hay posiciones valoradas.
- **Flowers:** sin cambios. $S_F$ no tiene átomos. Las posiciones valoradas son F1–F10, `obligation(r, transfer, feesF, a)`, todas excluded.

### 3.3 Qué puede comprobar cada expediente

| Expediente | V | C | P | Revisión | Otros |
|---|---|---|---|---|---|
| Brookside | Sí | Sí, en B7–B11 | Solo si se presenta después de generar la regla y sin haber recibido su discrepancia | No: no queda ningún expediente posterior con posiciones a las que la regla revisada se exponga | Controles B1–B6 |
| Casa Fiesta | Sí | No: no hay valoraciones | No | No | Cuenta consecuencias no valoradas (exceso de generalidad) |
| Flowers | Sí, de forma vacía | Vacío: F1–F10 no se pueden alcanzar sin `feesF` en $S_F$ | No | No | Ninguno |

**Caso concreto, `obligation(id,s,c,o,b,au,e) ⇒ right(id,s,c,o,b,au,e)`:**
- **V:** la he comprobado leyendo el código, no ejecutando.
  - El texto contiene solo reglas.
  - El elaborador acepta cabezas de verbos sembrados (las candidatas `prohibition` de v2).
  - `install` añade reglas sin declarar verbos.
  - En $S_B$ la clausura de referencia no produce `right`, así que `check_scope` la deja dentro del certificado.
  - Ninguna regla de $O$ se dispara con el `right` derivado, porque no hay `Act` ni `~Act` de esas prestaciones. El certificado de conservación se cumple.
- **C:** se cumple en B7–B11 si la regla se generó o revisó con la discrepancia de B.
- **P:** se cumple en B si la regla estaba comprometida antes de B, en el orden CF → F → B.
- **No puede refutarse con estos tres expedientes.** Ninguno contiene una posición `right` excluded ni una obligación sin pretensión documentada.

### 3.4 Observación adicional necesaria (no se amplía el corpus)

Para someter `obligation ⇒ right` a refutación y medir la revisión haría
falta:

| | Observación |
|---|---|
| O1 | **Un expediente donde $S$ establece o $O$ deriva `obligation(id,s,c,o,b,…)` y la fuente excluye que $b$ pueda exigir esa prestación.** Por ejemplo: una cláusula que niegue derechos al beneficiario nombrado, una obligación declarada inexigible o una deuda prescrita reconocida. Daría `right(s,c,o,b)` excluded. Tiene que presentarse después de generar la regla. |
| O2 | Al menos un expediente posterior a O1, no visto, con obligaciones en $S$ y alguna `right` o `obligation` valorada, para medir la pérdida prospectiva de la regla revisada. |
| O3 | Para la versión prospectiva de C: otro expediente no visto con `obligation` en $S$ y `right` required sobre otras partes y prestaciones. Aporta exposición más allá de Brookside. |

Sin O1 y O2, la condición con feedback y su control (§5) reciben la misma
información útil. La diferencia entre ambas es nula por construcción.

## 4. Verificación por Residuum

**Estado actual** (leído en `src/experiments/proposal.rs` y
`src/experiments/exploration.rs`):
- `admit` → `check_scope` → `evaluate_admitted`/`EvaluationSession` → `certify_addition`.
- El certificado actual exige una **extensión conservativa**: fuera de las cabezas declaradas, la clausura extendida tiene que tener los mismos eventos y las mismas justificaciones que la de referencia.
- Si la cabeza ya la produce la referencia en ese expediente, el resultado es `OutOfCertificateScope`.
- Si las consecuencias de la regla activan reglas de $O$ (por ejemplo, `right` derivado más `~Act` da `enforceability`), el resultado es `CertificateFailed`. Es un estado técnico y $D$ no se lee.

**Consecuencia:** las composiciones de varios pasos a través de $O$, que son
justamente lo que el piloto quiere permitir, hoy no se certifican.

**Modo de verificación propuesto, `attributed_extension`** (pendiente de implementar):
1. La clausura de $O\cup R_k$ sobre $S_i$ se completa dentro del presupuesto. Si no, el estado es `pending` y no hay veredicto.
2. Monotonía: todo evento de la clausura de referencia está en la extendida.
3. Atribución: cada evento nuevo, y cada justificación nueva de un evento existente, tiene al menos una derivación que usa alguna regla de $R_k$. Se registra qué regla y con qué testigos y sustituciones.
4. Se informan aparte:
   - las justificaciones redundantes de eventos que $O$ ya derivaba;
   - las cadenas a través de reglas de $O$ (qué reglas de $O$ se activaron por primera vez);
   - las consecuencias no valoradas.

El certificado conservativo sigue disponible como modo estricto. Para una
regla que lo cumpla, como `obligation ⇒ right` en $S_B$, los dos modos
coinciden.

`OutOfCertificateScope` y `CertificateFailed` nunca se convierten en
refutación ni en eliminación del espacio de hipótesis.

## 5. Protocolo

### 5.1 Orden y paso prequential

El orden de los expedientes se fija antes de cualquier ejecución.
- **Orden principal propuesto: CF → F → B.** Brookside queda como prueba prospectiva de lo generado sin feedback.
- **Orden secundario: B → CF → F.** Sirve para observar la generación a partir de la discrepancia de B. Su resultado esperado es `unexposed` en CF y F.

Para cada expediente $i$ en orden:

1. **Exploración.** El aprendiz ve $S_i$, la clausura de $T_k$ sobre $S_i$ y las herramientas de §6. Puede proponer, verificar y revisar reglas con un presupuesto de $n$ intentos, sin ninguna información de las valoraciones de $i$.
2. **Compromiso.** El controlador congela $R_k$ y calcula $N_k(S_i)$ y $\ell_i$ (pérdida prequential). Ese número es el que mide la capacidad predictiva.
3. **Feedback**, solo en la condición G+F. El controlador entrega la vista de discrepancia de $i$ (§6.3).
4. **Revisión.** El aprendiz dispone de $m$ intentos más. Las revisiones solo afectan a los expedientes futuros. Sobre los expedientes vistos se registra C, nunca P.

### 5.2 Condiciones

| Condición | Diferencia |
|---|---|
| **G+F** | Generación con feedback de discrepancias. |
| **G−F** (control) | El mismo generador, el mismo modelo, la misma semilla de muestreo si el proveedor lo permite y el mismo presupuesto $n+m$ de intentos. En el paso 3 solo recibe «expediente $i$ evaluado», sin contenido. Lo demás es idéntico, incluidas las herramientas. |

**Efecto de la refutación sobre la revisión:**
- la diferencia de pérdida prequential acumulada entre G+F y G−F en los expedientes posteriores a la primera discrepancia;
- la proporción de reglas `refuted` que se revisan y su pérdida posterior.

Si no hay ninguna discrepancia antes del último expediente, el efecto se
declara **no medible**. No se declara nulo.

## 6. Interfaces propuestas

Se construyen sobre las acciones del protocolo de invocación
(`docs/llm-explorer-invocation-protocol.md` §5). Los identificadores los
asigna el controlador.

### 6.1 Acciones del aprendiz

| Acción | Estado | Contrato |
|---|---|---|
| `inspect_theory(theory_ref)` | Existe en el protocolo | Muestra $O$ y $R_k$ con sus estados. |
| `view_dossier(dossier_id)` | Nueva, solo de lectura | Devuelve $S_i$ del expediente actual o de los ya vistos. Nunca el registro documental. |
| `begin_forward` / `advance_session` | Existen | Clausura de $T_k$ o de $T_k\oplus r$ sobre $S_i$. |
| `explain(event_ref)` | Nueva, sobre los registros de derivación ya conservados | Devuelve antecedentes ordenados, testigos, sustitución canónica y origen de cada regla (de $O$ o de $R_k$). |
| `propose_rule(text, parent_id?, move, rationale, expected_observation)` | Existe como `propose_candidate` más A3, A4 y A6 | Ejecuta las etapas `parsed` → `elaborated` → `extracted` → `admitted`. Devuelve el estado y el texto conservado. |
| `evaluate_rule(rule_id, dossier_id, mode)` | Existe en modo `conservative`; `attributed` es nuevo | Ejecuta la clausura hipotética aislada sobre un expediente ya visto o el actual. Devuelve el estado de §4 y las posiciones proyectadas. **Nunca devuelve pérdida ni valoraciones.** |
| `commit_hypotheses(rule_ids, retire_ids)` | Nueva | Fija $R_{k+1}$. Solo es válida en los pasos 1 y 4 de §5.1. |
| `minimal_supports`, `compare_coverage`, `partition`, `discriminants` | Existen en el protocolo (vista C) | Herramientas geométricas sobre los $S$ vistos y las teorías hipotéticas. Nunca reciben valoraciones como entrada. |
| `request_valuation` | Existe | **Desactivada en 004-R:** no hay valorador disponible. |

### 6.2 Registro `HypothesisRecord`

```text
id, canonical_form, original_text, parent_id?, move, origin (condición, expediente, paso),
admission_stages[], verification[{dossier, mode, status, attribution, unvalued_count}],
exposure[{dossier, position, prequential: bool}],
status ∈ {valid, compatible(n,e), prospective(n,e), refuted(...), unexposed,
          case_specific, superseded_by, pending(...)},
rationale, expected_observation
```

El registro se encadena por hash con el ledger existente
(`tools/normative_explorer/ledger.py`).

### 6.3 Vista de discrepancia `DiscrepancyView`

Solo en G+F, y solo después del compromiso de §5.1, paso 2.

```text
dossier_id,
missed:   [{position (cabeza, s, c, o, b), label: required}],
excess:   [{position, label: excluded, deriving_rules: [rule_id]}],
counts:   {valued_total, discrepant}
```

- Las posiciones valoradas sin discrepancia no se revelan, solo se cuentan.
- No incluye fuentes, citas, fundamentos ni el nombre del caso.

### 6.4 Canales

| El explorador puede consultar | El explorador no puede consultar |
|---|---|
| $S_i$ de los expedientes vistos | $P_i^\pm$ (valoraciones reservadas) |
| Clausuras | El registro documental: alegaciones, fundamentos, textos y citas |
| Derivaciones y provenance | Los nombres naturales, en la condición opaca |
| Antecedentes y sustituciones | El ledger de la otra condición |
| Herramientas geométricas | Los expedientes aún no presentados |
| Su propio registro | |
| Las vistas de discrepancia recibidas (solo G+F) | |

## 7. Criterios de evaluación

| Métrica | Definición | Rol |
|---|---|---|
| $L^{preq}$ | $\sum_i \ell_i(R_{k(i)})$ con la teoría comprometida antes de cada $i$, más la exposición total | **Principal:** capacidad predictiva |
| $\Delta L^{preq}$ | G−F menos G+F, en los expedientes posteriores a la primera discrepancia | Efecto de la refutación en la revisión |
| Revisión efectiva | Reglas `refuted` que se revisan y cuya versión revisada queda expuesta y con $\ell=0$ en un expediente posterior | Capacidad de revisión |
| Recuentos V / C / P | Número de reglas en cada estado, con $(n,e)$ | Descriptivo. Nunca se suman entre sí. |
| Exceso | Consecuencias no valoradas por regla y por expediente | Diagnóstico. No entra en $\ell$. |
| Trabajo | Intentos, rechazos por código, pasos de Residuum y llamadas | Coste |
| Contaminación | Resultado de las pruebas de §8 | Filtro de interpretación |

**Pérdida:**
- $\ell_i=|P_i^+\setminus N|+|P_i^-\cap N|$, solo sobre posiciones valoradas;
- las posiciones se comparan por identidad canónica proyectada en `(cabeza, s, c, o, b)`;
- un cierre incompleto (`incomplete`) no aporta pérdida ni certifica ausencias;
- la similitud de embeddings nunca entra en $\ell$.

**Pronóstico con el corpus actual**, antes de ejecutar nada:

| Métrica | Pronóstico |
|---|---|
| $L^{preq}$ en CF y F | 0 en cualquier ejecución |
| $L^{preq}$ en B | Entre 0 y 5, según el aprendiz haya generado o no una regla que produzca B7–B11 antes de ver B |
| $\Delta L^{preq}$ | No medible |
| Revisión efectiva | No medible |

El piloto puede mostrar generación, verificación y, a lo sumo, una
exposición prospectiva. No puede mostrar revisión guiada por refutación
(§3.4).

## 8. Filtración (definida, no ejecutada)

- **Vocabulario opaco:** las constantes de instancia se renombran de forma determinista, con la tabla sellada por hash. Los nombres de $O$ se mantienen, porque son la ontología fija. Se ejecutan las condiciones natural y opaca con el mismo presupuesto.
- **Reconocimiento de casos:** prueba prerregistrada sobre $S_i$, en versión natural y opaca, con distractores fijados de antemano. Si un expediente es reconocido se marca `contaminated` y su P se informa por separado.
- **Conocimiento doctrinal:** que el modelo proponga reglas conocidas, como la correlatividad, es legítimo y no se filtra. Lo que se mide es su comportamiento ante experiencias nuevas.

Las dos pruebas requieren llamadas pagadas y autorización expresa con
presupuesto.

## 9. Ablación de embeddings con catálogo fijo

- Es el Piloto 004 v2 tal como está congelado: 534 candidatas, cuatro expedientes y las condiciones B/C/D.
- Sigue pendiente de tus 12 valoraciones.
- Su función es aislar el efecto de las descripciones revisables y los embeddings en la ordenación.
- **No restringe al explorador principal**: el espacio de 004-R es $\mathcal L_{adm}$, no el catálogo.

## 10. Decisiones pendientes

1. Revisar y aprobar las valoraciones B7–B11 (`right`) y la entrada del contenido del acuerdo en $S_B$.
2. Orden de los expedientes: el principal propuesto, el secundario o ambos.
3. Presupuestos $n$ y $m$ y el perfil A7.
4. Autorizar la implementación de A3, A4, A6, `attributed_extension`, `view_dossier`, `explain`, `commit_hypotheses` y `DiscrepancyView`. Antes conviene decidir si merece la pena con un corpus que no puede medir la revisión (§3.4).
5. Corpus: buscar experiencias del tipo O1–O3 cuando lo autorices.

## 11. Lo que no se ha hecho

- No hay implementación ni fixtures.
- No se ha ejecutado el elaborador ni el explorador. La comprobación de V para `obligation ⇒ right` es solo lectura del código.
- No hay llamadas pagadas.
- No se han tocado el motor, LegalGeneral, las 534 candidatas ni los cuatro expedientes.
- No hay merge ni push.
