# Refinamiento Horn frente al orden FOUR de programas

El experimento está en
[algebra_comparison.rs](../src/experiments/refinement_tests/algebra_comparison.rs).
Reutiliza el checker finito, `close` y el modelo FOUR con fusión existente.
Los cambios de visibilidad del modelo son internos a `cfg(test)`. No se modifica
TypedAST, matcher, runtime, residual, defaults ni la gramática.

**Resultado:** coinciden las tablas de los tres programas directos, pero las dos
relaciones no son iguales bajo la interpretación de programas definida aquí.
Hay contraejemplos con producto y también con una spec Horn ordinaria encadenada.
Esto rechaza esta identificación concreta, no cualquier semántica algebraica de
contratos.

## Interpretación explícita

Se conserva Ω: dos propietarios, dos cosas y dos tiempos, ocho premisas owns.
Para cada instancia se definen tres átomos:

```text
O = owns(owner, object) @ t
P = protected(owner, object) @ after(t)
Q = insured(owner, object) @ after(t)
```

Las reglas se parsean, elaboran e instancian normalmente antes de proyectar sus
eventos ground en O, P, Q. Ambos evaluadores reciben las mismas reglas. La spec
nunca se inserta en el cierre de la implementación.

Se enumeran las **64 valoraciones** de esos átomos en FOUR: U=(0,0), T=(1,0),
F=(0,1), B=(1,1). El modelo conserva su truth order y su unidad de fusión I=B.
La satisfacción usa el filtro `I <= valor`, es decir {T,B}.

La denotación experimental de un programa es una función de valoraciones a FOUR:

\[
\llbracket R\rrbracket_v
=\bigwedge_{(A\le C)\in R}(v(A)\multimap v(C)).
\]

Una colección de axiomas se interpreta mediante **meet**, no tensor. La teoría
vacía vale el truth top T, no la unidad B ni el conocimiento vacío U. El orden
algebraico candidato es el orden punto a punto del álgebra de funciones:

\[
Impl\le_L Spec\iff
\forall v\;\llbracket Impl\rrbracket_v\le_T\llbracket Spec\rrbracket_v.
\]

No se compara sólo la valoración de los hechos derivados: se incluyen todas las
valoraciones, con evidencia negativa e inconsistente.

Las ocho componentes ground no tienen reglas que las conecten. Verificar 64
valoraciones por componente evita materializar 4^24 valoraciones globales. Para
esta comparación la reducción es exacta: meet preserva desigualdades locales;
un contraejemplo local se extiende poniendo T en las otras componentes, donde
todas las fórmulas de teoría/spec valen T y resultan neutras para meet. El test
conserva esa extensión para el contraejemplo encadenado. Esto no se extrapola a
programas con componentes conectadas.

## Tabla solicitada

Spec₀ exige O <= P; Spec₁ exige O <= P y O <= Q; Spec₂ exige O <= Q.
Impl₀ contiene O <= P; Impl₁ contiene ambas reglas; Impl₂ contiene O <= Q.

**La misma tabla resulta tanto para ≼Ω como para ≤L:**

| Implementación | Spec₀ | Spec₁ | Spec₂ |
| --- | --- | --- | --- |
| Impl₀ | Sí | No | No |
| Impl₁ | Sí | Sí | Sí |
| Impl₂ | No | No | Sí |

El test obtiene ≼Ω del checker anterior. La tabla esperada queda como regresión
para ambos cálculos independientes; no define sus resultados.

## Alternativa y producto

Se distinguen tres specs:

```text
dos garantías: (O ⊸ P) ∧ (O ⊸ Q) = O ⊸ (P ∧ Q)
alternativa:    O ⊸ (P ∨ Q)
producto:      O ⊸ (P ⊗ Q)
```

| Programa | Horn alternativa | ≤L alternativa | Horn requisito conjunto | ≤L producto |
| --- | --- | --- | --- | --- |
| Impl₀ | Sí | Sí | No | No |
| Impl₁ | Sí | Sí | Sí | **No** |
| Impl₂ | Sí | Sí | No | No |

La columna Horn conjunta significa que están soportados ambos eventos. No añade
un consecuente tensor al lenguaje ni presupone que esa comprobación implemente
la fusión FOUR: esa identificación es precisamente lo investigado.

Contraejemplo mínimo dentro del universo enumerado: Impl₁ frente a producto,
con **O=P=Q=U**:

```text
U ⊸ U = T
Impl₁ = T ∧ T = T
U ⊗ U = F
Spec_producto = U ⊸ F = U
T no <=T U
```

Las dos reglas son satisfechas por esa valoración, pero la regla con producto no.
Duplicar una premisa para obtener dos hechos y combinar sus soportes no equivale
a garantizar su fusión: el producto no es idempotente. Esta valoración no soporta
O; es un contraejemplo a la comparación incondicional, no un modelo de las reglas
junto con el hecho O.

Tampoco se colapsa una alternativa de consecuencias con una alternativa de
cláusulas. El test encuentra valores donde `O ⊸ (P ∨ Q)` difiere de
`(O ⊸ P) ∨ (O ⊸ Q)`. La igualdad con meet se comprueba en todas las valoraciones.

## Contraejemplo sin producto

Incluso para una spec Horn ordinaria:

```text
Impl: O <= P; P <= Q
Spec: O <= Q
```

Horn deriva Q desde O en las ocho instancias, por lo que Impl ≼Ω Spec. Pero con
**O=T, P=U, Q=F**:

```text
T ⊸ U = U
U ⊸ F = U
Impl = U ∧ U = U
Spec = T ⊸ F = F
U no <=T F
```

Esta valoración no satisface los axiomas de Impl: U no está en {T,B}. Así falla el
orden punto a punto aunque **cada modelo de Impl sí satisfaga Spec**. No son la
misma relación.

## Búsqueda exhaustiva y controles

Se enumeran todos los ocho subconjuntos de las reglas O <= P, O <= Q, P <= Q.
La última preserva el tiempo: en Formal es `protected(...) @ t <= insured(...) @ t`.
El grafo es acíclico. Se comparan ocho programas, cinco specs, ocho entradas y
64 valoraciones: **20 480 comprobaciones**, resumidas en 40 pares programa/spec.

| Comparación de aceptación | Horn sí / otra no | Horn no / otra sí |
| --- | ---: | ---: |
| Orden ≤L punto a punto | **5** | **0** |
| Todo modelo de Impl satisface la cláusula Spec | **3** | **0** |
| Modelos de Impl + hecho O soportan la consecuencia exigida | **0** | **0** |

Los cinco desacuerdos de orden son Impl₁ contra producto; la cadena O <= P <= Q
contra Spec₁, Spec₂ y producto; y las tres reglas juntas contra producto. Los tres
desacuerdos entre clases de modelos son los casos producto.

Con O efectivamente soportado, el control de consecuencia por modelos coincide
con Horn en **todos los 40 pares**. No demuestra completitud general ni se
rebautiza ese control como ≤L.

La búsqueda retiene contraejemplos ordenados por cantidad de reglas y luego por
bits de evidencia de su valoración. Ambos ejemplos mostrados necesitan dos reglas;
ningún programa con menos reglas falla **en este universo**. No se encontró un
caso donde el orden acepte y Horn rechace dentro de él.

La residuación participa directamente: se comprueba
`I <= (O ⊸ P) iff O <= P`, su instancia con producto y preservación de meet
en todas las valoraciones. No se cambia ninguna operación del modelo para forzar
coincidencias.

## Verificación

Los tres tests nuevos pasaron individualmente con `scripts/test_safe.py`.
Compilación inicial ~635 MiB RSS; tras añadir controles, ~461 MiB. Cada test usó
~15 MiB y ~0.33 s. Se conservaron 1 GiB total, sin swap, un job, un hilo y timeouts.
No se ejecutó la suite completa. Logs de resultados:
`target/test-diagnostics/20261001T024352330419Z-970414/`.

También pasaron formato y dos regresiones individuales: el checker de refinamiento
original y las leyes de fusión. Se reutilizó el binario compilado; logs en
`target/test-diagnostics/20261001T024821556133Z-973934/`.

El experimento mantiene separados refinamiento por consecuencias, consecuencia
condicional por modelos y orden de valores de teorías. Ninguno se declara ahora
como semántica definitiva de interfaz/implementación.
