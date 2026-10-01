# Puente entre conocimiento, valoración y satisfacción

Éste es un experimento finito de tests, no una nueva semántica adoptada por M2.
Parser, `TypedAST`, búsqueda abductiva, cierre y ejecución permanecen sin cambios.
Se corrige primero la identificación anterior entre unidad y conocimiento vacío.
El código está en
[satisfaction_bridge.rs](../src/algebra/tests/evidence_pairs/satisfaction_bridge.rs).

La [extensión por modelos de teorías](four-model-entailment.md) enumera todas las
interpretaciones y compara entailment universal con Horn, incluidos ciclos y
residuales. Esta página conserva los resultados de la comparación por soporte;
la extensión evita identificarlos con completitud para desigualdades de verdad.

El [experimento posterior de teoría/modelos](theory-model-bridge.md) comprueba
preservación de clases de modelos al saturar hechos, compara sus núcleos mínimos
con Horn y conserva la clase completa para interpretar fórmulas residuales.
No reemplaza el closure operacional por el entailment FOUR.

La API actual ya separa estos objetos: `query_program_residual` recibe evidencia
como `&[Event]`, no como `Antecedent`. `query_residual` sobre una regla también
recibe `known: &[Event]`. El resultado lógico puede contener `Unit`; no es una
representación del argumento de conocimiento vacío. No hace falta cambiar esas
firmas para realizar este experimento.

## Tres papeles distintos

| Papel | Dominio | Modelo FOUR de fusión | Modelo FOUR de meet |
| --- | --- | --- | --- |
| Conocimiento vacío | `⊥_K=∅` en K | U por cada átomo | U por cada átomo |
| Unidad del producto lógico | `I=1_L` en L | B | T |
| Bottom lógico | `0_L`, si existe | F | F |

El producto de cero factores es `I`, incluso si hay conocimiento disponible.
El testigo neutro del matcher `Unit` y un requisito final `I` son usos válidos de
esa unidad: ninguno implica `Known=∅`. Tampoco valorar `I` como B introduce
evidencia positiva y negativa de todos los eventos.

## Valoración indexada por conocimiento

Fijamos un universo finito de átomos ground para el experimento. La identidad de
cada átomo incluye verbo, argumentos y tiempo proposicional. Los dos signos de
ese mismo átomo determinan sus componentes; la metadata operacional no interviene.

```text
V_K(P) = (1[P ∈ K], 1[~P ∈ K])
V_K(~P) = EvidentialNot(V_K(P))
```

Así `V_∅(P)=U` para todo átomo P. El objeto completo es una valoración en
`FOUR^Atoms`, no un único U. Añadir evidencia induce aumento puntual en el orden
de conocimiento. No convierte automáticamente ese aumento en el orden de verdad.

La extensión a fórmulas interpreta cada operador en un modelo elegido:

```text
V_K(I) = unidad del modelo
V_K(A ⊗ B) = V_K(A) ⊗ V_K(B)
V_K(A ∨ B) = V_K(A) ∨_t V_K(B)
V_K(A ∧ B) = V_K(A) ∧_t V_K(B)
V_K(B ⊸ C) = V_K(B) ⊸ V_K(C)
```

Las leyes residuadas ya comprobadas valen en estos carriers. Todavía queda por
establecer qué interpretación de fórmulas y de la teoría representa fielmente M2.

## Candidata de satisfacción

La candidata comprobada en el fragmento ground es soporte positivo:

```text
K ⊨⁺ φ  ⇔  π_p(V_K(φ)) = 1
```

Designa T y B. Los átomos ausentes y sus negaciones no quedan satisfechos; un
átomo con ambas evidencias y su negación sí. La unidad está satisfecha en ambos
modelos y el bottom de verdad no. Esto es una política candidata, no una decisión
universal de consistencia ni una prueba de realización completa de L.

En fusión, el soporte positivo de `A ⊗ B` es `p_A ∧ p_B`, aunque el par completo
no sea monótono en conocimiento. Eso puede corresponder a existencia de testigos
conjuntivos compatibles en el fragmento ground. No basta para justificar todas
las fórmulas, variables, multiplicidades o el residual.

Extender EvidentialNot a fórmulas compuestas requiere revisión propia: intercambiar
componentes puede exponer el soporte negativo de una operación que no es monótona
en conocimiento. El experimento no amplía la negación atómica del runtime.

## Tres pruebas que no deben confundirse

1. Desigualdad puntual: `V_K(A) ≤_t V_K(C)`.
2. Consecuencia operacional: si K satisface A, `Close_R(K)` satisface C.
3. Entailment entre fórmulas: una desigualdad válida en todas las valoraciones
   admitidas por la interpretación de la teoría.

Por ejemplo, A=T y C=T satisfacen la desigualdad puntual. Añadir evidencia
negativa sobre C lo cambia a B y rompe `T ≤_t B`; su soporte positivo permanece.
Por eso no se interpreta directamente una regla Horn como esa desigualdad en
cada estado intermedio del closure.

El objetivo pendiente es precisar qué valoraciones admite una teoría y cómo sus
reglas inducen el operador monotónico sobre K. No se da por establecido que
satisfacción positiva resuelva ese problema ni que preserve toda la adjunción.

## Resultados de los siete tests

| Conocimiento | `V_K(P)` | `V_K(~P)` | Soporte de P | Soporte de ~P |
| --- | --- | --- | --- | --- |
| `∅` | U | U | No | No |
| `{P}` | T | F | Sí | No |
| `{~P}` | F | T | No | Sí |
| `{P,~P}` | B | B | Sí | Sí |

En ambos modelos, `I` permanece satisfecho sin insertar eventos y F no tiene
soporte positivo. Las lecturas no convierten ausencia en negación.

El experimento usa tiempo fijo `tau`, eventos ground y un máximo de cuatro
átomos. No genera tiempos ni introduce reglas cíclicas. Comprueba:

1. Los cuatro estados de conocimiento y sus negaciones, sin mutar `Known`.
2. Evaluación de join, meet y tensor, y las dos direcciones de residuación en
   las 64 valoraciones de A, B y C, para ambos modelos.
3. `K ⊆ K'` si y sólo si sus valoraciones atómicas aumentan puntualmente en
   `≤_K`, recorriendo todos los pares de esas 64 valoraciones.
4. Un contraejemplo que impide extender esa monotonía a toda fórmula de fusión:
   pasar A de F a B, con B=T, cambia `A ⊗ B` de F a T. Negar ese producto
   pierde soporte positivo. La negación compuesta no se incorpora al runtime.
5. Coincidencia entre soporte positivo y existencia de matches para 56 formas
   de antecedente ground: unidad, seis átomos con signo y sus productos. Se
   recorre cada forma bajo las 64 valoraciones, en ambos modelos.
6. Coincidencia entre un cierre independiente por valoración positiva y el
   `close` real para `A & B <= C; C <= D`: 256 contextos, ocho elecciones de
   polaridad de A, B y C, y ambos modelos. También extensividad, idempotencia,
   monotonía por inclusión y una regla ground `I <= C` desde conocimiento vacío.
7. Un cierre Horn estable que incumple su desigualdad puntual de verdad.

La última comprobación parte de `K={A,B}`, deriva C y luego añade `~C`. El
conjunto `{A,B,C,~C}` sigue siendo cerrado. Sus valores son A=T, B=T y C=B:

```text
V_K(A ⊗ B) = T
T no ≤_T B
K ⊨⁺ A ⊗ B  y  K ⊨⁺ C
```

Esto ocurre en ambos productos. La desigualdad puntual exige también
`n_C ≤ n_(A⊗B)`; el operador Horn conserva soporte del consecuente sin imponer
esa restricción sobre evidencia opuesta. No se la añade al motor.

Hay otra diferencia de interpretación: en ese cierre, la fórmula
`(A ⊗ B) ⊸ C` pierde soporte positivo en el modelo de fusión, mientras que lo
conserva en el de meet. Las leyes de adjunción siguen válidas en ambos carriers;
soporte positivo de una implicación y cierre Horn no se identifican universalmente.

Por tanto, el puente funciona para el fragmento de antecedentes y consecuencias
con soporte explícito comprobado aquí. No certifica una realización fiel de todo
L, entailment universal de la teoría ni satisfacción general de residuales.
Antes de una migración o incrementalidad queda decidir cómo la teoría restringe
valoraciones y qué relación exacta debe tener con la consecuencia operacional.

Los siete tests pasan individualmente mediante `scripts/test_safe.py`, con 1 GiB
total, sin swap y timeout. El test de cierre tarda aproximadamente 0,16 s y usa
un pico RSS de 15,27 MiB; la compilación usa 581 MiB. No se ejecuta la suite completa.
