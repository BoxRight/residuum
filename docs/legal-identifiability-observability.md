# Observabilidad de los casos discriminantes

Investigación en `experiment-legal-order-geometry`, continuando
[los cinco casos discriminantes](legal-identifiability-informative-cases.md).
Se mantienen las nueve hipótesis, las 13 etiquetas externas y las 27 reglas de
`LegalGeneral`. No se generan hipótesis nuevas ni se integran revisiones.
No se modifica el motor ni se utilizan datos de expedientes reales.

El resultado principal es que **las tres clases del protocolo anterior no son
clases globales de equivalencia**. Al admitir dos seeds ya existentes en la
ontología, `prohibition` y `~Act`, se separan en siete clases de eventos.
Dos parejas conservan equivalencia de eventos bajo una condición estructural
demostrable, pero no tienen la misma provenance.

## Qué puede observarse y qué entrada se admite

El protocolo de este estudio admite como datos externos únicamente eventos
ground de verbos `Seeded`, con cualquiera de las dos polaridades evidenciales
admitidas por el lenguaje. Los hechos `Derived` deben obtenerse mediante reglas;
una premisa Derived de un experimento de teoría no se convierte en seed.

Para un módulo hipotético fijo H y una entrada admitida S, se observa todo
`Known` al alcanzar el punto fijo: eventos con argumentos, polaridad, categoría,
tiempo y metadata operacional del evento. No sólo se pregunta por `compliance`.
La comparación experimental utiliza la representación completa de `Event`,
además de la identidad lógica; conserva autoridad y fecha como argumentos.

Como los seeds son iguales en las dos evaluaciones y ninguna regla los elimina,
comparar todo `Known` equivale aquí a comparar todas las consecuencias Derived
y Effect. Derivar un Effect no ejecuta State. Tampoco se genera automáticamente
evidencia histórica al derivarlo.

Se conserva una segunda observación independiente: las justificaciones con su
regla, antecedentes ordenados, sustitución, multiplicidad y módulo de origen.
No se identifica esa observación con el conjunto de eventos. Se ignora únicamente
el valor de `derivedAt` al comparar ejecuciones del mismo programa.

## Clasificación de los cinco casos

Se conservan las abreviaturas ground del estudio anterior:
A = Act, P = permissionFiled, M = omissionProhibition, O = obligationFiled,
R = right, C = compliance, D = disputedPerformance y V = violation.
Se agregan al **dominio de entradas**, no a la ontología:

- B = `prohibition`, con los argumentos normativos de O y autoridad administrative.
- N = `~Act`, exactamente el mismo acto A, con autoridad private y tiempo gTau.

La [fixture de accesibilidad](../examples/experiments/legal_geometry/observability_cases.res)
expresa exclusivamente seeds existentes. Las dos formas de acceso son distintas:
introducir exactamente la instantánea del caso, o alcanzar sus eventos dentro
de un cierre que puede contener más información.

| Caso discriminante | ¿Todos sus hechos son Seeded? | Camino desde seeds mediante reglas originales | ¿Puede alcanzarse sin C previamente? |
|---|---|---|---|
| AP | sí | entrada A + P | sí |
| AM | sí | entrada A + M | sí |
| AD | no: D es Derived | O + A + N → C y V → D | no |
| AV | no: V es Derived | B + A → V | sí |
| APV | no: V es Derived | B + A + P → V y permission | sí |

Ninguno de los cinco exige suministrar directamente Derived si sólo queremos
alcanzar sus eventos como parte de un cierre. AD es la excepción para la
**independencia discriminante**: para obtener D ya hubo que obtener el mismo C.
La instantánea teórica AD sin C puede suministrarse mediante `premises`, pero
no aparece como un estado alcanzable de esta ontología desde seeds solamente.

AV y APV tienen fuentes de V que no presuponen C. La evidencia adicional B no
es inocua: puede producir `notDo` y `patrimonialLiability`. Se comparan esas
consecuencias, en vez de identificar el camino alcanzable con la instantánea
original que contenía menos hechos.

No hay Program, Default ni asociación `Verb.program` en la ontología general
cargada. El test inspecciona el AST para comprobarlo. Estos caminos utilizan sus
reglas ordinarias mediante `close_program`; no se inventa una activación dinámica.

## Fronteras de acceso en una ventana finita ampliada

**MODEL RESULT.** Se examinan los 128 subconjuntos del catálogo Seeded
`{O,R,B,M,P,A,N}`. Su cierre conjunto con la ontología original tiene 17 eventos.
No se enumeran los 131.072 subconjuntos arbitrarios de esos 17 eventos: muchos
introducirían directamente Derived y cambiarían el protocolo.

Para cada caso T se distingue:

\[
T\subseteq C_G(S)
\]

de:

\[
T\subseteq C_G(S)\quad\text{y}\quad C\notin C_G(S).
\]

La segunda condición mide si podemos alcanzar el soporte sin haber resuelto ya
el objetivo. Es una condición sobre el cierre original, no una etiqueta jurídica.

| Caso | Todos los soportes mínimos de acceso en esta ventana | Soportes mínimos de acceso sin C |
|---|---|---|
| AP | AP | AP |
| AM | AM | AM |
| AD | OAB, RAB, OAN, RAMN | ninguno |
| AV | AB, OAN, AMN | AB, AMN |
| APV | ABP, OAPN, AMPN | ABP, AMPN |

La minimalidad es por inclusión. Los soportes de distinto tamaño pueden ser
incomparables; no se elimina RAMN por tener más hechos que OAB.
La oposición A/N puede producir `inconsistent` mediante la regla ya existente;
no elimina A, N, C ni V, ni implica una política especial de selección.

## Comparación de todas las consecuencias

**MODEL RESULT.** Para cada una de las nueve hipótesis anteriores y cada uno
de los 128 inputs, se calcula un cierre real: **1.152 evaluaciones**.
Se cotejan con una saturación de hiperaristas ground independiente del evaluador.
Las hiperaristas se extraen de las reglas elaboradas; la saturación no usa
consultas residuales. Se verifica que el dominio de 17 eventos contiene todas
las salidas y que coinciden los eventos completos, no sólo una máscara de C.

Se auditan todas las justificaciones contra sus reglas, testigos, sustituciones
y orígenes. En las nueve ejecuciones con todo el catálogo, también se compara
el evaluador de referencia con el actual, conservando la multiplicidad de registros.

Las 36 parejas arrojan 34 diferencias y dos igualdades de eventos. Las clases son:

| Clase de eventos | Miembros |
|---|---|
| 1 | AP |
| 2 | AM+AD; AM+APD |
| 3 | AM+AD+AV |
| 4 | AM+APD+APV |
| 5 | APM+AD; APM+APD |
| 6 | APM+AD+AV |
| 7 | APM+APD+APV |

Las antiguas clases AM y APM se dividen cada una en tres. Por ejemplo:

- AB separa AM+AD de AM+AD+AV: B y A producen V sin necesitar C; el control AV
  del segundo paquete añade C.
- ABP separa AM+APD de AM+APD+APV.
- AB separa las variantes AV de las variantes APV, porque no se aportó P.

Estos son contraejemplos con inputs legales para el protocolo Seeded. Por tanto,
la falta de equivalencia de esas parejas no depende de extrapolar el modelo finito.

Las cuatro sondas AP, AM, AB y ABP distinguen las siete firmas de C de esta
familia. No se optimiza el número de preguntas ni se asignan respuestas externas:
todos los campos `external_answer` siguen pendientes y se conservan las nueve
hipótesis. Esas firmas de C no justifican ignorar otras consecuencias en general.

Como controles de otros subdominios, las dos parejas iguales se evalúan también
con todo el catálogo más cada uno de los ocho escenarios Seeded originales:
performance, authority, legislation, transferCase, otherTime, otherAuthority,
holding y extinction. Las 16 comparaciones preservan todos los eventos,
incluidas conclusiones legislativas, de jerarquía, patrimoniales y Effects.
No se enumeran todas las combinaciones de estos fondos: el resultado general
siguiente proviene de un argumento estructural distinto.

## Equivalencia general condicionada para dos parejas

**THEOREM, bajo las hipótesis indicadas.** Fijemos la ontología G y una de las
nueve extensiones ground H ya definidas. Para cualquier entrada S compuesta sólo
por Seeded, retirar de H la cláusula hipotética AD→C o APD→C no cambia ningún
evento de su menor cierre. No se afirma igualdad de las justificaciones.

Hipótesis que el test inspecciona en el AST:

1. `disputedPerformance` es Derived, por lo que no entra mediante S.
2. Su único productor es `performanceDispute`, con cuerpo C & V y cabeza D.
   La ocurrencia de C conserva exactamente los argumentos, polaridad y tiempo
   del D producido, cambiando sólo el nombre del verbo.
3. Las cláusulas hipotéticas que usan ese D tienen como cabeza precisamente
   ese C; no un C de otra norma, fecha, autoridad, persona o tiempo.
4. No hay otros productores introducidos por programas asociados, defaults,
   imports adicionales o realimentación de State.

Demostración. Considérese el menor cierre del programa sin la cláusula de retorno.
Si contiene D, no pudo obtenerlo como seed ni de otra regla: se obtuvo de C & V,
así que ya contiene el C correspondiente. Por tanto toda instancia habilitada
de AD→C o APD→C tiene su cabeza dentro de ese cierre. El cierre es también
cerrado para el programa con retorno. Por minimalidad y por inclusión de los
conjuntos de reglas, los dos menores cierres coinciden. El argumento no depende
del número de átomos ni de los demás seeds o conclusiones.

Tras retirar esos retornos en copias del harness, AM+AD y AM+APD tienen las
mismas declaraciones activas; también APM+AD y APM+APD. Se verifica igualdad
de declaraciones y de los orígenes de las declaraciones conservadas. Aplicando
el argumento anterior:

\[
\forall S\text{ Seeded},\quad C_{AM+AD}(S)=C_{AM+APD}(S),
\]

\[
\forall S\text{ Seeded},\quad C_{APM+AD}(S)=C_{APM+APD}(S).
\]

Esta igualdad cubre **todos los eventos del cierre**, por lo que cualquier
observación definida únicamente sobre esos eventos coincide, también fuera de
la ventana de 17 puntos. Las otras parejas tienen un contraejemplo Seeded finito.
Así, bajo este protocolo y estos programas fijos, las siete clases describen la
equivalencia de eventos; no sólo la coincidencia en las cuatro sondas.

No es equivalencia contextual frente a cualquier programa futuro. Una regla
importada que produzca D independientemente de C, o permitir premisas Derived,
rompería la segunda o la primera hipótesis. Precisamente la instantánea AD del
estudio anterior distingue los retornos si se admite D directamente.

## La provenance sigue distinguiendo esas parejas

**EXPERIMENTAL OBSERVATION.** Con todo el catálogo Seeded, ambas parejas
iguales por eventos tienen registros diferentes de C:

```text
hypotheticalComplianceAD:  A & D     <= C
hypotheticalComplianceAPD: A & P & D <= C
```

El resultado C ya existía, pero cada regla añade una justificación distinta.
El test no borra esas diferencias ni identifica nombres de regla, antecedentes
o sustituciones para forzar equivalencia. Se exportan completos los registros
y los módulos de origen; sólo se omite el sello concreto de ejecución.

Por tanto **no se deben llamar indistinguibles si `explain` forma parte de la
observación**. Son equivalentes para consecuencias expresadas como eventos,
no para el historial de justificaciones. Una justificación operacional diferente
tampoco prueba una razón normativa diferente ni recupera una razón desconocida.

## Artefactos y reproducción

- [Fixture Seeded](../examples/experiments/legal_geometry/observability_cases.res).
- [Verificador experimental](../src/bin/residuum/cli/legal_geometry_observability_tests.rs).
- [Acceso y sus fronteras](legal-geometry/observability/observability-paths.tsv).
- [Universo tipado de 17 eventos](legal-geometry/observability/observability-ground-universe.tsv).
- [Los 1.152 cierres](legal-geometry/observability/observability-engine-closures.tsv).
- [Clases de eventos](legal-geometry/observability/observability-seeded-classes.tsv).
- [Las 36 parejas y sus discriminantes mínimos](legal-geometry/observability/observability-discriminating-pairs.tsv).
- [Sondas con respuestas pendientes](legal-geometry/observability/observability-seeded-probes.tsv).
- [Controles de otros subdominios](legal-geometry/observability/observability-background-observations.tsv).
- [Provenance de acceso](legal-geometry/observability/observability-path-provenance.tsv).
- [Provenance de hipótesis y fondos](legal-geometry/observability/observability-hypothesis-provenance.tsv).

Los tests regeneran estos TSV en `target/legal-geometry/observability-*.tsv`.
La saturación independiente y el retiro de retornos ocurren sólo en el harness;
ninguna declaración jurídica fuente se modifica.

Los SHA-256 siguen siendo los de las fuentes de referencia:

| Archivo | SHA-256 |
|---|---|
| legal_monotonic.res | `24655c98ede7b128b8cd82df3a4473c114428b987bb0fe6a6d370c2b37f5575e` |
| vendor/legal_general.res | `639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc` |
| vendor/legal_patrimonial.res | `f7daf4d5eb937a7fde8534dba5618acc50d5aaa5b05ff86c9656892eb3a03fbf` |

Compilación y ejecuciones individuales mediante `scripts/test_safe.py`, con
1 GiB, sin swap, un trabajo y un hilo, timeout de test de 15 segundos.
No se ejecuta la suite completa. Se incluye la regresión de los dos tests de
identificabilidad anteriores. [Registro de los cuatro tests](legal-geometry/observability/verification.md):
todos pasan individualmente; el mayor tiempo es 1,8688 s y el mayor pico RSS
de test es 15,93 MiB. La compilación separada tarda 2,2471 s, con 351,28 MiB RSS.
Los registros completos están en `target/test-diagnostics/20261010T164715552807Z-474718`
y `target/test-diagnostics/20261010T164659825312Z-474312`.

No se decide una revisión jurídica, no se contesta ninguna pregunta externa,
no se reduce el espacio de hipótesis y no se demuestra que se haya identificado
una justificación normativa. Se establece qué diferencias son accesibles por
evidencia y qué proyección de observación permite hablar de equivalencia.

La [retícula de particiones](legal-identifiability-partition-lattice.md) organiza
estos resultados guardados y calcula lotes mínimos de observaciones, separando
eventos y provenance, sin volver a ejecutar cierres.
