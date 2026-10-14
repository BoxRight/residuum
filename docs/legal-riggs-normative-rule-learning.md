# Aprendizaje experimental de reglas discriminantes sobre Riggs

En `experiment-legal-order-geometry` se generan **64 cláusulas parametrizadas**
y se evalúan mediante el motor existente. La observación histórica conserva
**16 candidatas**, con cuatro comportamientos distintos en los seis expedientes
originales y seis comportamientos distintos sobre los 64 subconjuntos de los
seeds históricos. Ninguna se aprueba como norma jurídica.

La teoría sucesoria original conserva todos sus resultados y justificaciones.
Las teorías hipotéticas producen un **impedimento propuesto**. La adición Horn
no convierte ese impedimento en una exclusión efectiva: la atribución, el activo,
la titularidad y la capacidad literales continúan derivados. Esta frontera se
observa explícitamente; no se añade una prioridad ni una retractación.

## 1. Referencia, experiencia recalcitrante y fuente

T es [RiggsSuccessionReference](../examples/experiments/legal_geometry/riggs_succession_reference.res):
las 27 reglas inalteradas de LegalGeneral y sus cuatro reglas sucesorias. Para
cada expediente se calcula independientemente `close_program(S,T)`. No se
modifica LegalGeneral ni se usan datos de expedientes de 12hectareas.

La referencia literal tiene como frontera, en los seis seeds históricos:

```text
operativeDisposition(will, testator, beneficiary, object, authority, effective)
deathRecorded(death, testator, deathAuthority, deathEffective)
survivorRecorded(beneficiary, testator)
```

Se conservan los índices `@ dossierTime`, la fecha del instrumento `will1880`,
la fecha de muerte `death1882` y las autoridades originales de cada hecho.
El cierre histórico conserva 12 eventos y seis justificaciones.

La observación externa viene del fallo: la mayoría declara ineficaz la
disposición para transmitir el título a Elmer y lo excluye del interés
sucesorio considerado. La opinión también reconoce el resultado de la lectura
literal y describe el propósito adquisitivo del homicidio.
[Riggs v. Palmer, 115 NY 506, pp. 508–509 y 514–515](https://www.nycourts.gov/reporter/archives/riggs_palmer.htm).

La [tabla de observación](../examples/experiments/legal_geometry/riggs_rule_observations.tsv)
declara la traducción experimental: ese resultado exige una prueba del
impedimento para el beneficiario, instrumento y objeto del expediente. Es una
**condición necesaria de ajuste al resultado**, no una reproducción completa
del remedio judicial ni de toda la distribución de la herencia.

La recalcitrancia se representa por la pareja:

\[
(C_T(S_{Riggs}),\;y_{histórico}=\text{impedir esa adquisición}).
\]

La observación no se agrega a Known como seed y no se infiere de T. Los
contrafactuales conservan sus datos estipulados y no reciben etiquetas históricas.

## 2. Reutilización y adaptación

Se inspeccionaron los contratos de
[funciones justificativas](legal-recalcitrance-justificatory-functions.md),
[conversación](legal-conversational-justification-learning.md),
[referencia fija](legal-riggs-fixed-reference-experiment.md),
[aprendizaje argumentativo](legal-riggs-conversational-argument-learning.md),
fronteras, revisiones y dependencias geométricas.

Se reutilizan el filtrado del espacio de versiones, los kernels de particiones,
`meet` y `finer` del harness conversacional, el cierre real, la auditoría de
DerivationRecord, la saturación independiente de cláusulas ground y el método
de antiintercambio/extremos. La referencia y el evaluador de premisas de Earl/Gray
se reutilizan sin cambios.

La adaptación es una gramática **relacional de reglas parametrizadas**. No se
trasladan los 168 clasificadores O/R/A/P, sus etiquetas de compliance ni la
operación anterior que sustituía productores de compliance. Tampoco se limita
el espacio a tres lecturas argumentativas.

## 3. Gramática finita de candidatas

El [catálogo Formal generado](../examples/experiments/legal_geometry/riggs_candidate_rules.res)
importa la referencia y declara sólo un verbo experimental nuevo:

```text
derived verb hypotheticalSuccessionImpediment(
    will : Fact, testator : Person, beneficiary : Person,
    object : Thing, authority : Authority, effective : PropositionTime
)
```

Su significado es «esta candidata propone impedir la adquisición de este
interés bajo este instrumento». La autoridad y vigencia del argumento identifican
el instrumento de origen; no convierten la propuesta en una sentencia judicial.
El módulo de la regla conserva la procedencia experimental.

Todas las candidatas tienen el esquema:

\[
r_M:\quad E\ \&\ \bigwedge_{i\in M} a_i\ \le\ Impediment\ @t.
\]

E es `testamentaryEntitlement(will,testator,beneficiary,object,authority,effective) @ t`.
M recorre los 64 subconjuntos de seis átomos posibles:

| Código | Átomo tipado y bindings relevantes |
|---|---|
| K | `intentionalKilling(act,beneficiary,testator,actAuthority,actEffective) @ t` |
| L | `acquisitionLinkedToWrong(will,act,death,beneficiary,object,authority,effective) @ t` |
| F | `acquisitionPurposeRecorded(act,will,beneficiary) @ t` |
| B | `patrimonialAsset(beneficiary,object,will,authority,effective) @ t` |
| notK | EvidentialNot de exactamente K |
| notF | EvidentialNot de exactamente F |

`intentionalKilling` conserva su contrato previo de homicidio intencional ilícito.
L conserva el testigo de muerte y las reglas originales que enlazan la adquisición
con ese homicidio. B representa el beneficio patrimonial **atribuido por T**,
no una constatación independiente de cobro o posesión material.

Los parámetros se generan a partir de los identificadores utilizados, y el
parser/elaborador existente comprueba sus tipos. El test reconstruye el texto
completo generado y lo compara exactamente con el catálogo. Los bindings
comparten beneficiario/testador, instrumento/objeto y el mismo acto entre K, L
y F. Los parámetros de acto y muerte son locales al match cuando existen.

L no contiene un campo testador en su firma original: cuando K no aparece en la
candidata, el enlace con él pasa por el instrumento y la regla de referencia.
El dominio estudiado contiene un testador por instrumento. La equivalencia
Seeded indicada abajo no se extiende a expedientes que atribuyan el mismo
instrumento a testadores diferentes ni a L aportado directamente sin sus testigos.

Las restricciones previas son: cabeza range-restricted por E, tipos existentes,
bindings compartidos, tiempos coincidentes y ninguna condición por ausencia.
Son restricciones expresivas del experimento. No se declara que todos los
antecedentes sean fundamentos jurídicamente aceptables.

La candidata E sola es un control deliberadamente amplio. Las candidatas con
notK/notF son controles de filtrado con evidencia negativa explícita. Incluirlas
en el catálogo no afirma que deban fundar una exclusión. Incluso las conjunciones
K/notK son representables: la oposición no produce explosión ni eliminación.

**Importante:** el módulo del catálogo se elabora para obtener sus reglas; nunca
se ejecutan las 64 simultáneamente. Para cada evaluación se incorpora exactamente
una candidata a una copia privada de T.

## 4. Operación hipotética y consecuencia efectiva

Aquí se fija una sola operación:

\[
T\oplus r_M=T\cup\{\text{firma experimental de Impediment},r_M\}.
\]

Es una **adición de consecuencia**, sin excepción, sustitución, restricción ni
cambio de defaults. Para cada candidata y expediente se calculan por separado
el cierre original y el hipotético.

| Nivel observado | Resultado |
|---|---|
| Atribución literal y cadena patrimonial de T | Conservadas |
| Impedimento propuesto por la candidata | Derivado sólo si su antecedente tiene match |
| Decisión histórica | Fuente externa que requiere impedir la adquisición |
| Decisión efectiva del ejecutable hipotético | No resuelta por esta adición Horn |

El impedimento es positivo y distinto de `~testamentaryEntitlement`. Su significado
específico queda declarado, pero el motor no le concede poder de borrar el título.
El experimento no ejecuta State ni presenta la coexistencia como un remedio judicial.
Aprender una condición discriminante y definir cómo desplazar una consecuencia
jurídica son operaciones distintas.

## 5. Resultado histórico y generalización

La observación positiva conserva exactamente `candidateImpediment00` a `15`:
las 16 conjunciones de E con subconjuntos de K/L/F/B. Las otras 48 requieren al
menos una negación evidencial que no está soportada por el expediente histórico.
Ese rechazo significa insuficiencia de la regla en la abstracción histórica
delimitada; no se deriva la negación de sus premisas por ausencia.

Las 16 supervivientes se agrupan en seis fronteras Seeded distintas:

| Familia representativa | Soporte mínimo, además de disposición+muerte+supervivencia |
|---|---|
| E | Ninguno |
| E & K | Homicidio del beneficiario contra ese testador |
| E & L | Homicidio y vínculo `deathCausedBy` con esa muerte |
| E & F | Finalidad sobre ese acto, instrumento y beneficiario |
| E & K & F | Homicidio y finalidad con el mismo acto |
| E & L & F | Homicidio, vínculo causal y finalidad |

B es redundante respecto de estas entradas: E ya produce el activo. K es
redundante junto a L sobre entradas exclusivamente Seeded porque la regla
original que produce L necesita K. Esa redundancia no se generaliza a teorías
que aportan directamente L como un hecho Derived.

Los seis expedientes existentes inducen cuatro clases de predicción:

| Clase | Candidatas | Histórico | Sin homicidio/tercero | Sin finalidad | Vínculo pendiente |
|---|---|---|---|---|---|
| E, con B opcional | 0, 8 | Sí | Sí | Sí | Sí |
| E & K, con B opcional | 1, 9 | Sí | Sin prueba | Sí | Sí |
| E & L, K/B opcionales | 2, 3, 10, 11 | Sí | Sin prueba | Sí | Sin prueba |
| E & F, K/L/B opcionales | 4–7, 12–15 | Sí | Sin prueba | Sin prueba | Sin prueba |

Sin supervivencia establecida ninguna produce el impedimento, porque falta E.
«Sin prueba» no es `~Impediment`, ni una sentencia que permita adquirir.

Además se verifica generalización relacional de E & K & L & F: un renombramiento
simultáneo de personas, instrumento, objeto, acto y fechas mantiene los tipos y
produce el nuevo impedimento con los nuevos bindings. Dos expedientes concurrentes
conservan sus dos consecuencias independientes. Dos actos distintos que soportan
la misma cabeza conservan dos justificaciones, no una justificación deduplicada.
Invertir el orden de los seeds conserva los eventos y el multiconjunto de pruebas.
Estos son controles estructurales del harness, no decisiones jurídicas nuevas.

## 6. Particiones, discriminantes y siguiente conversación

Se calcula la partición por predicciones y se reutiliza el orden «más fina ≤ más
gruesa». Incorporar un expediente concatena firmas, y el kernel obtenido coincide
con el meet de la partición anterior y la nueva.

El caso `killingWithoutAcquisitionPurpose` divide las 16 supervivientes en 8/8,
frente a 2/14 para ausencia explícita de homicidio y homicidio de tercero, 4/12
para vínculo pendiente y 0/16 para supervivencia pendiente. Es la siguiente
pregunta minimax única entre estos cinco expedientes:

> Para el beneficiario que mató ilícitamente al testador y causó la apertura de
> esta adquisición, pero cuya finalidad adquisitiva está expresamente negada,
> ¿debería proponerse el impedimento sobre este instrumento e interés?

Su adecuación jurídica requiere deliberación externa. La compatibilidad de las
premisas de los argumentos originales se conserva como información separada:
en ese expediente la línea causal es Compatible y la lectura con finalidad es
Incompatible; donde falta el vínculo, ambas son Indeterminate.

Las [tres respuestas hipotéticas](../examples/experiments/legal_geometry/riggs_rule_followup_valuations.tsv)
se prueban en ramas independientes:

- Requerir impedimento conserva 8 candidatas sin F.
- No requerirlo conserva las otras 8 con F.
- Suspender la valoración conserva las 16.

Estas respuestas evalúan la cobertura pretendida de las reglas sobre el expediente
delimitado. No establecen ni completan hechos pendientes. No se interpretan como
refutaciones de argumentos por ausencia de evidencia ni como fallos históricos.
La partición se refina por meet para una respuesta determinada; la suspensión
no agrega una etiqueta que elimine candidatas.

Se calculan también todos los casos mínimos por inclusión que distinguen parejas
supervivientes sobre los 64 subconjuntos Seeded. Hay 124 registros de pareja/caso;
su catálogo conjunto tiene cinco soportes distintos, máscaras 7, 15, 31, 39 y 47:
ruta literal, ruta+homicidio, ruta+homicidio+causa, ruta+finalidad y
ruta+homicidio+finalidad. Son expedientes tipados parciales, no cinco etiquetas
jurídicas ya obtenidas. Los nuevos casos separan seis clases que los seis
expedientes completos anteriores sólo separaban en cuatro.

## 7. Geometría de las teorías hipotéticas

Se declara E' explícitamente: los 12 eventos del cierre histórico más el evento
ground del impedimento. Tiene 13 puntos y 8.192 subconjuntos. Para comparar
con la referencia sobre el mismo carrier se añade allí **sólo la firma pasiva**
del nuevo verbo; no se añade ninguna regla productora a T.

Se extraen las cláusulas ground reales mediante el instanciador del motor y se
calcula una saturación independiente de bitsets. Las relaciones completas
siguen gobernando la extracción; los bitsets sólo codifican eventos ya elaborados.

**RESULTADO FINITO:** referencia y 64 candidatas, 532.480 estados calculados.
Se verifican extensividad, monotonía por inclusiones inmediatas, idempotencia,
cierre vacío, antiintercambio en todos los estados cerrados y generación por el
conjunto extremo. Las proyecciones sobre los 12 eventos originales coinciden
en todos esos estados. El grafo ground es acíclico en las 65 teorías.

La referencia tiene 1.984 estados cerrados en E'; las 16 supervivientes tienen
entre 1.776 y 1.968. Las 48 rechazadas no se activan en este subuniverso positivo:
sus negaciones sí se examinan en los expedientes, pero no están en E'. Su resultado
geométrico local no certifica todas las entradas con oposición evidencial.

El modelo independiente se contrasta con `close_program` en:

- Los 64 subconjuntos históricos Seeded de cada superviviente y la referencia.
- Dos controles Seeded de cada rechazada.
- Los 13 singletons y E' completo de las 65 teorías, incluyendo puntos Derived.

Son 2.094 comparaciones modelo/evaluador. Los estados con puntos Derived son
teorías finitas de prueba, no seeds externos admitidos. Las fronteras mínimas se
obtienen en el dominio Seeded; los extremos reconstruyen estados cerrados completos.

**ARGUMENTO CONDICIONAL:** en una clausura Horn finita sin cuerpos vacíos y con
dependencias acíclicas, si añadir y permite obtener x fuera de F cerrado, hay
un camino de dependencia y→x. Obtener también y añadiendo x exigiría el camino
contrario, formando un ciclo. De ahí antiintercambio bajo esas hipótesis. En una
clausura finita con antiintercambio, los extremos constituyen la base única;
aquí se comprueban además su generación e indispensabilidad mediante tablas.
Esto no prueba esas propiedades para toda la ontología con todos los tiempos.

**Contraejemplo a «añadir reglas siempre refina clases de clausura»:** tome
F=C_T(S histórico), sin impedimento, y F∪{Impediment}. Bajo la referencia pasiva
son dos clausuras distintas. Bajo cada superviviente tienen la misma clausura,
porque el impedimento ya es derivable desde F. Hay fusión de clases en E',
aunque se preserve antiintercambio y toda consecuencia original. Los testigos
están en `closure-class-mergers.tsv`. Este contraejemplo permite diferenciar
particiones de expedientes, particiones de hipótesis y clases de clausura.

## 8. Provenance, certificación y límites

Las 384 comparaciones candidata/expediente conservan al proyectar sobre T:
eventos completos, tipos elaborados, polaridad, índices y fechas, antecedentes
ordenados, sustituciones y multiplicidad de justificaciones. Se recalcula T
independientemente en cada comparación. Todas sus declaraciones y orígenes
son idénticos. Sólo `derivedAt` se ignora entre ejecuciones.

Las pruebas nuevas identifican candidata, módulo RiggsCandidateRules, testigos
y bindings; cada registro se reconstruye con la regla real. Las pruebas de la
ruta original mantienen sus módulos de origen LegalGeneral y
RiggsSuccessionReference. Earl y Gray y sus fuentes se conservan como contraste
en los artefactos; no son las únicas formas de antecedente del catálogo.

Cuatro tests nuevos y dos regresiones individuales pasan mediante
`scripts/test_safe.py`, con 1 GiB, swap 0, un trabajo y un hilo. Los comandos,
incidencias de compilación y consumos están en
[verification.md](legal-geometry/riggs-rule-learning/verification.md).
El verificador está en
[legal_geometry_riggs_rule_learning_tests.rs](../src/bin/residuum/cli/legal_geometry_riggs_rule_learning_tests.rs).
El [directorio de resultados](legal-geometry/riggs-rule-learning/) conserva el
catálogo, cierres comparados, pruebas completas, versiones, particiones,
discriminantes, universo, dependencias, fronteras y bases extremas.

**OBSERVACIÓN EXPERIMENTAL:** se construyen y filtran reglas nuevas, y sus
consecuencias se verifican en otros expedientes y bindings. **ALCANCE FINITO:**
gramática de seis átomos opcionales, un interés sucesorio delimitado, seis
expedientes y dos dominios de enumeración. **PENDIENTE:** generalización jurídica
externa, explicación suficiente, legitimidad institucional y mecanismo de
exclusión efectiva. El ajuste histórico no identifica una candidata única.

No se modificaron motor, ontología, fixtures originales, main o v1; no hubo merge.
