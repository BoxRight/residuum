# Horn frente a entailment por modelos FOUR

Experimento exclusivamente de tests y modelos finitos. No cambia `TypedAST`,
`Closure`, matcher, runtime, lenguaje ni búsqueda abductiva. El código está en
[model_entailment.rs](../src/algebra/tests/evidence_pairs/satisfaction_bridge/model_entailment.rs).

El resultado es una comparación semántica, no la afirmación de que la semántica
de M2 deba adoptar uno de estos modelos.

## Interpretaciones y satisfacción

Para un universo finito E de átomos ground, una interpretación es una función
`M:E → FOUR`. Se enumeran las `4^|E|` funciones, sin filtrar mediante Horn ni
seleccionar sólo la valoración del conocimiento real. Los dos signos de un
átomo se representan mediante sus componentes p y n.

La representación concreta de cada interpretación es un conjunto de soportes
con signo. Tiene la misma forma finita que un `Known`, pero recorre todos los
valores posibles; no se identifica la semántica con el único closure calculado.

```text
V_M(P) = M(P)
V_M(~φ) = swap(V_M(φ))
V_M(I) = I del modelo
V_M(φ & ψ) = V_M(φ) ⊗ V_M(ψ)
V_M(φ -o ψ) = V_M(φ) ⊸ V_M(ψ)
```

Join y meet se evalúan con el orden de verdad. Los dos productos y sus residuales
son los definidos en [los modelos de pares de evidencia](evidence-pair-models.md).
Negación compuesta y fórmulas residuales son objetos de tests, no nuevas formas
de regla ejecutables por el matcher.

La satisfacción principal es el filtro de la unidad:

```text
M ⊨ φ       ⇔ I ≤_T V_M(φ)
M ⊨ A ≤ C   ⇔ V_M(A) ≤_T V_M(C)
```

En fusión, I=B y los valores designados son `{T,B}`. En meet, I=T y sólo T es
designado. Los hechos `P` y `~P` usan la misma condición de satisfacción que las
demás fórmulas: no se mezcla un criterio de hechos con otro de consultas.

Para `Γ=(Facts,Rules)`:

```text
M ⊨ Γ       ⇔ (∀F∈Facts, M ⊨ F) y (∀R∈Rules, M ⊨ R)
Mod(Γ)      = {M:E→FOUR | M ⊨ Γ}
Γ ⊨ φ       ⇔ ∀M∈Mod(Γ), M ⊨ φ
Γ ⊨ A ≤ C   ⇔ ∀M∈Mod(Γ), V_M(A) ≤_T V_M(C)
```

El entailment sobre una clase vacía es verdadero por cuantificación universal.
Se cuentan por separado esas teorías; no se presenta vacuidad como evidencia.
Como premisa, I deja intacta la clase de modelos. También se comprueba la
equivalencia de modelos de `{P & ~P}` y `{P,~P}` en estos dos modelos.

## Controles de interpretación

Se comparan tres configuraciones adicionales:

* Meet con hechos y consultas satisfechos por `p=1`, pero reglas interpretadas
  como desigualdades completas de verdad. Es una definición explícita de modelos,
  aunque no identifica correctamente la regla con su fórmula residual.
* Fusión con soporte positivo y reglas `!p_antecedent || p_consequent`.
* Meet con esa misma política de soporte.

El `!` de esa expresión es complemento de un bit en los tests; no es negación
por ausencia del lenguaje. Estos dos últimos controles expresan la semántica
de soporte Horn, no la lectura de la regla como desigualdad residuada.

## Cálculo independiente de Horn

```text
Γ ⊢_Horn C ⇔ C ∈ close(Facts, Rules).known
```

Se invoca el `close` existente sobre reglas ground y un reloj independiente.
La enumeración de modelos no llama al matcher, al closure ni al oráculo por
soporte del experimento anterior. La comparación de derivabilidad se limita a
átomos con ambos signos; I se comprueba como unidad lógica, no como evento.
No se declara incompletitud por no ejecutar fórmulas que Horn aún no representa.

Los átomos de las teorías son `Derived`, de modo que ambos signos son admisibles
como cabezas. Los hechos son premisas lógicas disponibles; esto no introduce
declaraciones `seeded` ni autoriza nuevas formas de seed externa en M2.

## Enumeración exhaustiva de programas pequeños

Para dos átomos P y Q, se recorren los 16 contextos de hechos con signo y los
16 modelos de cada teoría. Las cabezas son los cuatro átomos con signo.

Hay 60 reglas candidatas: cinco cuerpos unidad/unarios y diez productos de dos
literales, incluida repetición; cada cuerpo se combina con las cuatro cabezas.
Se recorren estos 251 programas: vacío, cada una de las 60 reglas y todos los
190 pares distintos de las 20 reglas unidad/unarias. Esto incluye ciclos,
autociclos, antecedentes opuestos y conclusiones inconsistentes.

Son 4.016 teorías y 20.080 consultas por configuración: cuatro literales e I.
Es exhaustividad sobre esta familia declarada, no sobre todos los programas
ground ni todos los conjuntos de reglas binarias.

| Semántica | Fallos soundness | Fallos completeness | De ellos con modelos | Teorías sin modelos | Closure no es modelo |
| --- | ---: | ---: | ---: | ---: | ---: |
| Fusión, filtro de I, regla de verdad | 0 | 680 | 680 | 0 | 640 |
| Meet, filtro de I, regla de verdad | 0 | 2.308 | 120 | 2.427 | 2.551 |
| Meet, soporte positivo, regla de verdad | 0 | 1.144 | 536 | 720 | 1.232 |
| Fusión, soporte positivo, regla de soporte | 0 | 0 | 0 | 0 | 0 |
| Meet, soporte positivo, regla de soporte | 0 | 0 | 0 | 0 | 0 |

Los fallos de completitud cuentan consultas; las últimas dos columnas cuentan
teorías. Que el closure no sea un modelo no equivale a un fallo de soundness:
soundness exige que sus conclusiones estén satisfechas en todos los modelos,
no que su valoración mínima sea ella misma uno de ellos.

## Familias con tensor, cadenas y contradicción

Cada familia recorre todos sus contextos y todos los modelos. La cadena usa cuatro
átomos, por tanto 256 contextos y 256 interpretaciones posibles por teoría.

| Programa | Consultas por semántica | Completitud: fusión/I/verdad | Completitud: meet/I/verdad | Completitud: meet/positivo/verdad |
| --- | ---: | ---: | ---: | ---: |
| `A & B <= C` | 448 | 16 | 82 | 0 |
| `A & B <= C; C <= D` | 2.304 | 160 | 568 | 64 |
| `P <= Q; Q <= P` | 80 | 8 | 8 | 8 |
| `P <= Q; P <= ~Q` | 80 | 7 | 9 | 7 |

Todos tienen cero fallos de soundness. Los dos controles por soporte tienen cero
fallos de completitud en todas estas familias. El ciclo sin hechos no genera
evidencia. Los fallos vacuos de meet/I siguen incluidos en su total.

## Contraejemplos y clasificación

**Horn completeness failure, sin vacuidad.**

```text
Γ = {~Q, P <= Q}
Γ ⊨ ~P
Γ no ⊢_Horn ~P
```

La desigualdad exige `n_Q ≤ n_P`; el hecho `~Q` fija `n_Q=1`, por lo que todos
los modelos tienen `n_P=1`. Horn sólo aplica la dirección forward y no deriva
`~P`. Hay tres modelos en fusión/I y meet/positivo, y uno en meet/I.
Es mínimo por eliminación de premisas: quitar el hecho o la regla destruye ese
entailment. Además, la enumeración encuentra dos premisas como el menor tamaño
de contraejemplo no vacuo en su familia.

**Horn completeness failure por vacuidad.** En meet/I, `{P,~P}` no tiene modelos:
P exigiría T y `~P` exigiría que P fuese F. Semánticamente se sigue Q por vacuidad,
pero Horn no lo deriva. En fusión/I, la valoración P=B sí es modelo y Q no se
sigue. No se atribuye explosión a EvidentialNot del motor.

**Diferencia esperada entre soporte y entailment.** Para
`Γ={A,B,~C,A & B <= C}`, Horn obtiene `{A,B,C,~C}`. Su valoración A=T, B=T, C=B
incumple la desigualdad. Sin embargo, en fusión/I y meet/positivo esa teoría
tiene modelos y todos satisfacen C. El C producido es sound; lo que falla es
identificar la valoración del closure con un modelo de verdad. No se clasifica
este caso como unsoundness.

**Definiciones incompatibles para la regla y el residual.** Meet con designación
`p=1` permite que B satisfaga una fórmula, aunque `I=T` no esté debajo de B.
Esa política deja de identificar satisfacción de la fórmula residual con la
desigualdad. Si el contrato exige una sola lectura para ambas, esa interpretación
de la regla está mal definida respecto del contrato. La tabla algebraica del
modelo no está mal: la incompatibilidad está en la designación elegida.

**Horn soundness failure.** No se encontró ninguno en estas familias. El harness
cuenta y conserva un contramodelo si aparece alguno; no lo reclasifica como una
diferencia esperada.

## El residual dentro del mismo entailment

Con el filtro de I, para cada interpretación:

```text
M ⊨ (A ⊗ B) ⊸ C
⇔ I ≤_T V_M((A ⊗ B) ⊸ C)
⇔ I ⊗ V_M(A ⊗ B) ≤_T V_M(C)
⇔ V_M(A ⊗ B) ≤_T V_M(C)
```

Por cuantificación sobre la misma `Mod(Γ)`, queda:

```text
Γ ⊨ (A ⊗ B) ⊸ C  ⇔  Γ ⊨ A ⊗ B ≤ C
Γ ⊨ A ⊗ B ≤ C    ⇔  Γ ⊨ A ≤ B ⊸ C
```

Se comprueban 128 teorías por configuración: los 64 contextos de A, B y C,
con y sin `A & B <= C`. La segunda equivalencia conserva cero discrepancias
en todas las configuraciones, porque compara desigualdades mediante adjunción.
La primera depende también de la designación:

| Configuración | Discrepancias fórmula residual / desigualdad |
| --- | ---: |
| Fusión/I/verdad | 0 |
| Meet/I/verdad | 0 |
| Meet/positivo/verdad | 8 |
| Fusión/positivo/soporte | 0 |
| Meet/positivo/soporte | 24 |

Un test también compara, interpretación por interpretación, satisfacción de la
regla con satisfacción de su fórmula residual, recorriendo las 64 valoraciones:

| Configuración | Discrepancias regla / fórmula residual |
| --- | ---: |
| Fusión/I/verdad | 0 |
| Meet/I/verdad | 0 |
| Meet/positivo/verdad | 7 |
| Fusión/positivo/soporte | 11 |
| Meet/positivo/soporte | 0 |

Coincidencia fórmula/desigualdad no basta si las reglas de la teoría se
interpretaron mediante otra condición. Fusión/soporte coincide con Horn y con
la primera equivalencia de entailment para fórmulas e inequaciones, pero su
regla de soporte difiere de la fórmula residual en 11 valoraciones. Meet/soporte
identifica regla de soporte y fórmula residual, pero no fórmula y desigualdad.
Ninguno de esos controles establece la identificación triple completa.

El contraejemplo concreto para meet/positivo es
`Γ={A,B,C,~C}`, sin reglas: todos sus modelos satisfacen `(A ⊗ B) ⊸ C` por el
componente positivo. Pero el modelo A=T, B=T, C=B incumple `A ⊗ B ≤ C`.

## Alcance y verificación

La semántica residuada con fusión y filtro de I conserva evidencia opuesta, da
la misma lectura a regla y residual y es sound para las consultas Horn de esta
familia. Es incompleta para Horn: aparecen consecuencias negativas adicionales.
El control por soporte coincide con Horn, pero no permite identificar sin más
sus reglas con desigualdades residuadas de verdad.

Los cinco tests del experimento se ejecutan individualmente mediante
`scripts/test_safe.py`, con 1 GiB total, swap deshabilitado y timeout. Cada test
usa alrededor de 15 MiB RSS; las dos enumeraciones principales tardan menos de
un segundo cada una. No se ejecuta la suite completa ni se amplía el lenguaje.
