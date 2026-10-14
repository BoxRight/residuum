# Construcción experimental de funciones justificativas por recalcitrancia

Se trabaja en `experiment-legal-order-geometry`. La norma jurídica original
permanece fija. Las funciones aprendidas son hipótesis compatibles con
valoraciones estipuladas: no son nuevas normas aprobadas ni razones históricas,
morales o jurídicas identificadas como verdaderas.

El resultado es una construcción progresiva **de un espacio de funciones**, no
de una función única: seis observaciones reducen 168 candidatas a 11. Todas
corrigen las dos discrepancias observadas, pero discrepan en otros cinco casos.
Sus revisiones hipotéticas conservan antiintercambio en el dominio comprobado.
Ninguna fue seleccionada por esa propiedad ni integrada en la ontología.

## 1. Norma, hechos y dominios

Se carga la ontología `LegalGeneral` mediante el import existente de
[general.res](../examples/experiments/legal_geometry/general.res). Se conservan
sus 27 reglas, tipos, efectos y orígenes. No se usan datos de un expediente real.

El catálogo inicial tiene cuatro eventos ground Seeded:

| Símbolo | Verbo | Datos comunes relevantes |
|---|---|---|
| O | `obligationFiled` | gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate |
| R | `right` | los mismos argumentos normativos |
| A | `Act` | gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate |
| P | `permissionFiled` | los mismos argumentos normativos |

Todos tienen índice proposicional `gTau`. Los símbolos son abreviaturas de eventos
tipados completos, no proposiciones sin autoridad o fecha. `gConduct` no es la
constante `transfer`. El objetivo fijo es:

```text
compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
```

Definimos la clasificación normativa utilizando sólo el cierre original:

\[
N(S)=1\iff compliance\in Close_G(S).
\]

**MODEL RESULT.** En los 16 subconjuntos de `Σ={O,R,A,P}`, el evaluador certifica:

\[
\mathcal B_N=\{OA,RA\},\qquad
\mathcal R_N=\uparrow OA\cup\uparrow RA.
\]

Hay seis casos positivos: OA, RA, ORA, OAP, RAP y ORAP.
`E_0=Close_G(Σ)` contiene ocho eventos: O, R, A, P, obligación, permiso,
compliance y do. Éste es el universo de la enumeración geométrica exhaustiva.

Como control separado de consecuencias se añade N̄=`~Act`, con exactamente los
argumentos y tiempo de A. Para evitar confundirlo con la función normativa N,
los TSV lo abrevian con la letra `N` dentro de nombres de **casos**. Las 32
entradas de `Σ∪{N̄}` producen un universo de 14 eventos. Además de los ocho
anteriores aparecen N̄, inconsistent, violation, patrimonialLiability,
enforceability y disputedPerformance. Este control no aporta etiquetas nuevas
ni añade `~Act` al vocabulario de las funciones justificativas.

Los [casos Formal](../examples/experiments/legal_geometry/recalcitrance_cases.res)
expresan únicamente datos existentes. El harness utiliza sus inputs elaborados
y `close_program`; no solicita que la API residual resuelva sus parámetros
pendientes, ni implementa aprendizaje dentro del motor o de la CLI.

## 2. Lenguaje de hipótesis

Cada hipótesis es una anticadena θ de subconjuntos de Σ:

\[
J_\theta(S)=1\iff\exists B\in\theta:\ B\subseteq S.
\]

La región es la unión de los conos de esos soportes. No se conserva un soporte
que contiene estrictamente a otro de la misma familia. La familia vacía expresa
la función siempre falsa; la familia `{∅}` expresa la siempre verdadera.

Se generan directamente todas las anticadenas de `P(Σ)`: **168 funciones
monotónicas distintas**. No se enumeran todas las subfamilias de soportes para
después descartar las redundantes. Ninguna función, incluida la de la norma,
se designa como verdadera o preferida.

**THEOREM en este lenguaje finito.** Cada región ascendente tiene una única
anticadena de mínimos que la representa. Por tanto dos familias canónicas
distintas representan funciones distintas sobre los 16 casos. Esto no identifica
la razón normativa: sólo identifica una representación clasificatoria.

La monotonía es una restricción del lenguaje de hipótesis. Una valoración que
rechace un superset de un caso ya aceptado puede resultar inexpresable aquí.
La exclusión de ausencia, excepciones, contextos adicionales y hechos fuera de
Σ debe tenerse en cuenta al interpretar la compatibilidad.

## 3. Secuencia de experiencias independientes

Las [valoraciones externas](../examples/experiments/legal_geometry/recalcitrance_labels.tsv)
se declaran antes de filtrarlas. Cada fila conserva un origen
`stipulation:recalcitrance-study/k`, el carácter
`hypothetical_external_assumption` y una explicación de su papel experimental.
No provienen de una autoridad jurídica real, de una encuesta ni de `Close_G`.

| Paso | Caso | N(S) | Valoración y | Relación con la norma | Candidatas antes → después |
|---|---|---:|---:|---|---:|
| 1 | ∅ | 0 | 0 | coincidencia negativa | 168 → 167 |
| 2 | RA | 1 | 1 | coincidencia positiva | 167 → 83 |
| 3 | OA | 1 | 0 | sobreinclusión | 83 → 31 |
| 4 | AP | 0 | 1 | subinclusión | 31 → 14 |
| 5 | A | 0 | 0 | coincidencia negativa | 14 → 14 |
| 6 | P | 0 | 0 | coincidencia negativa | 14 → 11 |

El filtrado es exactamente:

\[
\mathcal H_{k+1}=\{J\in\mathcal H_k:J(S_{k+1})=y_{k+1}\}.
\]

El paso 5 no aporta una restricción nueva a las supervivientes: rechazar OA ya
obliga a rechazar A por monotonía. No se modifica la valoración para que elimine
una candidata. La función de la norma deja de ser compatible en el paso 3, pero
la **norma permanece vigente y fija en todas las evaluaciones N**.

El [inventario de funciones](legal-geometry/recalcitrance/functions.tsv) conserva
la primera observación que rechaza cada hipótesis. Los
[espacios de versiones](legal-geometry/recalcitrance/version-spaces.tsv)
registran las 168 candidatas en cada etapa, incluida la inicial. Las
[condiciones compartidas](legal-geometry/recalcitrance/shared-conditions.tsv)
distinguen soportes mínimos comunes de clasificaciones unánimes.

En la etapa final todas aceptan RA, ORA, AP, OAP, RAP y ORAP; todas rechazan
∅, O, A, OA y P. AP es un soporte mínimo común. RA es un caso positivo común,
pero no un mínimo común: algunas candidatas ya aceptan R sin A.

### Funciones finales

`|` separa soportes alternativos; la concatenación indica una conjunción de hechos.

| Hipótesis | Anticadena θ | Casos positivos |
|---|---|---:|
| H032 | R, OP, AP | 11 |
| H033 | R, AP | 10 |
| H065 | OR, RA, OP, RP, AP | 10 |
| H066 | OR, RA, OP, AP | 9 |
| H068 | OR, RA, RP, AP | 9 |
| H070 | OR, RA, AP | 8 |
| H120 | RA, OP, RP, AP | 9 |
| H121 | RA, OP, AP | 8 |
| H123 | RA, RP, AP | 8 |
| H126 | RA, ORP, AP | 7 |
| H128 | RA, AP | 6 |

La familia más pequeña por cobertura, H128, no se elige automáticamente. Tampoco
se identifica menor número de soportes, menor cobertura y adecuación jurídica.

Un control independiente estipula `J(A)=1` y `J(AP)=0`: ninguna de las 168
funciones puede satisfacerlo porque `A⊆AP`. El
[certificado de incompatibilidad](legal-geometry/recalcitrance/incompatible-control.tsv)
informa cero candidatas, sin fabricar una función ni relajar el lenguaje.
Este control no se añade a la secuencia principal.

## 4. Particiones sobre hipótesis y sobre casos

Para las hipótesis iniciales se define:

\[
h\equiv_k h'\iff
\forall i\leq k:\ J_h(S_i)=J_{h'}(S_i).
\]

Se utiliza el orden «más fina ≤ más gruesa». Si π(S) es la partición por la
respuesta a un caso, entonces:

\[
\pi_{k+1}=\pi_k\wedge\pi(S_{k+1}).
\]

**MODEL RESULT.** Se comprueba ese meet para cada etapa, mediante igualdad de
kernels, sobre las 168 hipótesis. Los números de bloques son:

```text
1 → 2 → 3 → 5 → 9 → 10 → 15
```

Las supervivientes pertenecen a un mismo bloque: todas predicen la secuencia
de valores estipulada. El refinamiento distingue también candidatas rechazadas;
por eso puede aumentar el número global de bloques sin disminuir el espacio
superviviente, como ocurre en el paso 5.

También se construye una partición de los 16 **casos**. Dos casos son equivalentes
si tienen igual N y todas las supervivientes actuales los clasifican igual.
Al eliminar funciones disminuye la información de este canal sobre casos:
la partición puede hacerse más gruesa. Sus números de bloques son:

```text
16 → 16 → 13 → 11 → 10 → 10 → 9
```

No es una contradicción con el refinamiento anterior: se particionan conjuntos
diferentes y con operaciones diferentes. Las tablas de
[hipótesis](legal-geometry/recalcitrance/hypothesis-partitions.tsv) y
[casos](legal-geometry/recalcitrance/case-partitions.tsv) conservan todos los kernels.

### Casos discriminantes pendientes

Para cada pareja superviviente se buscan **todos** los estados mínimos por
inclusión donde difieren sus predicciones, comprobando todos los subconjuntos
propios. No se presupone que la región de diferencias sea ascendente.

En la etapa final el catálogo conjunto de mínimos es:

\[
\{R,OR,OP,RP,ORP\}.
\]

Sus valoraciones externas siguen pendientes. Las
[parejas y mínimos](legal-geometry/recalcitrance/remaining-pair-minima.tsv)
indican qué predice cada hipótesis. Los
[discriminantes por etapa](legal-geometry/recalcitrance/discriminants.tsv)
conservan las oportunidades de distinción anteriores.

**MODEL RESULT.** Observar las cinco respuestas distingue las 11 funciones.
Cada caso tiene una pareja testigo que coincide en los otros cuatro y difiere
sólo en él. En este universo, los otros casos ya tienen respuesta unánime entre
las supervivientes, por lo que esos cinco son necesarios para distinguirlas todas
mediante resultados binarios. El
[certificado de identificación](legal-geometry/recalcitrance/identification-cases.tsv)
conserva las parejas. No se asignan respuestas ni se implementa un protocolo
adaptativo de deliberación.

## 5. Revisiones normativas hipotéticas y consecuencias

Para cada región candidata se calculan:

\[
U_\theta=\mathcal R_{J_\theta}\setminus\mathcal R_N,
\qquad
O_\theta=\mathcal R_N\setminus\mathcal R_{J_\theta}.
\]

U representa subinclusiones que exigirían ampliar la norma; O representa
sobreinclusiones que exigirían restringirla. Todas las supervivientes retiran OA
y añaden AP; varían en si añaden R, OR, OP, RP u ORP.

H128 tiene la diferencia mínima de cobertura: añadir AP y retirar OA. Es un
resultado de clasificación, no un criterio para aprobarla. N y H128 tienen
seis positivos cada una, pero regiones diferentes.

**Límite expresivo de las adiciones Horn.** Añadir cláusulas positivas a G no
puede eliminar OA: monotonía respecto del conjunto de reglas preservaría su
compliance. Por ello la restricción no se presenta como una simple ampliación.

El harness construye una copia hipotética privada del módulo, retira allí
`obligationCompliance` y `rightCompliance` y selecciona cláusulas ground θ del
[catálogo Formal de revisiones](../examples/experiments/legal_geometry/recalcitrance_revisions.res).
Las otras 25 reglas permanecen idénticas y con su origen original. Los archivos
jurídicos no se editan; el catálogo completo de templates no se usa como si fuera
una sola revisión. No se infiere que estas cláusulas ground sustituyan una norma
parametrizada fuera del dominio estudiado.

**MODEL RESULT.** Se evalúan las 168 funciones sobre las 16 entradas iniciales.
Las 11 supervivientes se comprueban además con y sin `~Act`, dando 2.864 cierres
de revisiones, más los cierres de referencia. En cada caso el compliance
hipotético coincide con J. El Known original y la evidencia de entrada no se
modifican durante la evaluación.

En el dominio inicial, las diferencias de eventos sólo afectan compliance:
obligación, permiso y el Effect do conservan sus productores originales. En el
control con `~Act` puede cambiar además disputedPerformance, porque consume
compliance y violation conjuntamente. Por ejemplo, OAN sigue produciendo
violation y patrimonialLiability bajo todas las supervivientes, pero pierde
compliance y disputedPerformance. OPAN, en cambio, satisface AP y permite ambas
conclusiones. No se hace que violation gane sobre compliance.

Las [diferencias completas](legal-geometry/recalcitrance/revision-consequences.tsv)
registran eventos añadidos y perdidos, con tipos, autoridad, vigencia y tiempo.
No hay ejecución de State, efectos patrimoniales nuevos automáticos ni un puente
de State a evidencia histórica.

## 6. Certificación geométrica y de provenance

Se extraen del AST las cláusulas ground reales y se realiza una saturación de
bitsets independiente de `close_program`. No se usa la función J como evaluador
de las restantes consecuencias jurídicas.

**MODEL RESULT.** Para la norma y las 168 revisiones se calculan los 256
subconjuntos de E₀: 43.264 cierres matemáticos. Se verifican extensividad,
idempotencia, monotonía por todas las inclusiones inmediatas y antiintercambio
en todos los estados cerrados. Los cierres se contrastan con el evaluador real
para todas las entradas Seeded del dominio inicial.

Todas estas revisiones conservan antiintercambio. La función constante verdadera
tiene cierre vacío no vacío por su regla I; se registra ese caso y no se afirma
que cumpla además la condición `C(∅)=∅`. Ya fue rechazada en el primer paso.
Las 11 supervivientes sí tienen cierre vacío vacío.

La norma tiene 78 estados cerrados en E₀; las supervivientes tienen entre 67 y
80. No existe una inferencia jurídica de validez a partir de ese número.

Este resultado depende del lenguaje elegido: los nuevos soportes utilizan sólo
Seeded y ninguna regla original produce Seeded. Sus aristas hacia compliance
no crean un retorno a las premisas. No se han examinado aquí hipótesis que
dependan de conclusiones Derived como disputedPerformance; precisamente esas
dependencias podían romper antiintercambio en el estudio anterior. Conservarlo
en esta familia no prueba una superioridad jurídica de sus justificaciones.

Se inspecciona además el grafo ground del universo ampliado de 14 eventos:
permanece acíclico para todas las candidatas. Bajo finitez, cierre del universo
y estas dependencias, el argumento condicional de antiintercambio de la
[teoría geométrica](legal-ontology-order-geometry.md) se aplica también allí.
No se enumeran sus 16.384 teorías para cada candidata ni se confunde esta
demostración condicionada con la enumeración exhaustiva de E₀.

Las [tablas geométricas](legal-geometry/recalcitrance/geometry.tsv) conservan
ambos alcances. El filtro de aprendizaje utiliza exclusivamente valoraciones,
no antiintercambio, DAG, número de reglas o propiedades jurídicas supuestas.

Cada justificación del evaluador se audita mediante su regla real, antecedentes
ordenados y sustitución; se comprueba su origen. Las reglas retenidas siguen en
`LegalGeneral`; los templates pertenecen a `HypotheticalJustificatoryRevisions`.
Los tiempos y términos tipados se conservan. `derivedAt` es un sello operacional
y no se utiliza para comparar ejecuciones.

Los artefactos guardan [40 payloads únicos](legal-geometry/recalcitrance/provenance-records.tsv)
de eventos y justificaciones y
[19.150 ocurrencias](legal-geometry/recalcitrance/closure-provenance.tsv).
Cada ocurrencia referencia el payload completo, sin eliminar repeticiones en un
contexto ni justificaciones distintas. Es una codificación del informe para
evitar repetir texto, no una modificación de deduplicación del motor.

## 7. Qué se ha construido y qué no

| Cuestión | Resultado |
|---|---|
| Compatibilidad | Once funciones satisfacen las seis valoraciones estipuladas. |
| Identificabilidad | Son distintas en el universo y pueden separarse mediante cinco casos adicionales; sus respuestas aún no existen. |
| Adecuación jurídica | No evaluada; requiere deliberación independiente sobre los casos y el lenguaje de hipótesis. |
| Función normativa N | Permanece fija y certificada por el cierre original. |
| Revisiones | Copias hipotéticas con consecuencias y geometría auditadas; ninguna integrada. |
| Razón normativa verdadera | No se presupone su existencia como función del catálogo ni se identifica. |

**THEOREM del procedimiento.** El filtrado mantiene exactamente las funciones
del espacio inicial que satisfacen las observaciones recibidas. No puede
introducir una función excluida del lenguaje inicial. Conocer todas las
clasificaciones de un universo finito determina una función de ese universo,
pero no su interpretación normativa ni su extensión a nuevos hechos.

La representación actual no aprende una política basada en ausencia de A, una
prioridad entre normas o una excepción negativa. Tampoco modela diferencias
de autoridad, fecha, personas u objetos fuera de las constantes fijadas. Las
etiquetas negativas son valoraciones 0, no eventos `EvidentialNot`.

El experimento muestra que Residuum puede certificar consecuencias de hipótesis
y que un harness simbólico puede revisar progresivamente funciones justificativas
representadas por anticadenas. No incorpora una primitiva de aprendizaje al
lenguaje ni demuestra que la recalcitrancia produzca una justificación única o
jurídicamente adecuada.

## 8. Reproducción

El [verificador experimental](../src/bin/residuum/cli/legal_geometry_recalcitrance_tests.rs)
contiene tres tests individuales: filtrado y particiones; cierres y provenance;
geometría y contraste independiente. Se compila y ejecuta exclusivamente con
`scripts/test_safe.py`, conservando 1 GiB total, sin swap, un job y un hilo.

Los [resultados de verificación](legal-geometry/recalcitrance/verification.md)
registran tiempos, memoria y logs; el
[registro de hashes](legal-geometry/recalcitrance/input-sha256.txt) fija fuentes y
entradas. No se ejecutó la suite completa, no se modificó el motor, no se hizo
merge y se conservaron `main` y `v1`.
