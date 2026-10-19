# Piloto 004-R: contrato mínimo sobre tres experiencias documentadas (propuesta v0)

Estado: **propuesta, no congelada**. Rama `experiment-pilot004-documentary`.
Fuentes: `docs/legal-geometry/pilot-004/documentary/sources/` (hashes en
`SOURCES.sha256`). El Piloto 004 v2 congelado (534 candidatas, cuatro
expedientes, `MANIFEST.sha256`) no se modifica y sirve solo como referencia
técnica.

Veredicto adelantado (§5): con las reglas de representación de §1, **las tres
experiencias no generan diferencias predictivas entre hipótesis admisibles
ni permiten ninguna refutación informativa**. §6 enumera exactamente lo que
falta. No se amplía el alcance.

## 0. Objetivo

Comprobar si la revisión semántica de nodos, guiada por refutaciones
verificables, mejora la navegación entre generalizaciones simbólicas. Este
contrato solo fija:
- qué se observa ($S_i$);
- qué se valora ($P_i^+,P_i^-$);
- cómo se mide la pérdida.

Antes de implementar nada, comprueba si esas tres piezas permiten
discriminar entre hipótesis.

## 1. Reglas de representación (fijadas antes del aprendizaje)

| | Regla |
|---|---|
| R1 | Solo se usan las firmas de LegalGeneral original. No hay verbos, entidades ni reglas nuevas. Lo que no cabe se excluye y se anota en §2 como pérdida. |
| R2 | Se declaran constantes de instancia de sorts existentes (`Person`, `Thing`, `Conduct`, `Fact`, `PropositionTime`), como hizo v2 con `enterWith`. No llevan verbos asociados, y su nombre no introduce semántica en el motor. **Decisión tuya:** si consideras ontología las constantes de `Conduct`, solo queda `transfer`/`enact`/`use`, y ninguna posición de §3 sobrevive salvo las de honorarios de Flowers. |
| R3 | `object : Thing` es obligatorio. Solo se acepta una cosa que el documento individualiza: el documento del acuerdo, la política del Anexo A, el cartel especificado o el establecimiento nombrado. Si la cosa sería una invención, la relación se pierde. |
| R4 | `beneficiary` solo se usa cuando hay acreedor jurídico defendible: la contraparte de un acuerdo o la parte favorecida por un pronunciamiento. No hay beneficiarios reflexivos. Un destinatario (a quien se dirige una petición o contra quien se demanda) no es beneficiario. |
| R5 | `authority`: un acuerdo bilateral es `private`, aunque una parte sea el Estado. Un pronunciamiento de tribunal es `judicial`. Ningún acuerdo se convierte en acto `administrative`. |
| R6 | $S_i$ contiene solo hechos establecidos por la fuente: firmados por ambas partes, admitidos o constatados por la autoridad que los emite. Las alegaciones van al registro documental (§2), fuera de $S_i$, y nunca producen valoraciones. |
| R7 | Las valoraciones son posiciones con cabezas existentes (`obligation`, `permission`, `prohibition`). `required` y `excluded` son etiquetas de comparación. Lo no valorado no se completa. |
| R8 | Con varios deudores y sin reparto documentado, no se valora como `required` ninguna posición individual: sería suponer solidaridad. Sí se valora como `excluded` una posición individual cuando la fuente excluye la de todos. |

## 2. Configuraciones Formal propuestas

Los fragmentos son texto del contrato. No están elaborados ni probados, y se
elaborarán cuando apruebes el contrato.

### 2.1 Brookside (DJ 202-14-282, firmado 16-2-2021)

**Constantes:**
- `jant : Person` (JANT, LLC; «Brookside Bar and Grill» es su d/b/a);
- `usa : Person`;
- `premisesB : Thing` (el establecimiento de Higganum);
- `saDocB : Thing` (el documento del acuerdo);
- `policyA : Thing` (Anexo A, incorporado por referencia);
- `signB : Thing` (cartel «Service Animals Welcome»);
- `saB_j, saB_u : Fact`;
- `d20210216 : PropositionTime`.

```text
// Establecido: firma de ambas partes; vigencia = fecha de la última firma.
Act(saB_j, jant, agree, saDocB, usa,  private, d20210216) @ tB
Act(saB_u, usa,  agree, saDocB, jant, private, d20210216) @ tB
```

Reserva: `saDocB` es el objeto instrumental del acto de acordar, no su
objeto jurídico, que son las prestaciones.

**Registro documental** (fuera de $S_B$):

| Grado | Contenido |
|---|---|
| Alegado | Al menos dos visitas con el animal; petición de identificación y certificación por una camarera y el dueño; al menos una vez se le pidió que se marchara. |
| Controvertido | Brookside niega no haberla dejado permanecer. |
| Admitido | Pidieron certificado o identificación porque el dueño no creía que el perro fuera de servicio. |

**Pérdidas de información:**

| Circunstancia | Motivo de la pérdida |
|---|---|
| Petición admitida | La petición tiene destinatario, no beneficiario (R4). No hay forma de expresar el contenido de una petición. |
| Creencia del dueño y relación causal | No hay primitiva. |
| Presencia con el animal | Alegada, y además sin beneficiario. |
| Cobertura del Título III y «discapacidad» | Son calificaciones, no hay primitiva. |
| Obligación de no discriminar y de modificar políticas | No hay cosa individualizada. |
| Formación | No hay cosa: la guía FAQ es material de la formación, no objeto de la prestación. |
| Confirmaciones escritas y notificaciones | Sin objeto individualizado. |
| Plazos de 14, 30 y 60 días, duración de dos años, renovación anual | No hay plazos en días ni vigencia final. |
| Reserva de demandar para ejecutar el acuerdo | El demandado no es beneficiario (R4). |
| Cláusula de no interpretación y de no uso probatorio | Sin primitiva. Se conserva como restricción del contrato, véase §3. |

### 2.2 Casa Fiesta (comunicado USAO EDKY, 6-8-2020)

Todo lo que entra en $S_{CF}$ procede de una afirmación oficial de una de las
partes (grado R). No es una constatación judicial ni administrativa.

**Constantes:**
- `casaFiesta : Person`;
- `usa : Person`;
- `saDocCF : Thing`;
- `policyCF : Thing`;
- `signCF : Thing`;
- `saCF_c, saCF_u, cfA1, cfA2 : Fact`;
- `dCF : PropositionTime`, fecha no publicada y anterior o igual al 6-8-2020; no se le añade ningún orden.

```text
Act(saCF_c, casaFiesta, agree, saDocCF, usa,        private, dCF) @ tCF
Act(saCF_u, usa,        agree, saDocCF, casaFiesta, private, dCF) @ tCF
// «As part of the settlement, Casa Fiesta adopted … and posted signage»
Act(cfA1, casaFiesta, adoptPolicy, policyCF, usa, private, dCF) @ tCF
Act(cfA2, casaFiesta, post,        signCF,   usa, private, dCF) @ tCF
```

Reserva sobre `cfA1`/`cfA2`: el beneficiario `usa` descansa solo en «as part
of the settlement». Si no lo aceptas, ambos actos salen de $S_{CF}$, y las
conclusiones de §5 no cambian.

**Registro documental:** se alega que el personal exigió documentación y que
intentó sentarla aparte o fuera.

**Pérdidas de información:**
- la formación del personal (no hay cosa);
- la persona demandante y su animal (alegados);
- el contenido del acuerdo (no está disponible y no se busca, por tu instrucción).

### 2.3 Flowers v. Prasad (B260140, 17-7-2015)

**Constantes:**
- `john, seth : Person`;
- `bPrasad, vic, rPrasad, sappaniapillai, abraham : Person` (las cinco partes apeladas);
- `feesF, costsF : Thing` (importes de honorarios y de costas de apelación);
- `opF : Fact`;
- `d20150717 : PropositionTime`.

```text
// S_F no contiene átomos establecidos y representables (véase abajo).
```

**Registro documental:**
- **Alegado:** todo el sustrato fáctico. Prasad sería titular de Valley India Café, John sería discapacitado con un perro de servicio con licencia, y a ambos se les habría negado el servicio «due to the service dog».
- **Establecido (historia procesal):**
  - la demanda del 26-9-2012;
  - la estimación del demurrer y la summary adjudication sobre Unruh;
  - el desistimiento con perjuicio a petición de los apelantes, para recurrir;
  - la sentencia del 17-10-2014;
  - la revocación con instrucciones del 17-7-2015.

**Pérdidas de información:**

| Circunstancia | Motivo de la pérdida |
|---|---|
| Demanda, recurso y desistimiento | El demandado es destinatario, no beneficiario (R4). El objeto, la pretensión, no es una cosa. |
| Resoluciones de instancia y su revocación | `annulled` solo se deriva de conflictos entre `permission`/`prohibition`/`obligation`, y las resoluciones no lo son. |
| Instrucciones de dictar nuevas resoluciones | Su objeto es una resolución futura: sería un argumento inventado. |
| «Pueden accionar por Unruh y por la DPA» | Es una facultad procesal. `permission` es permiso de conducta propia, y una conducta `sueUnruh` llevaría el fundamento en el nombre. |
| Costas a los apelantes | Cinco deudores sin reparto (R8). |

## 3. Posiciones observables y valoraciones respaldadas

Proyección para comparar: `(cabeza, subject, conduct, object, beneficiary)`.
Se descartan `id`, `authority` y `effective`, como en v2 §6. `authority` y
`effective` constan en la fuente y se conservan como metadata.

| id | Posición | Etiqueta | Fuente y alcance |
|---|---|---|---|
| B1 | `obligation(jant, adoptPolicy, policyA, usa)` | required | Acuerdo, «Service Animal Policy» (a). Contractual, durante la vigencia. |
| B2 | `obligation(jant, distribute, policyA, usa)` | required | Ídem (b). Los empleados son destinatarios, no acreedores. |
| B3 | `obligation(jant, post, policyA, usa)` | required | Ídem (d). |
| B4 | `obligation(jant, keepAvailable, policyA, usa)` | required | Ídem (e). |
| B5 | `obligation(jant, post, signB, usa)` | required | «Signage». |
| B6 | `prohibition(usa, sue, premisesB, jant)` | required | «refrain from filing a civil suit … at the Brookside Bar and Grill». `premisesB` es objeto como materia del pleito (reserva R3). |
| F1–F10 | `obligation(r, transfer, feesF, a)` para cada `r` ∈ {5 apeladas} y `a` ∈ {john, seth} | excluded | Nota 12: «we reject it». Alcance: la apelación, a 17-7-2015. |

Sin valorar:
- todo Casa Fiesta (el comunicado describe actos realizados, no obligaciones);
- las costas de Flowers (R8);
- toda posición sobre la conducta en el local.

**Restricción de uso:** la cláusula «compromise … shall not be used or
introduced into evidence in any other case» de Brookside se respeta así:
B1–B6 solo puntúan en $S_B$ y nunca se presentan como interpretación de la
ADA.

## 4. Procedimiento de pérdida

1. $N_h(S_i)$ = posiciones proyectadas de `close_program`$(T\oplus h, S_i)$.
   Si el cierre no termina, el expediente cuenta como `incomplete`: no aporta
   pérdida ni certifica ausencias.
2. $\ell_i(h)=|P_i^+\setminus N_h(S_i)|+|P_i^-\cap N_h(S_i)|$. Solo se cuentan
   posiciones valoradas, y la igualdad se decide por identidad canónica.
3. $L(h)=\sum_i \ell_i(h)$ sobre los expedientes cerrados. Se registra también
   el vector $(\ell_B,\ell_{CF},\ell_F)$.
4. La similitud de embeddings nunca entra en $\ell$. Solo ordena nodos en la
   política de navegación, como en v2.
5. Cada valoración lleva `source = document`, la cita y el alcance. No existen
   valoraciones `source = model`.

## 5. Análisis de suficiencia (previo a toda implementación)

**Clase de hipótesis.** Se toma la forma de v2 adaptada a $T$ = LegalGeneral:

```text
Act(a,s,c,o,b,au,e) @ t & L₁ @ t [& L₂ @ t]  <=  K(hypNorm, s, c, o, b, au', e') @ t
K ∈ {obligation, permission, prohibition}
```

Los literales `L` están sobre los verbos sembrados que aparecen en algún
$S_i$. Aquí solo aparece `Act`, así que los literales son `[~]Act` con
argumentos ligados o constantes de $S$.

**Resultados:**

1. **Casa Fiesta:** $P^+_{CF}=P^-_{CF}=\varnothing$, luego $\ell_{CF}(h)=0$ para toda $h$. No puede refutar nada.
2. **Flowers:** $S_F$ no tiene átomos, así que ninguna regla anclada en `Act` dispara. Entonces $N_h(S_F)=\varnothing$ y $\ell_F(h)=0+0=0$ para toda $h$. Las exclusiones F1–F10 son correctas pero no se pueden violar.
3. **Brookside:** $S_B$ solo contiene los dos actos `agree` sobre `saDocB`. Una cabeza anclada repite `(s,c,o,b)` del acto, de modo que solo puede producir posiciones `(·, jant|usa, agree, saDocB, ·)`. Ninguna coincide con B1–B6. Por tanto $\ell_B(h)=6$ para toda $h$ admisible.
4. **Si se admiten constantes en la cabeza:**
   - las únicas $h$ con $\ell_B<6$ son las que escriben `adoptPolicy`, `policyA`, `signB` o `premisesB` en la cabeza;
   - esas constantes solo aparecen en el resultado de Brookside, así que la regla transcribe la decisión (vulnera «no introduzcas predicados cuyo significado incorpore la decisión del caso»);
   - en Casa Fiesta y Flowers esas reglas no tienen nada valorado que acertar o fallar.
5. **Validación cruzada dejando uno fuera:**
   - al apartar B, el entrenamiento con CF y F da pérdida 0 a toda $h$: no hay señal, y la selección es arbitraria;
   - al apartar CF o F, la prueba da pérdida 0 a toda $h$.

   No existe ninguna partición en la que una refutación en entrenamiento cambie la pérdida esperada en prueba.
6. **Revisión semántica (condición C):** la única discrepancia observable (B1–B6 no derivadas) es idéntica para todas las candidatas admisibles. Una revisión de $h_k$ no puede cambiar de forma verificable la ordenación, porque $L$ es constante. Las condiciones B, C y D serían indistinguibles por construcción, no por un resultado empírico.
7. **Puente aparente entre CF y B:** `adoptPolicy` y `post` aparecen como actos en CF y como obligaciones en B, pero no forman un puente. Una regla «lo realizado en ejecución de un acuerdo era debido» predice en CF posiciones sin valorar y en B no dispara, porque $S_B$ no contiene actos de ejecución.

**Conclusión:** el $L(h)$ admisible es constante, igual a $(\ell_B,\ell_{CF},\ell_F)=(6,0,0)$. El corpus simplificado no permite ninguna refutación informativa.

## 6. Qué falta exactamente

Ninguno de estos puntos se ha resuelto ni buscado. Son condiciones mínimas y
cada una por sí sola no basta.

| | Falta | Por qué es necesaria | Situación en el corpus |
|---|---|---|---|
| M1 | Un expediente de prueba con posiciones valoradas del mismo tipo (cabeza, conducta y clase de objeto) que en un expediente de entrenamiento | Sin vocabulario compartido entre lo valorado en entrenamiento y en prueba, ninguna generalización se puede contrastar fuera de la muestra. | Lo valorado está solo en B (contractual) y en F (honorarios, negativo). Comparten cabeza, pero no conducta, objeto ni partes. CF no tiene nada valorado. |
| M2 | Que las posiciones valoradas tengan sus argumentos anclados en hechos establecidos de $S$ distintos del instrumento que las constituye | Si el instrumento con su contenido está en $S$, `filedObligation` deriva la posición en todas las candidatas (pérdida 0 constante). Si no está, nada en $S$ liga sus argumentos (pérdida máxima constante). | Ocurre lo mismo en B. En F, $S_F$ está vacío. |
| M3 | Variación de resultado: la misma posición proyectada debe ser `required` en un expediente y `excluded` en otro con configuración comparable | Sin ella, una regla que siempre afirma o siempre calla es óptima y no hay nada que refutar. | Todas las valoraciones de B son `required` y todas las de F son `excluded`, sobre posiciones distintas. |
| M4 | Una representación fiel del incidente (peticiones con destinatario, presencia con animal, motivo) | Es la configuración cuya generalización interesa. Sin ella, solo se observan instrumentos y su propio contenido. | LegalGeneral no tiene ningún rol de destinatario ni contenido de petición, y R1 prohíbe añadirlos. Es la carencia decisiva. |

**Consecuencia:** con las condiciones fijadas (sin casos nuevos, sin
primitivas nuevas, sin alegaciones en $S$), ninguna reordenación de estos
tres asuntos satisface M1–M3. M4 lo excluye directamente R1.

Para hacer el corpus evaluable haría falta al menos una de estas
autorizaciones, que **no** se toman aquí:
- ampliar el corpus (M1, M3);
- admitir alegaciones en $S$ marcadas como hipótesis de trabajo (M2 parcial);
- extender la firma (M4).

## 7. Protocolo de filtración (no ejecutado)

Los controles quedan definidos, pero no tiene sentido ejecutarlos mientras
§5 sea negativo.

**Vocabulario opaco:**
- todas las constantes de §2 se renombran de forma determinista (`p01…`, `c01…`, `o01…`, `f01…`, `d01…`) con una tabla sellada por hash antes de cualquier ejecución;
- las descripciones $d_0$ se generan con los nombres opacos;
- se compara la condición natural con la opaca, con la misma semilla y la misma política.

**Reconocimiento de casos:** prueba prerregistrada, pagada y pendiente de autorización.
- Al modelo se le da $S_i$ en las dos versiones, natural y opaca, sin el registro documental.
- Se le pide que identifique la jurisdicción, el caso o el resultado.
- Se compara con el azar y con una lista de distractores fijada de antemano.
- Si reconoce un caso, ese expediente se marca `contaminated` y se informa por separado.

**Canales:**

| Canal | Qué contiene | Quién lo ve |
|---|---|---|
| Factual | $S_i$ | El modelo puede verlo. |
| Valoración | $P_i^\pm$ | Solo lo ve el evaluador. El modelo recibe únicamente la vista de discrepancia de la condición C. |
| Fundamentos | El registro documental y las citas jurídicas | El modelo nunca los ve. |

## 8. Lo que no se ha hecho

- No se ha elaborado ningún fragmento Formal ni se ha escrito ningún fixture.
- No se han tocado el motor, LegalGeneral, las 534 candidatas ni los cuatro expedientes de v2.
- No se ha implementado el ciclo LLM ni se han hecho llamadas pagadas.
- No se ha buscado el acuerdo de Casa Fiesta.
- No hay merge ni push.
