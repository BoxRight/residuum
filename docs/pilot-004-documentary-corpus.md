# Piloto 004 · Corpus documental: tres experiencias reales, su expresividad y sus valoraciones posibles

Estado: **INVESTIGACIÓN DOCUMENTAL Y FORMALIZACIÓN PRELIMINAR; PENDIENTE DE REVISIÓN**.

| | |
|---|---|
| Fecha | 2026-10-19 |
| Rama aislada | `experiment-pilot004-documentary`, creada desde `experiment-legal-order-geometry` en `5d08310` |
| Copia de trabajo | `/tmp/residuum-pilot004-documentary` |
| Fuentes archivadas | `docs/legal-geometry/pilot-004/documentary/sources/` (originales, extracción de texto y `SOURCES.sha256`) |

**Qué no se ha hecho:**
- No se ha modificado el contrato v2 del Piloto 004.
- No se han modificado las fixtures, las 534 candidatas, LegalGeneral ni el motor.
- No se ha ejecutado el explorador ni se han hecho llamadas a modelos.
- No hay merge ni push.
- Los bocetos formales de §3 son texto: **no se han elaborado ni probado**.

**Convenciones de grado de certeza:**

| Grado | Significado |
|---|---|
| **D** | Determinación jurídica de la autoridad (dispositivo o *holding*). |
| **A** | Hecho admitido por la parte a la que perjudica, en un documento firmado por ella. |
| **R** | Afirmación de la autoridad en su propio nombre, fuera de un dispositivo. Por ejemplo, la Fiscalía relata que el restaurante «adoptó» una política. |
| **L** | Alegación de una parte. |
| **C** | Hecho controvertido: alegado y negado. |
| **F** | Fundamento: enunciado jurídico general expresado por la autoridad. Pertenece al canal de fundamentación. |
| **?** | Cuestión no resuelta. |
| **—** | No documentado. |

---

## 1. Catálogo documental

### 1.1 Flowers v. Prasad

| | |
|---|---|
| Autoridad | Court of Appeal of the State of California, Second Appellate District, Division Four |
| Resolución | opinión publicada («CERTIFIED FOR PUBLICATION») que resuelve una apelación contra una sentencia de sobreseimiento (*judgment of dismissal*) |
| Número | B260140; Superior Court de Los Ángeles, PC053779 |
| Cita | 238 Cal. App. 4th 930; 190 Cal. Rptr. 3d 33 |
| Fecha | presentada el 17-7-2015 |
| Ponente | Manella, J. (con Willhite, Acting P. J., y Collins, J.) |
| Fuente primaria | https://www.courts.ca.gov/opinions/archive/B260140.PDF, archivada como `flowers_v_prasad_B260140.pdf` (SHA-256 `c7f81846…`) |
| Verificación | texto íntegro leído; 17 páginas |

**Aviso sobre las fuentes secundarias.** Un despacho de abogados (hbblaw.com, «That Dog Can Hunt», 22-7-2015) llama a los demandantes «John Smith and his son Seth Smith» y sitúa el restaurante en Rohnert Park. La opinión dice **Flowers** y **Canoga Park**. Solo se usa la opinión.

### 1.2 Casa Fiesta Mexican Restaurant (Richmond, Kentucky)

| | |
|---|---|
| Autoridad | U.S. Attorney's Office, Eastern District of Kentucky, en coordinación con la Disability Rights Section del DOJ |
| Documento disponible | **solo el comunicado de prensa** «Richmond Restaurant Settles ADA Complaint», de jueves 6-8-2020 |
| Fuente primaria | https://www.justice.gov/usao-edky/pr/richmond-restaurant-settles-ada-complaint. El sitio bloquea la descarga automatizada (desafío anti-bot), por lo que se usó la copia de Internet Archive del 13-5-2021 y el PDF adjunto `2020-08-06_ada_settlement_casa_fiesta.pdf` (copia del 17-9-2024, SHA-256 `ec2f0edc…`). |
| Acuerdo | **no localizado**. No figura en las listas de archive.ada.gov (`enforce_current`, `enforce_archive`, `enforce_activities`), y el adjunto del comunicado es el propio comunicado, no el acuerdo. |
| Verificación | comunicado leído íntegro en dos copias coincidentes (HTML y PDF); acuerdo **no verificado** |

### 1.3 JANT, LLC d/b/a Brookside Bar and Grill (Higganum, Connecticut)

| | |
|---|---|
| Autoridad | U.S. Attorney's Office, District of Connecticut |
| Documento | Settlement Agreement, DJ # 202-14-282 |
| Firmas | Ndidi N. Moses (AUSA), por los EE. UU.; Emmanuel Mavredakis (Member), por JANT; ambas de 16-2-2021. La fecha efectiva es la de la última firma. |
| Vigencia | dos años |
| Fuente primaria | https://archive.ada.gov/brookside_bar_sa.html, archivada como `brookside_bar_sa.html` (SHA-256 `1084f547…`) |
| Verificación | texto íntegro leído. El «Attachment A» (la política) **no está** en la página publicada. La conversión HTML pierde la numeración de párrafos, aunque el texto remite a «paragraph 11». |

---

## 2. Reconstrucción de hechos y determinaciones

### 2.1 Flowers v. Prasad

**Hechos.** Todos proceden de la demanda. En apelación de un *demurrer* el tribunal **no establece** ninguno. La propia opinión los introduce como alegaciones: «The complaint alleges…», «The complaint further alleges…».

| Afirmación | Grado | Cita |
|---|---|---|
| Brinda Prasad es propietaria de Valley India Café, Inc., que explota un restaurante en Canoga Park | L | «The complaint alleges that respondent Brinda Prasad owns respondent Valley India Café, Inc., which operates a restaurant in Canoga Park.» |
| John es una persona con discapacidad con un perro de servicio con licencia | L | «John is a disabled person with a licensed service dog» |
| John y su hijo Seth fueron privados del servicio en el restaurante a causa del perro | L | «he and his son Seth have been denied service at the restaurant due to the service dog» |
| Seth es hijo de John | L (no discutido en la opinión) | «his son Seth» |
| Fecha del incidente | — | La opinión no la indica; la demanda se presentó el 26-9-2012. |
| Otros demandados (Rajendra Prasad, Sappaniapillai, Abraham) | procesal | añadidos como demandados Doe |

**Historia procesal (hechos del proceso, D o R del tribunal):**
- **Septiembre de 2013:** el tribunal de instancia estima el *demurrer* contra la pretensión basada en la Unruh Act, por considerar que la DPA es más específica e incompatible con ella. Da plazo para enmendar **solo a Seth**, que no enmienda.
- **Marzo de 2014:** concede la *summary adjudication* a los demás demandados por el mismo motivo.
- **Septiembre de 2014:** los demandantes piden el sobreseimiento con perjuicio «to expedite appellate review». El 17-10-2014 se dicta la sentencia de sobreseimiento.

**Determinaciones de la apelación (D):**
1. **Apelabilidad.** El sobreseimiento pedido para acelerar la apelación es apelable (doctrina *Austin v. Valverde*).
2. **Cuestión de derecho.** «because there is no irreconcilable inconsistency between the Unruh Act and the DPA, a plaintiff alleging disability discrimination relating to the use of a service dog may assert claims under both statutory schemes, subject to any specific limitations stated in them.»
3. **Dispositivo.** «The judgment is reversed, and the matter is remanded with directions to vacate the sustaining of the demurrer and grant of summary adjudication regarding appellants' Unruh Act claim, to enter new orders denying the demurrer and motion for summary adjudication, and to conduct further proceedings…»
4. **Costas.** Se conceden las costas de la apelación a los apelantes. Se rechazan los honorarios de abogado por falta de argumentación (nota 12).

**Fundamentos expresados (F, canal de fundamentación).** Son enunciados generales; ninguno se aplica a hechos probados.
- «the Unruh Act encompasses the ADA protections for disabled individuals who use service dogs.»
- «a public accommodation must make reasonable modifications to its policies, practices, and procedures to permit the use of service dogs by disabled individuals.»
- «denying a disabled person access to a public accommodation due to that person's service dog constitutes a **potential** violation of the ADA.» El adjetivo *potential* es del tribunal.
- Canon de especialidad (CCP § 1859): solo se aplica si las normas son irreconciliables; hay que armonizarlas y presumir que no hay derogación tácita.
- Historia legislativa de las cuantías mínimas: 4.000 USD (Unruh) frente a 1.000 USD (DPA), y la prohibición de doble recuperación del § 54.3(c).

**Cuestiones no resueltas (?):**
- si hubo negativa de servicio y por qué motivo;
- si John tiene una discapacidad en sentido legal;
- si el perro es un perro de servicio;
- la responsabilidad de cada demandado y los daños;
- las pretensiones por *intentional infliction* y *negligent hiring*;
- la posición propia de Seth como acompañante. La instancia lo trató de forma distinta, y la apelación anula el *demurrer* sobre «appellants' Unruh Act claim» sin pronunciarse sobre él por separado.

**Resultado posterior a la devolución:** no localizado.

**Lectura.** Es una decisión sobre la **viabilidad jurídica de una vía de reclamación**. No determina nada sobre la conducta del restaurante.

### 2.2 Casa Fiesta (Kentucky, 2020)

| Afirmación | Grado | Cita |
|---|---|---|
| Una persona con discapacidad que depende de un animal de servicio intentó comer en Casa Fiesta | R/L (descripción de la denunciante por la Fiscalía) | «an individual with disabilities, who relies on a service animal and attempted to eat at Casa Fiesta» |
| El personal le exigió documentación que probara que su perro era un animal de servicio | **L** | «The complainant alleged that restaurant staff demanded that she provided documentation proving that her dog was a service animal.» |
| El personal intentó sentarla apartada de otros clientes **o** fuera | **L** (la frase depende de «alleged») | «Staff also attempted to seat her away from other patrons or outside.» |
| Si se le negó el servicio, si comió o dónde se sentó | — | No consta. |
| Casa Fiesta celebró un acuerdo con el Gobierno federal para resolver la denuncia | R | «entered into a settlement agreement with the federal government to resolve a recent complaint alleging violations» |
| Casa Fiesta adoptó una política sobre animales de servicio, formó al personal y colocó carteles | **R** (presentado como ya realizado) | «As part of the settlement, Casa Fiesta adopted a Service Animal Policy, trained its staff members…, and posted signage permitting service animals.» |
| Admisión de hechos o de responsabilidad | — | El comunicado no la menciona. |
| Plazos, vigencia y demás términos del acuerdo | — | El acuerdo no está disponible. |
| Fecha del incidente | — | «a recent complaint» |

**Determinaciones (D):** ninguna. No hay pronunciamiento de ninguna autoridad sobre los hechos.

**Fundamentos expresados (F).** Son enunciados generales de la Fiscalía en el comunicado:
- deber de modificar políticas;
- definición del animal de servicio;
- acceso a todas las zonas abiertas al público;
- prohibición de preguntar por la discapacidad y de exigir documentación de certificación, adiestramiento o licencia.

### 2.3 Brookside Bar and Grill (Connecticut, 2021)

| Afirmación | Grado | Cita |
|---|---|---|
| La denunciante es una persona con discapacidad | R/L (así se presenta a la denunciante) | «a complaint from an individual with a disability who alleged…» |
| En al menos dos ocasiones, al entrar con su animal de servicio, una camarera y el propietario le pidieron que mostrara identificación y certificación del animal | **L** (la Fiscalía relata lo que alegó la denunciante) | «on at least two occasions, when entering Brookside Bar and Grill with her service animal, the complainant was approached by a waitress and the owner of the restaurant who asked the complainant to produce identification and certification for her service animal» |
| En al menos una ocasión le pidieron que se fuera | **L**, cuestionada (ver la fila siguiente) | «and on at least one occasion was asked to leave» |
| Brookside niega no haberle permitido quedarse | **C** | «The Brookside Bar and Grill denies that it did not permit the complainant to remain in the restaurant with her service animal» |
| Brookside admite que pidió ver un certificado o una identificación del animal | **A** | «but admits that they did ask to see a certificate or identification for the service animal» |
| Motivo admitido: el propietario no creía que el perro fuera un animal de servicio | **A** (estado mental) | «because at the time of the incident(s), the owner did not believe the dog was a service animal» |
| Brookside es un establecimiento de comida y bebida y un *public accommodation*; JANT es su propietaria y explotadora | A/R (recital firmado por ambas partes) | «a food and beverage service establishment, is a place of public accommodation covered by Title III» |

Que el animal sea en efecto un animal de servicio **no está establecido**. El texto usa la expresión «her service animal», pero la admisión dice que el propietario no lo creía, y nadie determina el adiestramiento.

**Obligaciones documentadas.** Son **D-contractuales**: constan en un acuerdo firmado y vinculante («This Agreement shall be binding on JANT, LLC…»), pero no son determinación de ninguna autoridad sobre los hechos.

| Obligado | Contenido | Plazo |
|---|---|---|
| Brookside | «will not discriminate … on the basis of disability»; «will modify policies, practices, or procedures to permit the use of service animals» | vigencia del acuerdo |
| JANT y Brookside | adoptar la Service Animal Policy (Attachment A, no publicado); distribuirla a los empleados con trato con el público; entregarla a los nuevos empleados; colocarla a la vista de los empleados; tenerla a disposición de los clientes | 14 días |
| JANT | confirmar por escrito a los EE. UU. la difusión y la colocación de la política | 30 días |
| Brookside | redistribuir anualmente la política y confirmarlo | anual |
| JANT | formación de propietarios, gerentes y empleados, incluida la FAQ del DOJ; notificarla | 60 días, y después anual |
| Brookside | cartel «Service Animals Welcome» con especificaciones de tamaño, contraste y altura | 30 días |
| EE. UU. | «agrees to refrain from filing a civil suit for failure to comply with Title III of the ADA at the Brookside Bar and Grill», con reserva de la acción para hacer cumplir el acuerdo | vigencia del acuerdo |
| EE. UU. | notificar por escrito el incumplimiento; JANT dispone de 30 días para subsanarlo o responder | — |

**Cláusula de alcance (decisiva para las valoraciones).** «nothing in this Agreement is intended to constitute an interpretation of the legal requirements of the ADA by the United States. Rather, the parties enter into this Agreement for the purpose of compromising disputed claims… This Agreement is a compromise and it shall not be used or introduced into evidence in any other case or proceeding…»

**Fundamentos (F).** Son recitales generales de los antecedentes:
- solo se permiten dos preguntas, salvo cuando sea «readily apparent»;
- se prohíbe exigir documentación.

Los dos se citan como 28 C.F.R. § 36.302(c)(7), aunque en el reglamento vigente las preguntas y la documentación están en (c)(6).

**Determinaciones (D) sobre los hechos:** ninguna.

### 2.4 Qué relaciones comparten realmente los tres casos

**Comunes:**
- una persona presentada como persona con discapacidad;
- un perro presentado como animal de servicio;
- un establecimiento de comida;
- una conducta del establecimiento hacia esa persona motivada (según la alegación) por el perro.

**No comunes:**

| | Flowers | Casa Fiesta | Brookside |
|---|---|---|---|
| Conducta | negativa de servicio (L) | exigir documentación (L); intento de sentarla aparte o fuera (L) | pedir identificación o certificado (A); pedir que se vaya (L, cuestionada) |
| Acompañante | sí (hijo) | — | — |
| Estado mental del local | — | — | no creía que fuera un animal de servicio (A) |
| Número de ocasiones | — | — | al menos dos |
| Vía | proceso civil estatal | denuncia administrativa federal y acuerdo | denuncia administrativa federal y acuerdo |
| Derecho invocado | Unruh y DPA (California), que incorporan la ADA | ADA, título III | ADA, título III |
| Desenlace documentado | la vía Unruh es viable | acuerdo con medidas ya realizadas | acuerdo con obligaciones futuras |

**No forman una secuencia de excepciones.** Ninguno documenta un tratamiento que contradiga a otro. Los tres terminan a favor del acceso, en formas no comparables: viabilidad procesal, medidas voluntarias y compromiso contractual. **No hay tratamientos distintos que contrastar.**

---

## 3. Mapa de expresividad hacia LegalGeneral

Categorías del encargo:
- **1:** composición de construcciones existentes;
- **2:** composición mediante un programa definicional;
- **3:** extensión descriptiva genuina;
- **4:** sin representación fiel.

Se aplican los criterios ya aprobados:
- se excluyen los argumentos ficticios;
- `Act` exige referentes reales;
- `constitutes` y `certifies` no tienen efectos normativos;
- no se usa el patrón inseguro `Relation.thing`.

| Hecho o relación | Categoría | Construcción, o motivo |
|---|---|---|
| Personas: denunciante, propietario, camarera, hijo, la Fiscalía como parte del acuerdo | 1 | `Person` |
| Persona jurídica que explota el local (JANT, Valley India Café, Inc., Casa Fiesta) | 1 como actor | `Person` |
| Persona jurídica a la vez como objeto de propiedad («Prasad owns … Café, Inc.») | **4** | Un tipo por entidad: no puede ser a la vez `Person` y `Thing`. |
| El perro | 1 | `Thing`, más `kind(dog, animal)` del vocabulario del piloto (3, ya existente) |
| El local o restaurante | 1 | `Thing`; titularidad con `patrimonialHolding` → `owns` si está documentada (para JANT es A/R: «owner and operator») |
| «Persona con discapacidad» | 3 con reserva | `condition(p, disability)` requiere una constante nueva. Su significado es una **calificación jurídica** (ADA); ningún documento precisa la condición concreta. Conviene marcarlo como calificación alegada, no como hecho natural. |
| «Animal de servicio» (adiestrado para tareas) | **4** | Es la calificación jurídica en disputa (Brookside: el propietario no lo creía). Su base descriptiva (adiestramiento, tarea) **no está documentada** en ningún caso. Un predicado `serviceAnimal(dog)` incorporaría la cuestión decisiva. |
| «Licensed service dog» (Flowers) | 4 | `certifies` exige un `Act` de certificación con autor y autoridad reales; no constan. |
| Persona acompañada del perro / entrar con el perro | 1 con reserva | `Act(e, p, enterWith, dog, b, private, d)` necesita un beneficiario. Usar `b = p` es una **convención reflexiva**; cualquier otro beneficiario sería forzado (excluido en `legal-semantic-domain-primitives.md` §10.2). **Decisión pendiente.** |
| Pedir que muestre identificación o certificado (Brookside, A) | 1 + 3 | `Act(r, owner, request, dog, owner, private, d)` (beneficiario reflexivo) junto con una extensión nueva `requests(act : Fact, r : Relation)` con contenido `Relation { debtor = complainant, thing = dog, conduct = showCertification, creditor = owner, mode = Doing }`, **sin efectos normativos**, por analogía con `constitutes`. No se puede usar `constitutes`: una petición no constituye el vínculo. |
| «Demanded» frente a «asked» (exigir frente a pedir) | **4** | La diferencia entre exigir y preguntar es precisamente la que importa para el fundamento («prohibits requiring documentation»). LegalGeneral no tiene modalidad ilocutiva. Representarla con dos constantes de conducta deja la distinción en una etiqueta. |
| Pedir que se vaya | 1 + 3 | como la petición anterior, con conducta `leave`; grado L y cuestionado |
| Intento de sentarla aparte **o** fuera | **4** | Disyunción en el hecho (K monótona sin disyunción) más un intento, no una conducta consumada. Solo una de las ramas podría representarse, y eso sería completar el expediente. |
| «Away from other patrons» como relación espacial | 3 parcial | `copresent` del piloto describe el resultado, no el intento; el resultado no consta. |
| Negativa de servicio «due to the service dog» | **4** en el motivo | La conducta puede ser un `Act` (`refuseService`) o un `~Act` de servir (sin equivaler a omisión). El **motivo o causalidad** («due to») no es representable, y es el núcleo de la alegación. |
| «At least two occasions» | 1 | Dos `Act` con fechas simbólicas sin orden. La monotonía de K expresa naturalmente la cota inferior («al menos»). |
| Estado mental: el propietario no creía | **4** | Sin actitudes epistémicas; la intención ya figuraba como «sin representación fiel». |
| Parentesco padre-hijo (Flowers) | 3 | `parentOf(p, q)`; extensión nueva, no usada en el piloto |
| Acuerdo como acto (Brookside, Casa Fiesta) | 1 | `Act(sa, jant, agree, premises, us, private, d)`. El objeto `premises` está documentado (el acuerdo se refiere al local); la autoridad, `private` o `administrative`, es una **decisión pendiente**. |
| Obligaciones del acuerdo (Brookside) | 1 | `obligationFiled(sa, jant, adoptPolicy, policyA, us, A, eff)` → `obligation` por `filedObligation`, y `do(...)`. Para Casa Fiesta, **—** (texto no disponible). |
| Contenido «will not discriminate on the basis of disability» | **4** en el contenido | La conducta «discriminar» es un término jurídico abierto; como constante de `Conduct` sería opaca. |
| Obligación de los EE. UU. de no demandar | 1 con reserva | `prohibition(sa, us, sue, premises, jant, …)`. El objeto `premises` es defendible («at the Brookside Bar and Grill»); revisar. |
| Plazos de 14, 30 y 60 días | **4** | `after` tiene granularidad anual en el orden certificado; los plazos en días no son prácticos en unario. La vigencia de dos años, como `after^2`, sí es representable (2). |
| Medidas ya realizadas (Casa Fiesta: adoptó, formó, colocó) | 1 | `Act(..., enact|train|post, ...)`, con grado R |
| Fundamentos (dos preguntas, prohibición de documentación, deber de modificar) | no se formalizan | Pertenecen al canal de fundamentación. Programarlos sería escribir la estructura que el explorador debe descubrir. |
| Determinación de Flowers (viabilidad de la vía Unruh, revocación, costas) | **4** | LegalGeneral no tiene acciones, pretensiones, remedios ni actos procesales. Un predicado `claimCognizable` incorporaría la decisión. |

**Boceto no elaborado del expediente Brookside.** Solo incluye las afirmaciones A y R, más las L marcadas. Las extensiones `requests`, `disability`, `request`, `showCertification`, `leave` y `agree` son **nuevas** y no están congeladas:

```
// A: admitido
Act(visit1, cmp, enterWith, dog1, cmp, private, dV1) @ t
kind(dog1, animal) @ t
Act(ask1, owner1, request, dog1, owner1, private, dA1) @ t
requests(ask1, Relation { debtor = cmp, thing = dog1, conduct = showCertification,
                          creditor = owner1, mode = Doing }) @ t
// L: alegado (segunda ocasión; petición de que se fuera, cuestionada)
Act(visit2, cmp, enterWith, dog1, cmp, private, dV2) @ t
Act(ask2, waitress1, request, dog1, waitress1, private, dA2) @ t
Act(leave1, owner1, request, dog1, owner1, private, dL1) @ t
requests(leave1, Relation { debtor = cmp, thing = dog1, conduct = leave,
                            creditor = owner1, mode = Doing }) @ t
// R/A: acuerdo (estado posterior; ver §6 sobre por qué no es objetivo de predicción)
Act(sa1, jant, agree, premises1, usa, private, d20210216) @ t
obligationFiled(sa1, jant, adoptPolicy, policyA, usa, private, d20210216) @ t
```

Este boceto ya contiene **convenciones discutibles**: el beneficiario reflexivo, la constante `request` y la autoridad del acuerdo. Antes de elaborarlo hacen falta tus decisiones (§8).

---

## 4. Tabla propuesta de restricciones

**Criterio aplicado.** Una posición entra en $P^+$ o $P^-$ solo si un documento **determina** su producción o su exclusión para estos hechos. No basta con que un fundamento general, aplicado por nosotros, la sugiera. Aplicar un fundamento es inferencia del investigador y queda fuera de las valoraciones.

### 4.1 Posiciones candidatas y valoración

| Caso | Posición candidata | Valor propuesto | Fuente y alcance | Respaldo |
|---|---|---|---|---|
| Flowers | `prohibition(·, restaurant, refuseService, dog, john, …)` | **sin valorar** | La opinión no decide la conducta; «potential violation» es un fundamento general. | ninguno |
| Flowers | `obligation(·, restaurant, serve, …, john)` | **sin valorar** | ídem | ninguno |
| Flowers | posiciones de Seth como acompañante | **sin valorar** | no resuelto | ninguno |
| Flowers | «la vía Unruh está disponible» | **no formalizable** (4) | D, pero sin construcción en LegalGeneral | — |
| Casa Fiesta | `prohibition(·, casaFiesta, requireDocumentation, dog, cmp2, …)` | **sin valorar** | hecho solo alegado; sin determinación; acuerdo no disponible | ninguno |
| Casa Fiesta | `prohibition(·, casaFiesta, seatApart, …)` | **sin valorar** | alegado y disyuntivo | ninguno |
| Casa Fiesta | obligaciones del acuerdo | **sin valorar** | texto no disponible; solo se sabe que ciertas medidas se realizaron (R), y son hechos, no posiciones | — |
| Brookside | `prohibition(·, jant/owner, request(showCertification), dog, cmp, …)` | **sin valorar** | Hay un hecho admitido, pero ninguna determinación. La cláusula de alcance excluye leer el acuerdo como interpretación de la ADA. Aplicar el recital de (c)(7) sería inferencia del investigador sobre un fundamento. | ninguno como determinación |
| Brookside | `obligation(·, jant, admit/permitUse, dog, cmp, …)` en la fecha de las visitas | **sin valorar** | ídem; además, la expulsión es controvertida | ninguno |
| Brookside | `permission(·, cmp, enterWith, dog, …)` | **sin valorar**, y además **mal tipada** respecto del documento (§4.2) | — | — |
| Brookside | obligaciones de JANT del acuerdo (adoptar política, formación, cartel, confirmaciones) | **P⁺ documentable** con condiciones (§4.3) | acuerdo firmado, vinculante durante 2 años | D-contractual, alto |
| Brookside | prohibición de los EE. UU. de demandar por el título III | **P⁺ documentable** con condiciones | ídem | D-contractual, alto |
| Todos | cualquier $P^-$ | **ninguno** | Ningún documento excluye una posición. El silencio no es exclusión. | — |

### 4.2 Por qué «obligación de admitir» no se traduce a `permission` ni a `right`

En LegalGeneral:

- **`permission(id, s, c, o, b, …)`** es un permiso de la **propia** conducta de `s`. No tiene efecto constitutivo; solo interviene en la anulación. Un permiso de entrar de la denunciante no expresa que el establecimiento **deba** admitirla, ni que **no pueda** exigir documentos.
- **`right(id, s, c, o, b, …)`** es solo sembrado y no hay regla que lo produzca. Con `Act(s, c, …)` da `compliance`; con `~Act`, `enforceability`. Su sujeto es quien realiza la conducta. Un «derecho de acceso» tendría que escribirse con el establecimiento como sujeto y la denunciante como beneficiaria, y no hay cabeza derivada para ello.
- **`obligation(id, est, admit, dog, cmp, …)`** con `do(est, admit, dog, cmp)` sí expresa un deber de hacer del establecimiento hacia la denunciante. Pero:
  - «admitir» es una conducta del establecimiento distinta de `enterWith` de la denunciante, y el catálogo congelado no contiene ninguna de esas posiciones;
  - el deber legal descrito («modify policies…») es un deber de **cambiar reglas propias**, de segundo orden. Su representación con `enact` y la cadena `legislationText` → `law` exigiría `powerGranted` del propietario, que **no está documentado** y sería un argumento ficticio.

### 4.3 Condición para usar las obligaciones del acuerdo

Estas obligaciones son **efecto constitutivo de un documento**. Hay dos posibilidades:

- **Si el acuerdo está en $S_i$**, cualquier hipótesis las reproduce por `filedObligation`, y la pérdida sobre ellas es 0 para todas las candidatas. **No discriminan.**
- **Si el acuerdo no está en $S_i$**, predecirlas a partir de los hechos del incidente supone tratar una **transacción** como consecuencia jurídica de esos hechos. Eso contradice la cláusula de alcance y confunde transacción con derecho.

**Propuesta:** registrarlas como hechos de un estado posterior de la historia temporal, **no** como objetivos de predicción.

### 4.4 Balance

| | Casos | Posiciones de primer orden valorables sobre la conducta en el local |
|---|---|---|
| Determinaciones sobre los hechos | 0 de 3 | — |
| $P^+$ | — | **0** |
| $P^-$ | — | **0** |
| $P^+$ contractual (no discriminante) | 1 de 3 (Brookside) | — |

---

## 5. Contrato preliminar de pérdida

Se mantiene la forma aprobada:

$$\ell(R,\varepsilon_i)=|P_i^+\setminus N_R(S_i,Q_i)|+|P_i^-\cap N_R(S_i,Q_i)|,\qquad L(R)=\sum_i \ell(R,\varepsilon_i).$$

**Cláusulas:**
1. $N_R(S_i,Q_i)$ es el conjunto de posiciones de $Q_i$ derivadas por Residuum en $T\oplus R$ sobre $S_i$, con el cierre **completo**.
   - Si el presupuesto se agota, el expediente se marca `incomplete` y no aporta pérdida.
   - Un cierre incompleto no certifica que una posición esté ausente: no se usa $P^-\cap N$ como aproximación, ni se anota un acierto.
2. Solo cuentan las posiciones valoradas. Las posiciones de $Q_i$ sin valorar se registran, pero no puntúan.
3. Las posiciones se comparan por identidad canónica: mismos argumentos e índice, con `hypNorm` y `hypEnacted` proyectados como en el contrato v2 §6.
4. Las consecuencias constitutivas (`violation`, `compliance`, `do` o `notDo`) se registran, pero no entran en la pérdida primaria, como en v2.
5. La pérdida normativa se informa separada de cualquier medida de semejanza semántica: la similitud coseno solo puede orientar la navegación y nunca entra en $\ell$.
6. Cada valoración lleva fuente, cita, alcance, grado y quién la propone. Las valoraciones de origen humano (§8, opción B) se marcan `source = valuer`, no `source = document`.

**Resultado con la tabla de §4.** Para todo $R$:
- $P_i^+=P_i^-=\varnothing$ en Flowers y Casa Fiesta;
- en Brookside, $P^+$ solo contiene obligaciones contractuales.

Si el acuerdo está en $S$, $\ell=0$ para todo $R$. Si no lo está, el objetivo no es legítimo (§4.3).

Por tanto **$L(R)$ es constante**: el corpus no produce ninguna refutación ni señal de aprendizaje.

---

## 6. Separación de canales y filtración

| Canal | Contenido con este corpus |
|---|---|
| **Fáctico** | Las afirmaciones A, R y L representables (§3), cada una con su grado como *provenance*. Las L no deberían mezclarse sin marca con las A. |
| **Valoración** | Vacío en el primer orden (§4). |
| **Fundamentación (reservado)** | Los F de los tres documentos: deber de modificar políticas; dos preguntas; prohibición de exigir documentación; acceso a todas las zonas públicas; «potential violation»; armonización Unruh/DPA. No se entregan al explorador. |

**Riesgos de filtración y cómo medirlos o reducirlos:**

1. **Identificadores reconocibles.** Afectan a «Flowers v. Prasad», «Valley India Café», «Canoga Park», «Brookside», «Higganum», «Casa Fiesta», «Richmond», los números DJ y frases literales como «Service Animals Welcome».
   - *Reducción:* sustituirlos por constantes neutras y fechas simbólicas. Renombrar no falsifica; cambiar relaciones sí, y no se hace.
2. **Conocimiento previo del derecho.** Es el riesgo **mayor**, y anonimizar no lo elimina. La regla de la ADA (dos preguntas, sin documentación, solo perros) es pública y está muy repetida, así que un LLM la conoce casi con seguridad.
   - En consecuencia, la premisa de que el explorador desconoce las justificaciones **no se cumple** con vocabulario natural.
   - *Medición:*
     - comparar la condición natural con una condición de **vocabulario opaco**, ya prevista en el contrato del dominio semántico y excluida del Piloto 004;
     - prerregistrar una **sonda de memorización**: presentar el expediente anonimizado y pedir que identifique la fuente o prediga el desenlace, puntuando el acierto contra una línea base con perturbaciones isomorfas.
   - La sonda necesitaría una llamada pagada, que **no se ejecuta sin autorización**.
3. **Fundamentos dentro de los hechos.** Expresiones como «service animal», «disability» o «demanded documentation» llevan dentro la calificación jurídica.
   - *Reducción:* separar la base descriptiva cuando esté documentada. Aquí no lo está, de modo que la limitación persiste.

---

## 7. Contrastes y contrafactuales

### 7.1 Comparaciones entre experiencias reales

Diferencias documentadas: tipo de conducta, acompañante, creencia del propietario, número de ocasiones, vía y derecho aplicable (tabla de §2.4).

**No se puede atribuir causalidad a ninguna**, porque los tres casos no reciben tratamientos que difieran en un sentido comparable. No hay variación de la variable de resultado.

### 7.2 Variaciones hipotéticas admisibles (sin valorar)

Cambian un rasgo fáctico y conservan el resto:
- perro no adiestrado o en adiestramiento;
- condición de animal de servicio evidente frente a no evidente;
- solo se formulan las dos preguntas;
- el animal está fuera de control;
- otra especie;
- la clienta elige la terraza frente a que el local la envíe allí;
- el acompañante sin discapacidad acude solo con el perro.

### 7.3 Contrafactuales con valoración externa

**Ninguno en este corpus.**

### 7.4 Contrafactuales todavía no valorados

Todos los de 7.2. Solo podrían valorarse de dos formas:
- con un valorador humano independiente, cuya valoración no sería documental;
- con otras decisiones que los hayan resuelto, lo que exige ampliar el corpus (§8).

### 7.5 Hipótesis rivales compatibles con el corpus

Al no haber valoraciones, **todas** son compatibles. Por ejemplo:
- **H1:** el deber del establecimiento depende de que el animal esté efectivamente adiestrado;
- **H2:** depende de cómo se presente, con independencia del adiestramiento;
- **H3:** pedir documentación está prohibido siempre;
- **H4:** solo está prohibido exigirla, no preguntar;
- **H5:** sentar aparte está prohibido;
- **H6:** sentar aparte está permitido cuando hay una razón sanitaria acotada.

**Experiencias que las distinguirían:**
- decisiones sobre perros no adiestrados (H1/H2);
- sobre preguntas sin exigencia (H3/H4);
- sobre la reubicación con una razón documentada (H5/H6).

En la investigación anterior (`docs/pilot-004-recalcitrant-cases.md`, con fuentes verificadas) ya hay candidatos de este tipo, pero **no se añaden** sin tu autorización.

---

## 8. Comparación con el dominio congelado del Piloto 004 (contrato v2)

| Aspecto | Piloto 004 v2 | Tres casos reales |
|---|---|---|
| Hechos expresables ya | `kind(obj, animal)`, `Act(..., enterWith, obj, ...)`, `copresent` | el perro como animal; la entrada con el perro (con la reserva del beneficiario) |
| Extensiones que harían falta | — | `requests(act, Relation)`; constantes `request`, `showCertification`, `leave`, `agree`, `adoptPolicy`; `condition(·, disability)` como calificación alegada; `parentOf`; un lugar como `Thing` con titular |
| Sujeto de las posiciones | **ana**, quien entra (`K(hypNorm, ana, enterWith, obj, bea, …)`) | **el establecimiento** y su personal (exigir, admitir, sentar aparte, pedir que se vaya) |
| Posiciones reutilizables | ninguna: el catálogo de 534 no contiene posiciones del establecimiento | — |
| Lugar y titular | no representados (límite declarado) | imprescindibles |
| Valoraciones | por decidir, 12 parciales | **0 de primer orden** documentadas |
| Transformaciones normativas admisibles | prohibición, permiso y obligación con la gramática de v2 | las mismas cabezas, con otro sujeto y otras conductas: **otra gramática** |

**Componentes reutilizables:**
- el aislamiento $T\oplus h$ (clonar el módulo e insertar la regla);
- la identidad canónica;
- el generador determinista de $d_0$;
- el formato de valoraciones (`TEMPLATE.tsv`);
- la proyección de argumentos constantes;
- el ledger, los presupuestos y el canal del explorador;
- la historia temporal (varias ocasiones, estado posterior del acuerdo);
- `constitutes` y `certifies`, como modelo de extensión sin efectos.

**Limitaciones que impiden evaluar fielmente las decisiones:**
- no hay remedios ni pretensiones (Flowers);
- no hay causalidad ni motivo («due to»);
- no hay actitudes epistémicas (la creencia del propietario);
- no hay disyunción en los hechos;
- no hay plazos en días;
- no hay modalidad ilocutiva (exigir frente a preguntar);
- la entidad no puede ser a la vez persona y cosa.

**Variante documental («004-D»).** Necesitaría otra gramática, centrada en el establecimiento, y las extensiones anteriores. **No se propone implementarla todavía**: con estas fuentes no habría valoraciones que la evaluaran. El contrato v2 queda intacto.

---

## 9. Respuestas a los criterios de calidad

1. **¿Qué decisiones normativas están documentadas?**
   - Flowers: la vía Unruh es viable junto a la DPA, más la apelabilidad, la revocación y las costas.
   - Brookside: obligaciones **contractuales** futuras de JANT y de los EE. UU.
   - Casa Fiesta: **ninguna**; solo un comunicado que relata alegaciones y medidas realizadas.
2. **¿Cuáles pueden convertirse fielmente en valoraciones parciales?**
   - Ninguna de primer orden sobre la conducta en el local.
   - Solo las obligaciones del acuerdo de Brookside, que no discriminan entre hipótesis (§4.3).
3. **¿Qué hechos y relaciones necesita representar el sistema?**
   - Personas, perro, local con titular, entradas, peticiones con su contenido y el acuerdo (§3).
   - Las calificaciones «discapacidad» y «animal de servicio» no tienen base descriptiva documentada.
4. **¿Qué justificaciones permanecen desconocidas para el explorador?**
   - En principio, los F de §6.
   - En la práctica, con vocabulario natural, el LLM probablemente ya conoce la regla de la ADA. Esa premisa no está asegurada y debe medirse.
5. **¿Qué hipótesis rivales son compatibles?** Todas, incluidas H1 a H6 de §7.5.
6. **¿Qué experiencias las distinguirían?** Decisiones de fondo con hechos probados y resultados distintos según el adiestramiento, el tipo de pregunta, la reubicación o el control del animal.
7. **¿Es suficiente el corpus?** **No.**
   - No contiene ninguna determinación sobre hechos, ningún $P^-$ ni tratamientos que varíen.
   - $L(R)$ es constante, así que no puede generar refutaciones ni medir pérdida prospectiva.
8. **¿Qué limitaciones lo impiden?**
   - De independencia: transacciones que declaran no interpretar la ley; un comunicado sin el acuerdo; una decisión sobre una vía procesal.
   - De expresividad: motivo, creencia, disyunción, modalidad, remedios y la entidad que es a la vez persona y cosa.
   - De filtración: el conocimiento público de la ADA.

## 10. Decisiones pendientes

1. **Insuficiencia del corpus.** Queda demostrada (§4.4 y §5). Pido autorización para alguna de estas opciones:
   - **A (recomendada).** Ampliar el corpus con decisiones de **fondo** que tengan hechos establecidos y resultados que varíen. Hay candidatos ya verificados en fuente primaria en `pilot-004-recalcitrant-cases.md`:
     - *Johnson v. Gambrinus*;
     - *Lentini*;
     - *Rose*, en la que se rechaza el acceso;
     - *Miller v. Fortune*, perro en adiestramiento, en la que se rechaza el acceso;
     - dictámenes del CRM neerlandés con resultados de los dos signos (2018-47, 2020-79, 2020-105, 2024-104).

     Cada uno pasaría por la misma reconstrucción.
   - **B.** Mantener los tres casos como canal fáctico y obtener valoraciones de un valorador humano independiente, registradas como `source = valuer`.
   - **C (no recomendada).** Reorientar el objetivo a predecir obligaciones transaccionales.
2. **Convenciones de §3:**
   - el beneficiario reflexivo en `enterWith` y en las peticiones;
   - la extensión `requests(act, Relation)`;
   - la autoridad del acuerdo (`private` o `administrative`);
   - `condition(·, disability)` como calificación alegada.
3. **Grado L en el canal fáctico:** si las alegaciones entran en el expediente con marca, o quedan fuera.
4. **Medición de la filtración:** si se incluye la condición de vocabulario opaco y una sonda de memorización. Esta última implica llamadas pagadas que necesitan autorización y presupuesto propios.
5. **Acuerdo de Casa Fiesta:** si se intenta obtener su texto por otra vía, por ejemplo una solicitud FOIA o una petición a la USAO.
