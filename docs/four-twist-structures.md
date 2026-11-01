# FOUR como estructura twist y valores más allá de FOUR

Nota de diseño de 2026-11-01. No modifica código ni contratos: identifica la
estructura algebraica que ya implementa `experiments/four.rs`, la sitúa en la
literatura reciente y enumera extensiones posibles con su costo computacional
esperado. Las extensiones son propuestas sin implementar ni medir.

## 1. Lo que ya está en el código

`Value { p, n }` en [four.rs](../src/experiments/four.rs) define:

\[
(a_1,a_2)\otimes(b_1,b_2)=\bigl(a_1\wedge b_1,\;(a_1\Rightarrow b_2)\wedge(b_1\Rightarrow a_2)\bigr),
\qquad
(a_1,a_2)\le(b_1,b_2)\iff a_1\le b_1\ \text{y}\ b_2\le a_2 .
\]

Ése es exactamente el producto y el orden de verdad de una **estructura twist**
sobre el álgebra de Boole de dos elementos: el primer componente es soporte, el
segundo es soporte en contra. `I` es \((1,1)\), la unidad del producto. Con la
implicación twist

\[
(b_1,b_2)\multimap(c_1,c_2)=\bigl((b_1\Rightarrow c_1)\wedge(c_2\Rightarrow b_2),\;b_1\wedge c_2\bigr)
\]

se cumple la ley de residuación \(a\otimes b\le c\iff a\le b\multimap c\).
`Value` podría entonces implementar el contrato `CommutativeResiduatedLattice` de
[algebra.rs](../src/algebra.rs), junto con join y meet del orden de verdad.

**Verificado.** [scripts/four_twist_check.py](../scripts/four_twist_check.py)
copia `tensor` y `le` de `four.rs` y comprueba en los 64 casos: igualdad con el
producto twist, asociatividad, conmutatividad, unidad y residuación. Es
exhaustivo para FOUR; no ejecuta código Rust.

La descomposición en dos componentes es también la que hace válido el modelo
menos informado de la sección 12 de
[semi-naive-pending-optimizations.md](semi-naive-pending-optimizations.md).

## 2. Literatura

- **Busaniche, Galatos y Marcos, *Twist structures and Nelson conuclei*.**
  Unifican las retículas residuadas de Kalman, Nelson y Nelson paraconsistente
  como pares (retícula residuada involutiva cíclica, conúcleo de Nelson). Toda
  álgebra así se encaja en una estructura twist sobre la imagen del conúcleo.
  Es la referencia principal para esta nota: formaliza el par
  (soporte, soporte en contra) sobre **cualquier** retícula residuada y cubre el
  caso paraconsistente (B sin explosión).
- **Jansana y Rivieccio, *Residuated bilattices*.** El mismo encuentro visto
  desde el bilátice, con los órdenes de verdad y de conocimiento.
- **Cabrer y Priestley.** Obtienen la representación como producto de los
  bilátices distributivos como consecuencia de una dualidad natural por
  *piggyback* sobre Priestley.

Ninguno de estos trabajos aporta algoritmos ni cotas de complejidad. Su valor
para Residuum es de corrección: garantizan que el par de componentes es una
representación fiel también fuera de FOUR.

## 3. Extensiones: qué puede guardar cada componente

Cada fila reemplaza el álgebra de Boole de dos elementos por otra base. La
última columna es una estimación de esta nota, no un resultado de la literatura.

| Base de cada componente | Valor de ejemplo | Lectura jurídica | Costo esperado de la consecuencia |
|---|---|---|---|
| Booleana de 2 (FOUR, actual) | (sí, no) | hay o no prueba a favor y en contra | Horn, polinomial (§12) |
| Conjuntos de fuentes \(\mathcal P(S)\) | ({testigo A, cámara}, {documento de coartada}) | **qué** pruebas apoyan y cuáles contradicen | \(\mathcal P(S)\cong 2^S\): una copia independiente de FOUR por fuente; polinomial, multiplicado por \|S\| |
| Momentos (conjuntos ascendentes de tiempos) | (probado desde t3, refutado desde t7) | desde cuándo es exigible o controvertido | Base de Heyting; probablemente tratable con tiempo finito; por verificar |
| Costos tropicales \((\mathbb N\cup\{\infty\},\min,+)\) | (probar cuesta 4, refutar cuesta 9) | dificultad de establecer frente a desvirtuar | Caminos mínimos (Knuth 1977); polinomial con pesos no negativos |
| Grados en [0,1], Gödel (min) | (0,8 a favor, 0,3 en contra) | confianza combinada por el eslabón más débil | Punto fijo max-min; polinomial |
| Grados en [0,1], Łukasiewicz o producto | (0,8, 0,3) con combinación aritmética | acumulación de evidencia de tipo probabilístico | Más difícil: el producto no es idempotente; los puntos fijos pueden requerir infinitos pasos o programación lineal |

### Recomendación

**Conjuntos de fuentes.** En el dominio jurídico importa qué evidencia apoya o
contradice, no sólo si existe alguna. Es además la extensión más barata: como
\(\mathcal P(S)\) es booleana, la consecuencia se descompone en \(|S|\) copias
independientes del problema FOUR actual. Es un caso en que la dualidad de Stone
sí tiene consecuencia computacional directa.

## 4. Qué habría que comprobar antes de implementar

- Que la compilación Horn de la sección 12 se generaliza componente a componente
  para bases booleanas. El argumento por fuente es inmediato; el código no.
- Para bases de Heyting no booleanas (tiempo), si la consecuencia sigue
  decidiéndose en un modelo mínimo. La implicación ya no es \(\lnot a\lor b\).
- Para bases con producto no idempotente, si la evaluación semi-naive sigue
  siendo correcta. La deduplicación de justificaciones supone suma idempotente.
- Que `EvidentialNot` y la polaridad de los literales siguen siendo el
  intercambio de componentes.

## Fuera de alcance

Esta nota no cambia la semántica Horn de `close`, ni la consulta residual, ni
`FourEntailment`. Un evaluador con valores enriquecidos sería un modo nuevo con
la enumeración actual como oráculo en el dominio pequeño.
