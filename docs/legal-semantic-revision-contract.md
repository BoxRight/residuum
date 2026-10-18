# Revisión semántica iterativa de nodos: reformulación crítica de la Etapa 0

Estado: **CONTRATO PROPUESTO**. No se ha implementado nada, no se ha calculado
ningún embedding y no se ha hecho ninguna llamada a modelos. Este documento
sustituye las §2 y §5–6 de `legal-semantic-retrieval-stage0.md`. Las demás
secciones de ese informe siguen valiendo como inventario.

Marcas usadas:
- **DEMOSTRADO**: con las hipótesis indicadas.
- **PREDICCIÓN**: falsable; no es un teorema.
- **EXISTE / TEST / NUEVO**: estado en el código.
- **ABIERTO**: decisión que no corresponde al harness.

## 0. Conclusiones

1. **Qué es un nodo.** Un nodo es un elemento de un **catálogo finito de
   hipótesis** $H$. No es un punto de la retícula de situaciones. Sobre $H$ se
   calculan tres estructuras distintas: el preorden de cobertura, la partición
   inducida por un protocolo y el espacio de versiones. Las situaciones viven
   en otra estructura, la retícula booleana $K=\mathcal P(\Sigma)$ con el
   operador de cierre $C_T$, y los contrafactuales se mueven en ella. No se
   identifican entre sí (§2).
2. **Qué significa revisar.** Revisar es sustituir la componente interpretativa
   $h_k(v)$ de la descripción, con Γ y la componente formal fijas. Cada
   afirmación lleva su procedencia, y las que dicen proceder de una fuente
   verificable se comprueban (§3–4).
3. **Hay que fijar la política de navegación.** Para aislar el valor de revisar
   la representación, la política tiene que ser una función **determinista**
   de la semejanza $s_k$ y del estado observado. Si el LLM eligiera además qué
   evaluar, su elección quedaría confundida con el efecto de la representación
   (§5).
4. **Límite demostrado.** En Residuum evaluar es barato. Si $|H|\cdot|O|$ cabe
   en el presupuesto, el espacio de versiones no depende de $s_k$, y la
   revisión no puede reducir las evaluaciones verificadas necesarias
   (Proposición 1).
   - En el subperfil certificado (10.850 cláusulas) ese régimen se cumple.
   - Por tanto, la métrica de evaluaciones verificadas no sirve en ese
     subperfil, salvo con un tope declarado y artificial.
   - La revisión solo puede influir en dos cosas: el **sesgo** con que se
     elige entre las hipótesis compatibles, que se ve en la pérdida
     prospectiva, y la **selección de contrastes**, cuyo coste se mide en
     valoraciones externas.
5. **Reducción informacional.** Bajo el contrato de información, cualquier
   ventaja de C sobre una política simbólica que use la misma información de
   refutación solo puede venir del conocimiento previo léxico del LLM y de
   $E$, o del azar (Proposición 2). Hacen falta dos controles más que no
   figuraban en tu propuesta:
   - **A_fb**: retroalimentación simbólica sin embeddings;
   - **vocabulario opaco**: nombres sin contenido léxico.
6. **Bloqueo.** Con las valoraciones independientes que hay hoy (una sola
   documentada, en un dominio con cabeza artificial), ninguna de las
   hipótesis falsables de §7 se puede medir. Es la decisión ABIERTA principal.

## 0 bis. Alcance acordado (revisión del usuario, 2026-10-18)

Este contrato es el de una **ablación controlada**:
*¿sirve por sí misma la revisión de la representación semántica?*
No es la arquitectura del sistema completo.

1. **Política fija.** Solo se exige en la ablación. En el sistema completo el
   LLM podrá interpretar refutaciones, proponer distinciones y nuevas Γ
   (`Propose`) y elegir contrafactuales. Ese será un experimento distinto, que
   se hará después.
2. **Controles.** Se mantienen A_fb, el vocabulario opaco y la reescritura sin
   feedback (D). La Proposición 2 no invalida la hipótesis: localiza el origen
   posible de una ventaja en el conocimiento conceptual previo, o en un sesgo
   que la familia simbólica no expresa, y el control opaco lo mide.
3. **Dominio.** El catálogo se deriva del dominio ontológico y de las
   transformaciones normativas de interés, no del subperfil `compliance` por
   el mero hecho de estar enumerado. Después se delimita un subespacio finito
   para la ablación. Hay que fijarlo **antes** de generar valoraciones.
4. **Valoraciones.**
   - La negociación simulada entre partes es una opción para el experimento
     principal. Sus resultados se registran como valoraciones externas al
     aprendiz, con fuente `simulada`.
   - El objetivo sintético oculto queda como control metodológico.
5. **Métricas.** Se separan tres capacidades:

   | Capacidad | Métrica |
   |---|---|
   | Revisión semántica | cambio útil de vecindades y prioridades |
   | Aprendizaje de generalizaciones | pérdida prospectiva |
   | Investigación contrafactual | valoraciones independientes necesarias para descartar hipótesis |

   La métrica primaria de la ablación es la pérdida prospectiva, acompañada
   del número de valoraciones. Las evaluaciones de Residuum son secundarias.
6. **Lo que sigue sin autorizar:** llamadas pagadas y cambios en el motor.

## 1. Crítica del contrato anterior

| Contrato anterior | Problema | Reformulación |
|---|---|---|
| $d$ y $z$ son anotaciones que no intervienen en la navegación | La hipótesis central exige que $d$ intervenga en la navegación | $s_k$ es una entrada de la política $\pi$ (§5) |
| Γ admite cualquier conjunto de cláusulas propuesto | Sin catálogo finito no hay referencia para medir | En la fase de medición, $H$ es finito y está fijado antes del episodio; proponer Γ nuevas es una operación aparte |
| La descripción es una única prosa más afirmaciones | No separa lo que es fiel a Γ de lo revisable | $d_k=(f,h_k)$ con procedencias (§3) |
| Las métricas son de recuperación | Que un vector cambie no es aprendizaje | Métricas sobre la selección y la pérdida prospectiva (§7) |
| Control: A frente a B | No aísla el valor de la revisión | Comparación principal C frente a B, con D, A_fb y controles negativos (§8) |

## 2. Estructuras simbólicas disponibles

| Estructura | Portador | Definición | Estado en el código |
|---|---|---|---|
| **K**: retícula de situaciones | $\mathcal P(\Sigma)$, con $\Sigma$ = eventos Seeded ground | inclusión; $C_T$ es un operador de cierre y sus puntos fijos forman una familia de Moore | TEST `universe`, `subset`, `extreme_points`; EXISTE `exploration::universe`, `frontiers` (`8edcc30`) |
| **⪯_g**: preorden de cobertura | catálogo $H$ | $\Gamma_s\preceq_g\Gamma_g\iff Cov_{A,Q}(\Gamma_s)\subseteq Cov_{A,Q}(\Gamma_g)$ | EXISTE `exploration::covers` (comparación por fronteras, `:520`) |
| **Π**: partición por protocolo | $H$ | $\theta\equiv_D\eta\iff o_Q(\operatorname{Run}_{T_\theta}(S))=o_Q(\operatorname{Run}_{T_\eta}(S))$ para todo $S\in D$ | EXISTE `kernel`, `meet`, `finer`, `Partition`, `discriminants` (`8edcc30`); TEST en esta rama |
| **V_n**: espacio de versiones | $H$ | $\{h:\ \kappa_\varepsilon(N_h(S))=1\ \forall\varepsilon\in O_n\}$ | TEST `submit`, `filter` (`legal_geometry_conversation_tests.rs:276,322`) |
| **Minimax**: selección de preguntas | situaciones candidatas | peor caso del tamaño de $V$ tras la respuesta | TEST `scores`, `best` (`:438,467`) |

Propiedades que **no** se pueden suponer:
- **⪯_g** no es antisimétrico. Su cociente por $\equiv_{cov}$ es un poset,
  pero no es una retícula en general: la intersección de dos coberturas no
  tiene por qué realizarse en $H$.
- **V_n** no es convexo en general (contraejemplo de paridad,
  `legal-relational-search-space-contract.md` §9).
- **Π** organiza las predicciones, no la generalidad: «admitir todo» y
  «excluir todo» caen en el mismo bloque si el protocolo solo tiene un caso.

**Elección:**
- Un **nodo** es un $v\in H$.
- Las **situaciones** no son nodos. Son puntos de $K$, y un contrafactual
  $S\xrightarrow{\delta}S'$ es un desplazamiento en $K$.
- El vínculo entre las dos estructuras es la evaluación
  $\operatorname{ev}:H\times K\to\mathcal P(E_Q)$,
  $\operatorname{ev}(h,S)=N_h(S,Q)$, que calcula el motor.
- Las cinco estructuras se registran por separado y ninguna se deduce de $s_k$.

Describir también situaciones (por ejemplo, para ordenar contrafactuales) es
posible, pero queda fuera de la primera fase: duplicaría las variables
experimentales.

## 3. Nodo y representaciones

Para $v\in H$:

$$
v\longmapsto\big(\Gamma_v,\ d_k(v),\ z_k(v)\big),\qquad
d_k(v)=\big(f(v),\ h_k(v)\big).
$$

- **Γ_v** es la configuración Formal elaborada. Su identidad es
  $\operatorname{id}(\Gamma_v)$ con $\mathrm{canon}$, como en
  `legal-semantic-retrieval-stage0.md` §2.2 (NUEVO). Es inmutable durante el
  episodio.
- **f(v)**, la componente formal, se escribe **una sola vez y no se revisa**.
  Dos opciones:
  - (a) $f=\rho(\Gamma_v)$, un renderizado determinista de $\mathrm{canon}$
    con roles, tipos, polaridad, patrón temporal y variables compartidas;
  - (b) una paráfrasis del LLM que, si supera el validador formal, queda
    congelada.

  Recomiendo (a) como base, y (b) solo como variante medida.
- **h_k(v)**, la componente interpretativa, es un conjunto finito de
  afirmaciones $c=(\text{texto},\ \text{procedencia},\ \text{referencia})$.
  Las procedencias posibles son:

  | Procedencia | Referencia obligatoria | Comprobación |
  |---|---|---|
  | `formal` | elemento de Γ (`body[i]`, rol, variable) | verdadera en Γ |
  | `consecuencia` | id de un registro de evaluación de 𝒫 | el evento o la ausencia constan en ese registro, que debe estar completo si afirma ausencia |
  | `valoración` | id de una ε **observada** | coincide con $P^\pm$ de ε |
  | `hipótesis` | ninguna | no se comprueba; se marca como no corroborada |

- **z_k(v)** $=E(\sigma(d_k(v)))$. Aquí $\sigma$ es una serialización fija y
  $E$ un embedder con pesos y versión fijos. Se registran además
  $z^f=E(\sigma(f))$ y $z^h_k=E(\sigma(h_k))$, para medir qué componente mueve
  la semejanza.
- **Semejanza:** $s_k(u,v)=\cos(z_k(u),z_k(v))$.
- **Vecindad:** $\mathcal N_k(v,m)$ son los $m$ nodos con mayor $s_k$; los
  empates se deshacen por id.

Sobre la semejanza (**DEMOSTRADO**, trivial):
- $s_k$ es simétrica, así que no induce un orden parcial.
- El ángulo es una métrica en la esfera, pero solo una pseudométrica sobre los
  nodos: descripciones iguales dan distancia cero.
- Nada garantiza que $u\preceq_g w$ implique que $s_k(u,w)$ sea alta, ni que
  dos nodos del mismo bloque de Π estén cerca.

## 4. Operación de revisión

$$
h_{k+1}(v)=\operatorname{Revise}\big(f(v),\ h_k(v),\ \Gamma_v,\ \iota(\varepsilon_k)\big),
$$

donde $\iota(\varepsilon_k)$ es la **vista** de la discrepancia que permite el
contrato de información (§4.1).

Una revisión es **admisible** si cumple todas estas condiciones:

| | Condición |
|---|---|
| R1 | $\operatorname{id}(\Gamma_v)$ no cambia y $f(v)$ no cambia. |
| R2 | Toda afirmación `formal`, `consecuencia` o `valoración` supera su comprobación. |
| R3 | **Sin constantes ground:** $h_{k+1}$ no menciona identificadores de expediente ni constantes de ningún expediente, observado o reservado. Solo usa verbos, tipos, roles y relaciones del vocabulario de $\tau$. Así la descripción no puede almacenar respuestas por caso. |
| R4 | No hay autovaloración: ninguna afirmación dice que una hipótesis sea correcta, compatible o adecuada si no remite a un registro de 𝒫 o 𝒱. |
| R5 | Presupuesto de longitud $L$, el mismo en C y en D. |
| R6 | No menciona verbos ajenos a $\tau$. Las analogías con conceptos externos quedan permitidas solo como `hipótesis`. |

Las revisiones rechazadas se registran como tales y cuentan en las métricas de
protocolo.

**Qué no es una revisión:**
- Si el LLM propone cambiar Γ, eso es `Propose(Γ')`: otra operación, que se
  elabora y se verifica por la vía de admisión, y que da otro nodo.
- En la fase de medición, `Propose` está desactivada, de modo que el catálogo
  $H$ queda fijo.

**Qué nodo se revisa.** En la iteración $k$ solo se revisa el nodo refutado
$v_k$. La revisión cambia $s(v_k,\cdot)$ y no toca las semejanzas entre otros
pares. Esto mantiene el efecto localizado y medible. Propagar revisiones a
vecinos sería otra variable experimental.

### 4.1 Contrato de información

Niveles acumulativos, que se pueden usar como variable de ablación:

| Nivel | Contenido de $\iota(\varepsilon_k)$ | Origen |
|---|---|---|
| I0 | «refutado en un expediente observado», sin más | harness |
| I1 | más: el expediente observado (sus semillas) y la posición exigida o excluida | 𝒱 (observadas) |
| I2 | más: el resultado $N_h(S,Q)$ y, para cada exceso, productor, antecedentes y sustitución | 𝒫 (`Closure.derivations`) |
| I3 | más: los requisitos residuales de cada posición que falta | `query_active_program_residual` |

**Prohibido en todos los niveles:**
- valoraciones reservadas, y cualquier agregado calculado con ellas;
- la tabla de puntuación offline (§5.2);
- el estado de compatibilidad de otros nodos más allá de lo observado;
- la regla objetivo, cuando el control es sintético;
- descripciones de otros nodos de la condición C, porque cruzarían
  información entre nodos sin registrarlo.

**Riesgo propio de I3:** un requisito residual puede dejar la justificación a
un solo paso. No se excluye, porque es una salida legítima del motor, pero se
mide por separado, comparando I2 con I3.

## 5. Política de navegación

### 5.1 Política fija

Un episodio tiene:
- catálogo $H$;
- expedientes observados $O$ y reservados $R$, disjuntos;
- valoraciones $\mathcal V_O$ y $\mathcal V_R$;
- nodo inicial $v_0$;
- presupuesto de evaluaciones $B$ y presupuesto de revisiones $B_r$.

La política $\pi_s$ es la misma en las condiciones B, C y D. Solo cambia la
semejanza que recibe:

```
actual ← v0
mientras queden evaluaciones:
    evaluar actual sobre O con el motor      # 𝒫, ℓ por posición
    si ℓ_O(actual) = 0 con ejecución completa: añadir a Compatibles
    si no: ε ← discrepancia; (C) revisar h(actual) | (D) reescribir sin ε | (B) nada
    actual ← argmax_{u no evaluado, admisible} s_k(actual, u)
salida: elegir(Compatibles) con regla fija
```

La regla `elegir` también es la misma en todas las condiciones. La comparación
principal usa la máxima semejanza media con los nodos refutados revisados. La
condición A usa en su lugar las preferencias simbólicas: la más específica, la
más general y la de menos átomos.

Aquí el LLM no elige qué evaluar. Solo escribe descripciones. Un agente que
elija acciones es una fase posterior, que añade varianza y confunde efectos.

### 5.2 Tabla de puntuación offline

Como $H$ es finito y evaluar es barato, se precalcula una vez
$\operatorname{ev}(h,S)$ para todo $h\in H$ y todo $S\in O\cup R\cup\Delta$,
donde $\Delta$ son los contrafactuales admisibles:
- La tabla sirve para **puntuar** cualquier trayectoria de forma reproducible,
  sin volver a llamar al LLM ni al motor.
- Ninguna política la ve.
- Las evaluaciones que hace la política se recalculan igualmente con el
  motor; coincidir con la tabla es un test de reproducibilidad.

### 5.3 Contrafactuales

Para el conjunto vigente $M_k=\mathcal N_k(v_k,m)$, el explorador elige una
transformación $\delta$ entre los desplazamientos admisibles de una situación
observada:
- añadir o quitar un evento Seeded tipado de $\Sigma$;
- o una sustitución relacional declarada.

Cada desplazamiento se comprueba elaborando el experimento resultante.

**La parte simbólica, fija:** entre los $\delta$ admisibles, el que maximiza
el número de bloques en que `discriminants` divide $M_k$ según
$\operatorname{ev}(\cdot,S')$.

**La parte semántica:** cuál es $M_k$. La revisión solo puede cambiar los
contrastes que se proponen a través de la vecindad.

**Cómo se mide:** cuánto discrimina $\delta$ dentro del espacio de versiones
**verdadero** sobre $R$, calculado offline. No basta con que discrimine dentro
de $M_k$.

**Semántica de Lewis.** Si se adopta, hay que declarar por separado:
- la **admisibilidad** (pertenencia a $K$ y elaboración correcta);
- la **semejanza entre situaciones**, por ejemplo la distancia de Hamming en
  $\Sigma$ o el número de pasos $\delta$.

$s_k$ es semejanza entre *descripciones de hipótesis*, no entre mundos, y no
se usa como ordenación de Lewis. La diferencia de tratamiento entre $S$ y
$S'$ que muestran las valoraciones es **dependencia respaldada**. Que haya
causalidad no se afirma.

## 6. Límites demostrados y contraejemplos

**Proposición 1 (irrelevancia en el régimen exhaustivo). DEMOSTRADO.**

*Enunciado.* Sea $H$ finito, $O$ finito, y supongamos $B\ge|H|$ con
ejecuciones completas. Entonces, para toda semejanza $s$, el conjunto
`Compatibles` final es $V_O$. Por tanto, el número de evaluaciones necesarias
para *determinar* $V_O$ es $|H|$, igual para A, B, C y D.

*Prueba.* $\pi_s$ evalúa nodos distintos hasta agotar $H$, y la
compatibilidad de cada uno depende solo de $\operatorname{ev}$ y de
$\mathcal V_O$, no de $s$. ∎

*Consecuencias.*
- $s$ solo influye en el **orden**, es decir, en cuántas evaluaciones hacen
  falta para encontrar la *primera* hipótesis compatible, y en `elegir`.
- En el subperfil de 10.850 cláusulas y unos pocos expedientes, $|H|\cdot|O|$
  cierres cabe holgadamente en el presupuesto de cómputo. Una política
  simbólica puede calcular $V_O$ directamente, y «evaluaciones necesarias»
  solo es una métrica si se impone un tope $B\ll|H|$. Ese tope modela un
  régimen que en este subperfil no existe, y tiene que declararse como tal.
- Los recursos realmente caros son las **valoraciones externas** y las
  **llamadas al LLM**.

**Contraejemplo mínimo.**
- Datos: $H=\{h_1,h_2,h_3,h_4\}$, $O=\{S\}$, $B=4$, con $h_2$ y $h_4$
  compatibles.
- Toda ordenación termina con `Compatibles` $=\{h_2,h_4\}$.
- Si `elegir` es simbólica, la salida es la misma cualquiera que sea $s$. Si
  `elegir` usa $s$, la salida puede cambiar, pero solo dentro de $V_O$.
- Mejorar la pérdida prospectiva exige entonces que $h_2$ y $h_4$ difieran
  sobre $R$ y que $s$ prefiera la correcta.

**Proposición 2 (reducción informacional). DEMOSTRADO bajo el contrato §4.1.**

*Enunciado.* La secuencia $h_1,\dots,h_k$ es función de:
- $(f,\Gamma)$;
- las vistas $\iota(\varepsilon_1),\dots,\iota(\varepsilon_k)$, que son
  funciones de $\mathcal V_O$ y de las salidas del motor;
- los parámetros del LLM;
- su aleatoriedad.

Por tanto $s_k$ es función de los datos simbólicos observados, de los priors
del LLM y de $E$, y del azar. En consecuencia, cualquier política simbólica
que reciba las mismas vistas, como A_fb, dispone de toda la información
*observacional* que tiene C. Una ventaja de C sobre la mejor política de ese
tipo solo puede atribuirse:
- al conocimiento previo léxico o conceptual del LLM y de $E$;
- a un sesgo inductivo útil que la familia simbólica considerada no expresa;
- al azar.

*Prueba.* Es la composición de funciones con las entradas que el contrato
autoriza. Las valoraciones reservadas no figuran entre ellas. ∎

*Corolario (vocabulario opaco).* Si los nombres de verbos y constantes se
sustituyen por identificadores sin contenido léxico, el componente de
conocimiento previo desaparece salvo por la estructura. **PREDICCIÓN:** con
vocabulario opaco, $C-B$ debe reducirse hacia $A_{fb}-A$. Si con vocabulario
natural $C$ supera a $A_{fb}$ y con vocabulario opaco no, la ventaja viene del
conocimiento léxico previo, no de la incorporación de refutaciones.

**Proposición 3 (peor caso). DEMOSTRADO para selección adaptativa de
preguntas con respuestas deterministas.** El árbol de decisión minimax exacto
sobre los objetivos de $H$ minimiza, por definición, el número de valoraciones
en el peor caso. Ninguna política, sea semántica o no, lo mejora en ese
criterio. Una ventaja de C solo puede observarse **en promedio**, bajo una
distribución de objetivos que hay que declarar: legal o sintética uniforme.

La selección `best` del repo es voraz (un paso) y no garantiza ese óptimo. La
comparación honesta es contra esa misma regla voraz y, cuando $H$ sea pequeño,
contra el árbol exacto.

**Control negativo (PREDICCIÓN).** Si las valoraciones las genera un objetivo
$h^*$ elegido uniformemente en $H$ y con vocabulario opaco, el conocimiento
previo no está correlacionado con el objetivo. Entonces C y D deben quedar
empatadas en expectativa, y C no puede superar a A_fb. Una diferencia
significativa indicaría filtración o un artefacto del protocolo.

## 7. Hipótesis falsables y medidas

Todas se registran previamente, con splits $O/R$, presupuestos y semillas fijos
antes de cualquier ejecución.

| | Hipótesis | Medida | Falsación |
|---|---|---|---|
| H1 | La revisión con feedback reordena la vecindad del nodo refutado hacia la cobertura relevante | Variación de la correlación de Spearman entre $s_k(v,\cdot)$ y el Jaccard de cobertura sobre $O$, de $k$ a $k+1$; superposición top-$m$ (RBO) de $\mathcal N_k$ frente a $\mathcal N_{k+1}$ | La variación en C no supera a la de D |
| H2 | La nueva prioridad selecciona generalizaciones que anticipan mejor | Pérdida prospectiva $\ell_R$ de la salida de `elegir`, en media sobre splits y objetivos | $\ell_R(C)\ge\ell_R(B)$, o $\ell_R(C)\ge\ell_R(A_{fb})$ |
| H3 | La revisión mejora los contrastes propuestos | Proporción de $\delta$ que discriminan dentro del $V$ verdadero sobre $R$; valoraciones hasta $\lvert V\rvert\le m$ | No hay mejora de C frente a B y D |
| H4 | El efecto no es reescritura persuasiva | Diferencia C − D con longitud igualada | C − D ≈ 0 |
| H5 | El efecto no es solo conocimiento léxico previo | C − B con vocabulario natural frente a opaco | La ventaja desaparece con vocabulario opaco y $C\le A_{fb}$ |

**Medidas descriptivas** (no son hipótesis):
- evaluaciones hasta la primera compatible sobre $O$, solo con tope declarado;
- frecuencia de revisiones admisibles que no mejoran el rango de ninguna
  hipótesis de menor $\ell_R$;
- tasa de revisiones rechazadas por R1–R6;
- proporción de la variación de $s$ atribuible a $z^h$ frente a $z^f$;
- coste en llamadas y en tokens.

## 8. Condiciones y ablaciones

**Condiciones principales:**

| Condición | Semejanza usada por $\pi$ | Revisión |
|---|---|---|
| A | ninguna: preferencias simbólicas (generalidad, tamaño) | — |
| **A_fb** | puntuación simbólica de los rasgos implicados por la discrepancia (verbos de productores en exceso, verbos de requisitos residuales), con la misma vista $\iota$ | — |
| B | $s_0$ con $d_0=(f,\varnothing)$ o con $h_0$ fija | ninguna |
| **C** | $s_k$ | `Revise` con $\iota(\varepsilon_k)$ |
| D | $s_k$ | reescritura con el mismo número de pasos y la misma longitud, sin $\iota$ |

A_fb es el control que separa «la refutación es informativa» de «la
representación semántica de la refutación es útil». Sin A_fb, que C supere a
B no distingue las dos explicaciones.

**Ablaciones secundarias,** por orden de prioridad:
1. vocabulario natural frente a opaco;
2. valoraciones con fundamento jurídico frente a objetivo sintético uniforme
   (control negativo);
3. niveles de información I1, I2 e I3;
4. embedding de $f\oplus h$ frente a solo $h$;
5. $f$ determinista frente a $f$ parafraseada por el LLM.

El diseño mínimo viable es {A, A_fb, B, C, D} × {natural, opaco}, más el
control sintético sobre C y D.

## 9. Riesgos y controles

| Riesgo | Mecanismo | Control |
|---|---|---|
| Autorrefuerzo | el mismo modelo describe, revisa y su texto define la vecindad | D; $f$ fija y determinista; A_fb; varias semillas del LLM |
| Codificar la respuesta | $h$ almacena «en el expediente X sale P» | R3 sin constantes ground; comprobación léxica contra los identificadores de $O\cup R$ |
| Filtración de reservadas | contexto o agregados con $\mathcal V_R$ | §4.1; registro de cada vista enviada; auditoría del ledger |
| Autovaloración | el LLM afirma que su hipótesis es correcta | R4; solo 𝒱 externo cuenta |
| Sobreajuste a $O$ | revisiones que explican solo lo observado | $\ell_R$ como medida principal; varios splits; pérdida cero en $O$ nunca es criterio |
| Conocimiento previo del caso | caso célebre (Riggs) o vocabulario conocido | vocabulario opaco; expedientes con nombres neutros |
| Longitud y estilo | textos más largos desplazan $z$ sin informar | $L$ igual en C y D; medir $z^h$ frente a $z^f$ |
| Anisotropía del embedder | diferencias de coseno pequeñas e inestables | medir rangos (Spearman, RBO), no cosenos brutos |
| Varianza | pocos episodios | registro previo; intervalos sobre splits y semillas; resultado negativo admitido |
| Valoraciones insuficientes | sin $\mathcal V_R$ no hay pérdida prospectiva | bloqueo declarado (§12) |

## 10. Relación con la historia temporal

| Historia | Contenido | Dónde vive | ¿Afecta al cierre histórico? |
|---|---|---|---|
| Formalizaciones jurídicas | $H_n$: módulos de etapa, programas y anclajes (`dacf1d4`) | Formal | es el cierre histórico |
| Descripciones y embeddings | $(\operatorname{id},k,f,h_k,\text{autor},E,\text{versión},z_k)$ | ledger | no |
| Hipótesis y evaluaciones | ejecuciones aisladas $T\oplus R_h$ por expediente | ledger más módulos privados | no; aislamiento certificado (`hypothetical_runs_stay_outside_the_history`) |
| Valoraciones y pérdidas | $\varepsilon$, fuente, estado, $\ell$ por posición | ledger | no |

- $k$ es un número de secuencia del ledger. No es un índice temporal de la
  ontología ni un `PropositionTime`.
- Que una descripción no puede alterar consecuencias históricas se sigue de
  las firmas: `close_program(seeds, &Module, clock)` no recibe $d$ ni $z$.
- Adoptar una hipótesis en la historia sigue siendo la operación certificada
  de `dacf1d4` (plantilla capturada más anclaje) y queda fuera de este
  experimento.

## 11. Correspondencia con las APIs

| Operación | API | Estado |
|---|---|---|
| Elaborar y admitir Γ | `load_formal_module` (`modules.rs:17`); `proposal::admit`, `check_scope` (`8edcc30:proposal.rs:228,340`) | EXISTE (el segundo, en el explorador) |
| Validador de perfil | `exploration::classify` (`8edcc30:exploration.rs:896`) | EXISTE en el explorador |
| $T\oplus R$ | `exploration::install` (`:36`), `certify_addition` (`:258`) | EXISTE en el explorador |
| Evaluar $\operatorname{ev}(h,S)$ | `close_program` (`derive.rs:192`); `ForwardSession::for_module` (`8edcc30:derive/steps.rs:33`) | EXISTE |
| Provenance para I2 | `Closure.derivations` / `DerivationRecord` (`typed.rs:216-248`) | EXISTE |
| Requisitos para I3 | `query_active_program_residual` (`residual/active.rs:70`) | EXISTE |
| Cobertura y ⪯_g | `frontiers`, `signature`, `covers` (`8edcc30:exploration.rs:417,514,520`) | EXISTE en el explorador |
| Partición y discriminantes | `kernel`, `meet`, `finer`, `Partition`, `discriminants` (`:527-626`) | EXISTE en el explorador |
| Espacio de versiones | `submit`, `filter` | TEST (`legal_geometry_conversation_tests.rs`) |
| Minimax voraz | `scores`, `best` | TEST (ídem `:438,467`) |
| Ledger, presupuesto, canal | `tools/normative_explorer/{ledger,episode,channel,manifest}.py` | EXISTE en el explorador |
| Comprobador de afirmaciones | `claims.py` (`claims/1`) | EXISTE; hay que ampliarlo con procedencias |
| `canon`, $\operatorname{id}(\Gamma)$ | — | NUEVO |
| $\rho(\Gamma)$, renderizado determinista de $f$ | base TEST `render_rule` | NUEVO |
| Validador R1–R6 | — | NUEVO |
| `Revise`, reescritura D, vistas $\iota$ | — | NUEVO (llamadas pagadas; requieren autorización) |
| $E$ y caché de $z$ | — | NUEVO; proveedor ABIERTO |
| $s_k$, $\mathcal N_k$, Spearman, RBO | — | NUEVO (harness, sin semántica jurídica) |
| Políticas A, A_fb, B, C, D; `elegir` | — | NUEVO |
| Tabla de puntuación offline | sobre `close_program` | NUEVO |
| Pérdida con intervalo | — | NUEVO |
| Vocabulario opaco | renombrado de símbolos en una copia de la fixture | NUEVO; las ontologías originales no se tocan |
| Desplazamientos $\delta$ y su admisibilidad | elaboración de experimentos (`elab.rs:344-380`) | EXISTE la validación; NUEVO el generador de $\delta$ |

Nada de lo NUEVO toca el motor ni LegalGeneral. Todo pertenece al harness o a
`src/experiments`.

## 12. Decisiones abiertas

1. **Valoraciones independientes (bloqueante).** H2–H5 necesitan $\mathcal V_O$
   y $\mathcal V_R$ con fundamento externo y en número suficiente para varios
   splits. Hoy hay una sola valoración documentada, y en un dominio con cabeza
   artificial.

   Mi propuesta tiene dos regímenes, declarados por separado:
   - (i) valoraciones jurídicas aportadas por ti o por quien designes;
   - (ii) un objetivo sintético en $H$, como control negativo y como prueba
     de la cadena completa, nunca como evidencia jurídica.
2. **Catálogo $H$ y dominio.** El subperfil de `compliance` es exhaustivo pero
   no tiene valoraciones. El dominio de la historia temporal usa cabezas reales
   de LegalGeneral, pero no tiene catálogo. Cualquiera de los dos hace falta
   concretarlo, con $|H|$ fijado antes del episodio.
3. **Régimen de coste.** Hay que decidir si se impone un tope $B\ll|H|$, que
   es artificial en este subperfil (Proposición 1), o si las métricas
   principales se limitan a $\ell_R$ y a valoraciones.
4. **Embedder.** API de pago, con autorización y presupuesto, o modelo local
   dentro de 1 GiB sin swap.
5. **Base de código.** Integrar en local la rama del explorador. Sigue
   pendiente de tu autorización (informe de la Etapa 0, §7).
6. **Presupuesto de llamadas.** Las revisiones C y D requieren llamadas
   pagadas: aproximadamente (nodos refutados × iteraciones × condiciones ×
   semillas), más los embeddings de $|H|$ descripciones. No se estimará ni se
   ejecutará sin tu autorización.

## 13. Verificación

- No se ha modificado código ni se han ejecutado tests.
- Las ubicaciones de `8edcc30` se han comprobado con `git show`:
  `exploration.rs`, con `install`, `frontiers`, `signature`, `covers`,
  `kernel`, `meet`, `finer`, `Partition`, `discriminants` y `classify`.
- Las de esta rama se han comprobado con `grep`:
  `legal_geometry_conversation_tests.rs`, con `filter` `:276`, `submit` `:322`,
  `scores` `:438` y `best` `:467`.
