# Modelos finitos con evidencia positiva y negativa

Este experimento añade modelos de referencia a los tests algebraicos. No cambia
`query`, `close`, EvidentialNot del runtime ni el contrato de M2. No elige aún una
interpretación definitiva de las proposiciones.

El carrier es `FOUR = {0,1}²`, con pares `(p,n)` de soporte positivo y negativo:

| Nombre | Par | Lectura |
| --- | --- | --- |
| U | (0,0) | Sin evidencia |
| T | (1,0) | Sólo evidencia positiva |
| F | (0,1) | Sólo evidencia negativa |
| B | (1,1) | Ambas evidencias |

Es la bilattice FOUR de Belnap. Su expansión con fusión y residual se describe
en [Jung y Rivieccio, sección II](https://achimjungbham.github.io/pub/papers/Jung-Rivieccio-2013-Kripke-semantics-for-modal-bilattice-logic.pdf).
Los símbolos del paper para operaciones de conocimiento no deben confundirse
con el `Tensor` de Residuum. Aquí reservamos `⊗` para el producto residuado.

## Dos órdenes y EvidentialNot

Para `x=(p,n)` e `y=(q,m)`, las comparaciones entre bits son las usuales:

```text
x ≤_k y  ⇔  p ≤ q y n ≤ m
x ≤_t y  ⇔  p ≤ q y m ≤ n
```

El primero acumula ambos soportes. El segundo aumenta soporte positivo y reduce
soporte negativo. Sus extremos son `U ≤_k x ≤_k B` y `F ≤_t x ≤_t T`.

```text
EvidentialNot(p,n) = (n,p)
```

Es involutivo, monótono en conocimiento y antítono en verdad. Fija U y B: no
convierte ausencia en negación y no borra una contradicción. Las operaciones son:

```text
x ∨_k y = (p ∨ q, n ∨ m)       acumulación de conocimiento
x ∧_k y = (p ∧ q, n ∧ m)
x ∨_t y = (p ∨ q, n ∧ m)       join lógico de valores
x ∧_t y = (p ∧ q, n ∨ m)       meet lógico de valores
```

No identificamos `∨_k` con `∨_t`, ni el carrier de valores con las fórmulas de L.
Una desigualdad entre fórmulas necesita cuantificar sobre las valoraciones
admisibles de la teoría; una comparación entre dos valores es sólo puntual.

## Modelo integral: producto igual al meet de verdad

Una primera realización es:

```text
x ⊗ y = x ∧_t y
I = T
b ⊸ c = (¬p_b ∨ p_c, ¬n_b ∧ n_c)
```

`¬` aquí es complemento de un bit, no EvidentialNot. La transformación
`(p,n) ↦ (p,¬n)` muestra el lattice booleano de dos coordenadas que justifica
el residual. Se cumplen todas las leyes de nuestro contrato: lattice, monoide
conmutativo, monotonía en verdad, adjunción, distribución, variancia y
currificación iterada. También el producto es monótono en conocimiento.

Es un modelo permitido, pero adicionalmente integral e idempotente: tensor y
meet coinciden. No aporta una interpretación que los distinga. Tampoco permite
identificar EvidentialNot con `x ⊸ F`: para `x=B`, este residual vale U, mientras
EvidentialNot(B)=B. La negación evidencial permanece como operación independiente.

## Modelo de fusión: producto distinto del meet

Una segunda realización sobre los mismos pares y órdenes es:

```text
(p,n) ⊗ (q,m) = (p ∧ q, (¬q ∨ n) ∧ (¬p ∨ m))
I = B
b ⊸ c = ((¬p_b ∨ p_c) ∧ (¬n_c ∨ n_b), p_b ∧ n_c)
```

Su tabla de producto es:

| ⊗ | F | U | B | T |
| --- | --- | --- | --- | --- |
| F | F | F | F | F |
| U | F | F | U | U |
| B | F | U | B | T |
| T | F | U | T | T |

Este modelo también cumple exhaustivamente todo el contrato residuado sobre
`≤_t`. No es integral y distingue tensor del meet: `U ⊗ U=F`, pero `U ∧_t U=U`.
Con `~` como EvidentialNot, además:

```text
b ⊸ c = ~(b ⊗ ~c)
~x = x ⊸ ~I
```

El elemento dualizante `~I` es B, no el bottom de verdad F. No se introduce la
ley clásica `x ⊗ ~x ≤ F`: con `x=B`, el producto es B y B no está debajo de F.
Eso evita interpretar automáticamente evidencia opuesta como explosión.

Hay un costo para la interpretación de acumulación: el producto completo no es
monótono en `≤_k`. Por ejemplo:

```text
F ≤_k B,
F ⊗ T = F,
B ⊗ T = T,
pero F no ≤_k T.
```

Puede desaparecer soporte negativo al aumentar conocimiento. El soporte positivo
del producto sí es `p ∧ q` y monótono; esto permite explorar un puente mediante
satisfacción positiva, pero no prueba por sí solo fidelidad al matcher de M1.

## La unidad y la evidencia vacía

En ambos modelos `U` es el mínimo de conocimiento. La unidad lógica I es T o B,
respectivamente. Un antecedente vacío debe estar satisfecho; no es lo mismo que
un átomo del mundo para el cual falta toda evidencia. Interpretar I como un valor
no significa insertar evidencia sobre cualquier evento en `Known`.

Hay tres papeles: `⊥_K=∅`, `I=1_L` y, en estos modelos acotados, `0_L`.
En FOUR corresponden a valoración atómica U, unidad T o B y bottom de verdad F,
respectivamente. Más precisamente, `Known=∅` asigna U a **cada átomo**; el conjunto
de conocimiento no es por sí mismo un elemento de FOUR. `I=B` es una propiedad
del modelo de fusión, no una decisión general para M2.

Si exigimos simultáneamente `I=U`, residuación sobre el orden de verdad estándar
y monotonía del producto completo en conocimiento, estas condiciones son
incompatibles. El argumento es pequeño:

1. Un producto residuado preserva bottom de verdad: `F ⊗ T=F`.
2. `U ≤_k F` y monotonía exigirían `U ⊗ T ≤_k F ⊗ T`.
3. Si U fuese unidad, esto daría `T ≤_k F`, que es falso.

No se afirma que U no pueda ser unidad de ningún producto residuado; la
imposibilidad usa las tres condiciones juntas.
El test enumera las 256 columnas posibles `f(x)=x ⊗ T` y verifica que ninguna
columna monótona en conocimiento con `f(U)=T` tiene adjunto derecho en verdad.
No enumera sin límites todas las tablas de productos posibles.

Tomar simplemente `⊗ = ∨_k` tampoco sirve. Aunque es un monoide de unidad U,
no tiene residual respecto de `≤_t`: `F ∨_k T=B` no está debajo de F, mientras
que F está debajo de cualquier candidato a residual. La adjunción falla.

## El puente con las reglas Horn sigue abierto

Una desigualdad puntual en verdad no es lo mismo que una regla que preserva
soporte positivo. Por ejemplo, `A=T`, `G=T` satisface `A ≤_t G`. Si añadimos
evidencia negativa sobre G, su valor pasa a B y la desigualdad deja de valer,
aunque G conserva soporte positivo.

Por tanto, el conjunto de valoraciones que satisface todas las reglas como
desigualdades de verdad no es automáticamente cerrado bajo acumulación de
conocimiento. El closure positivo actual sí conserva sus eventos al añadir
evidencia opuesta. Hay que especificar cómo `K ⊨ A`, el orden entre fórmulas y
las desigualdades de valor se relacionan; no se migra ese puente por intuición.

Para una familia finita de eventos ground se puede usar `FOUR^Events` como
valoraciones, con acumulación puntual. Si el universo de eventos o los tiempos
es infinito, el carrier de valores sigue teniendo cuatro elementos, pero el
dominio de valoraciones completo deja de ser finito.

El [experimento de satisfacción](satisfaction-bridge-experiment.md) compara una
valoración explícita con el matcher y el cierre ground. Soporte positivo coincide
en ese fragmento; desigualdad puntual y cierre Horn siguen siendo distintos.

## Resultado del experimento

Sí existen modelos finitos con los dos órdenes, EvidentialNot y residuación.
Las leyes del contrato algebraico sobreviven en ambos productos. No todas las
identificaciones operacionales sobreviven: unidad como ausencia de evidencia,
join de conocimiento como tensor, monotonía completa de la fusión en conocimiento,
y negación evidencial como complemento/residual a F necesitan distinguirse.

Los tests están en `src/algebra/tests/evidence_pairs.rs`: recorren todos los
elementos y tuplas del carrier para las leyes, y conservan los contraejemplos
como assertions esperadas. Sólo se ejecutan individualmente mediante el runner
limitado. Estos modelos quedan como referencias experimentales; abducción,
preferencia, parser y runtime permanecen congelados.
