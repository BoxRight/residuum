# Fronteras mínimas de derivabilidad jurídica

Las fronteras describen alternativas suficientes para una conclusión concreta.
Para el compliance seleccionado hay dos soportes mínimos Seeded; para violation
hay cinco; para annulled hay cuatro. Esto convive con la base extrema única de
cada estado cerrado completo: son problemas diferentes.

Continuación: [comparación con las consultas residuales reales](legal-ontology-residual-frontier-comparison.md).

Se trabaja en `experiment-legal-order-geometry`, usando las mismas fuentes y el
mismo universo del [estudio geométrico](legal-ontology-order-geometry.md).
No se modifica el motor ni ninguna declaración jurídica. Los datos son
sintéticos, sin expedientes de 12hectareas; los efectos permanecen como eventos
conocidos y no se ejecutan sobre State.

**Definición y dos dominios de evidencia.**

Para `q∈E`, con `E=C_P(Σ)` de 73 eventos ground:

\[
\mathcal R_q=\{S\subseteq E\mid q\in C(S)\},\qquad
\partial_{\min}\mathcal R_q=\min_{\subseteq}\mathcal R_q.
\]

En este dominio completo, extensividad hace que `{q}` siempre sea un soporte
mínimo: q ya está conocido. Por eso el análisis distingue una segunda frontera:

\[
\partial^{\Sigma}_{\min}\mathcal R_q
=\min_{\subseteq}\{S\subseteq\Sigma\mid q\in C(S)\},
\]

donde Σ contiene los 23 eventos Seeded de las fixtures, incluida evidencia
negativa explícita. Esta frontera permite investigar derivaciones a partir de
evidencia externa. No se identifica Σ con todos los posibles hechos jurídicos:
otros objetos, causas o tiempos quedan fuera del universo declarado.

Como todo subconjunto de un S contenido en Σ también está contenido en Σ,
la frontera restringida coincide con filtrar la frontera completa por `S⊆Σ`.
Esto se demuestra por inclusión y se verifica comparando dos cálculos: uno con
identidades iniciales para todos los eventos y otro sólo para los Seeded.

Ni `{q}` ni una premisa Derived se fabrican como resultados: son entradas de
teoría admitidas por la definición de K. La frontera no agrega hipótesis a Known,
no elige una explicación preferida y no modifica la consulta residual.

**THEOREM: representación exacta de una región por su frontera.**

En un universo finito y con cierre monotónico:

\[
\mathcal R_q=\bigcup_{B\in\partial_{\min}\mathcal R_q}\uparrow B,
\qquad \uparrow B=\{S\subseteq E\mid B\subseteq S\}.
\]

Si S deriva q, alguna reducción mínima B de S también lo deriva. Si S contiene
un soporte suficiente B, monotonicidad conserva la conclusión. La frontera es
una antichain: ninguno de sus miembros contiene estrictamente otro miembro.
Este resultado no necesita antiintercambio ni exige una base única del objetivo.

Para dos conclusiones a y b, la inclusión de regiones se decide exactamente por:

\[
\mathcal R_a\subseteq\mathcal R_b
\iff
\forall A\in\partial_{\min}\mathcal R_a\;
\exists B\in\partial_{\min}\mathcal R_b:\ B\subseteq A.
\]

Si falla, el soporte A es un contraejemplo mínimo para a que no hace derivable b.
Un núcleo necesario para q alcanzable es la intersección de sus soportes
mínimos. Puede no ser suficiente: para compliance, Act está en todos los
soportes Seeded, pero Act por sí solo no deriva compliance.

Estas operaciones son de orden entre conjuntos de evidencia y regiones. No se
identifican con `≤_L`, el residual o los operadores del modelo FOUR.

**Método y correctez en el universo declarado.**

Se reutiliza el análisis de soportes del harness anterior, ahora parametrizado
por el conjunto de entradas permitidas. El matcher existente instancia las
reglas reales contra E: 65 cláusulas ground G y 69 P. Para cada evento permitido
se comienza por su soporte singleton. Para cada cláusula se combinan por unión
los soportes de todas sus premisas, conservando sólo conjuntos mínimos para su
consecuencia. Se itera hasta que las antichains no cambian.

`THEOREM`: toda derivación Horn finita se descompone en árboles de aplicaciones
de esas cláusulas. Sus hojas dan un soporte que el procedimiento considera.
Descartar un superset no pierde un soporte mínimo posterior, porque reemplazar
un soporte por uno contenido en él no aumenta su unión con otras premisas.
Por ello el cálculo es suficiente y completo para este hipergrafo ground finito.
E está cerrado: para S⊆E ninguna prueba necesita un evento exterior a E.

Esto no prueba completitud general del motor residual ni del entailment FOUR.
No se enumeran `2^73` teorías ni `2^23` entradas. Los cálculos se realizan sólo
en tests y se verifican mediante `close_program`, sin otra semántica del motor.

**MODEL RESULT: inventario de fronteras y certificados.**

| Ontología | Dominio de entrada | Conclusiones alcanzables | Soportes mínimos | Retiradas individuales comprobadas |
|---|---|---:|---:|---:|
| General | E completo | 73 | 284 | 601 |
| General | Σ Seeded | 61 | 99 | 195 |
| Patrimonial | E completo | 73 | 332 | 703 |
| Patrimonial | Σ Seeded | 73 | 114 | 231 |

Se verifican 829 certificados de soporte por modelo/dominio y 1 730 retiradas.
Cada soporte hace derivable su objetivo. Retirar cualquiera de sus elementos
impide derivarlo; por monotonicidad esto descarta todos sus subconjuntos propios.
Se conservan **todas** las justificaciones del cierre, con reglas, antecedentes
ordenados, sustituciones canónicas, tipos, tiempos y origen de módulo/archivo.

Para cada soporte patrimonial, el cierre y la multiplicidad de records coinciden
con `legal_monotonic.res`. El origen se registra separadamente: las reglas
generales pertenecen a LegalGeneral y complianceAsset a LegalPatrimonial;
`derivedAt` no entra en la comparación entre ejecuciones.

Se evalúan además todas las entradas de cinco ventanas pequeñas: prestaciones
(128), autoridad (256), legislación (8), transferencia (2) y tenencia (2), en
ambos modelos. Para cada uno de esos 792 cierres y cada q∈E se contrasta la
predicción «S contiene algún soporte de q» con el resultado real de `close`.
Las **57 816 comprobaciones** coinciden.

**Conclusiones seleccionadas.**

Estos conteos corresponden a los eventos ground completos identificados en
[goal-frontier-results.json](legal-geometry/goal-frontier-results.json).
Las conclusiones normativas usan causa gNorm; law usa gStatute y annulled
relaciona gLow/gHigh. No se proyectan causas, autoridad o tiempos para fusionar
hechos distintos.

| Conclusión | Frontera E, G / P | Frontera Σ, G / P |
|---|---:|---:|
| obligation | 2 / 2 | 1 / 1 |
| permission | 2 / 2 | 1 / 1 |
| prohibition | 1 / 1 | 1 / 1 |
| compliance | 4 / 4 | 2 / 2 |
| violation | 8 / 8 | 5 / 5 |
| enforceability | 3 / 3 | 2 / 2 |
| inconsistent | 2 / 2 | 1 / 1 |
| disputedPerformance | 21 / 21 | 6 / 6 |
| patrimonialLiability | 9 / 9 | 5 / 5 |
| patrimonialAsset | 1 / 5 | 0 / 2 |
| owns | 2 / 6 | 0 / 2 |
| capacity | 3 / 7 | 0 / 2 |
| law | 4 / 4 | 1 / 1 |
| annulled | 9 / 9 | 4 / 4 |

El cero para el asset gNorm en G significa que **ninguna entrada de este Σ** lo
produce. G sí produce assets por patrimonialHolding; el holding presente tiene
causa gHolding, no gNorm. No se concluye que G sea incapaz de producir activos.

Para leer las alternativas de prestaciones, usamos las siguientes abreviaturas
de eventos exactos, todos en gTau:

| Abreviatura | ID | Evento |
|---|---|---|
| O | E039 | obligationFiled gNorm, autoridad administrativa |
| R | E067 | right gNorm, autoridad administrativa |
| P | E066 | prohibition gNorm, autoridad administrativa |
| M | E040 | omissionProhibition gNorm, autoridad administrativa |
| A | E002 | Act gAct, autoridad privada |
| N | E003 | ~Act gAct, autoridad privada |
| Nj | E000 | ~Act gAct, autoridad judicial |

Sujeto, conducta gConduct, objeto y beneficiario coinciden. La fecha completa
de cada uno está en [universe.tsv](legal-geometry/data/universe.tsv).
`gConduct` no es transfer. La evidencia negativa E001 tiene otro índice temporal
y no sirve para estos objetivos en gTau.

Las fronteras Seeded de esos objetivos son:

```text
compliance:           {O,A} ; {R,A}
violation:            {O,N} ; {O,Nj} ; {M,N} ; {M,Nj} ; {P,A}
enforceability:       {R,N} ; {R,Nj}
disputedPerformance:  {O,A,N} ; {O,A,Nj} ; {R,M,A,N} ; {R,M,A,Nj}
                     {O,P,A} ; {R,P,A}
inconsistent:         {A,N}
```

Los Nj sirven para violation/enforceability porque esas reglas admiten
`actAuthority` como parámetro separado de la autoridad normativa. Pero `{A,Nj}`
no produce el inconsistent privado: `conflictingActEvidence` exige igualdad de
todos los argumentos, incluida la autoridad. No se relaja el matching.

Para annulled, los cuatro soportes comparten higherAuthority y notEarlier y
alternan las parejas: permiso superior/prohibición inferior; obligación
superior/prohibición inferior; prohibición superior/permiso inferior; prohibición
superior/obligación inferior. Las normas archivadas pasan por filedPermission o
filedObligation cuando corresponde. Ninguna anulación elimina el hecho inferior.

Law tiene un soporte Seeded de tres hechos: powerGranted, Act(enact) y
legislationText. La capacidad y el poder intermedios se derivan, no se aportan
como seeds precalculados.

**Relaciones entre regiones y contraejemplos.**

Las fronteras permiten comparar regiones sin enumerar todo K. Se publican las
comparaciones entre los 14 objetivos seleccionados, separadas por modelo y
dominio, con un soporte contraejemplo cuando falla una inclusión.

| Relación propuesta | Resultado en Σ |
|---|---|
| Compliance implica violation | No: `{O,A}` |
| Violation implica compliance | No: `{O,N}` |
| Enforceability implica violation | No: `{R,N}` |
| Prohibition por sí sola implica violation | No: `{P}` |
| DisputedPerformance exige compliance y violation | Sí: su región es exactamente la intersección de ambas |
| Violation y patrimonialLiability gNorm tienen la misma región | Sí, en este universo: los únicos actos que habilitan la liability gNorm son los considerados por ese violation |
| Inconsistent implica disputedPerformance | No: `{A,N}` no contiene norma |
| DisputedPerformance implica inconsistent | No: `{R,P,A}` no contiene ~Act |

La igualdad de regiones de liability/violation es un resultado del universo
finito: liability omite el identificador del acto y un universo mayor puede
tener otros actos. No se identifica sus firmas ni se añade una regla inversa.

La igualdad de la región de dispute con la intersección usa que sólo
performanceDispute lo produce y que no se admite dispute como Seeded. El test
comprueba que su frontera es la minimización de todas las uniones de un soporte
de compliance y uno de violation. Al permitir premisas de E completo aparece
también `{disputedPerformance}`: entonces la igualdad falla. No hay inferencia
automática de sus antecedentes desde una conclusión aportada.

Estos ejemplos muestran coexistencia de regiones: compliance y violation son
incomparables pero pueden coexistir. Su coexistencia no elige ganadores ni
introduce una contradicción global; sólo dispara las reglas explícitas del dominio.

**Comparación general/patrimonial.**

`THEOREM`: añadir complianceAsset mantiene `C_G(S)⊆C_P(S)`, por tanto
`R_q^G⊆R_q^P`. Esto garantiza que cada soporte general contiene algún soporte
patrimonial, no que las familias mínimas deban ser siempre idénticas.

`MODEL RESULT`: para estas fuentes los soportes anteriores permanecen mínimos.
Cambian 12 conclusiones ground —cuatro activos y su titularidad/capacidad—, en
ambos dominios. Se añaden 48 soportes en E y 15 en Σ; no desaparece ninguno.
Las regiones de obligación, permisos, prohibiciones, cumplimiento, violación,
exigibilidad, anulación y ley seleccionadas permanecen iguales.

Para la causa gNorm y las entradas de Σ, P iguala las regiones de compliance,
asset, owns y capacity: sus dos soportes son `{O,A}` y `{R,A}`. G conserva las
regiones de compliance, pero las otras tres están vacías en Σ. En E completo,
la igualdad no se sostiene: `{asset}`, `{owns}` o `{capacity}` pueden ser premisas
independientes y no permiten concluir compliance hacia atrás.

La partición de estados por **igualdad del cierre completo** del informe de
[puntos extremos](legal-ontology-extreme-points.md) sigue siendo otro objeto.
Dos soportes mínimos de compliance conservan bases diferentes y producen
estados cerrados diferentes. Una región reúne entradas suficientes, sin
identificarlas por todo lo que conocen ni por sus justificaciones.

**Artefactos, reproducción y límites.**

- [Certificados completos de soportes y provenance](legal-geometry/data/goal-frontier-certificates.tsv).
- [Certificados al retirar cada hecho](legal-geometry/data/goal-frontier-omissions.tsv).
- [Conteos](legal-geometry/data/goal-frontier-totals.tsv) y [regiones observadas](legal-geometry/data/goal-frontier-regions.tsv).
- [Inclusiones y contraejemplos entre regiones](legal-geometry/data/goal-frontier-relations.tsv).
- [Resultados estructurados](legal-geometry/goal-frontier-results.json).

```text
python3 scripts/test_safe.py --test-target cli \
  --test cli::tests::legal_geometry::frontiers::minimal_frontiers_preserve_supports_witnesses_and_origins \
  --test cli::tests::legal_geometry::frontiers::frontier_upsets_reconstruct_all_controlled_derivability_regions \
  --test cli::tests::legal_geometry::full_event_frontiers_without_powerset_enumeration
python3 scripts/analyze_legal_geometry.py --render
```

Pasaron dos tests nuevos y una regresión, en procesos individuales limitados.
No se ejecutó la suite completa. El runner conserva 1 GiB, sin swap, un trabajo
de Cargo, un hilo y sus timeouts. La compilación inicial consumió 426.55 MiB RSS.
La última ampliación de la comprobación de dispute se recompiló con 359.75 MiB.

| Test | Tiempo | Pico RSS |
|---|---:|---:|
| Soportes, minimalidad y provenance | 1.5257 s | 16.91 MiB |
| Regiones y conjunción de fronteras, versión final | 0.8145 s | 15.56 MiB |
| Regresión del cálculo anterior | 0.8345 s | 15.50 MiB |

Logs: `target/test-diagnostics/20261010T002201957446Z-612226`,
`20261010T002226416981Z-612767` y `20261010T002440587742Z-614480`.
El harness está en `src/bin/residuum/cli/legal_geometry_frontier_tests.rs`.
Sólo se extrae como helper compartido el cálculo de antichains del test anterior;
no se modifica matching, close, Rule, Program, imports, defaults ni runtime.
Python resume los exports y aplica el criterio demostrado de inclusión entre
regiones; no ejecuta otra implementación del cierre.

Los teoremas son sobre cierres Horn finitos y, donde se indica, las fuentes
acíclicas. Los números, igualdades particulares y alternativas son resultados
del universo E/Σ. No se propone una geometría métrica ni una nueva doctrina
jurídica; tampoco una conjetura de completitud sobre objetos o tiempos externos.
