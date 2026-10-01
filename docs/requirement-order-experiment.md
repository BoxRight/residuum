# Join, Tensor y comparación de explicaciones

Este experimento conecta las fórmulas de una consulta real con modelos del
contrato algebraico, sin modificar búsqueda, parser, runtime ni preferencia.
No selecciona todavía un modelo proposicional para M2 ni implementa un decisor
general de `≤_L`.

La fixture [order_formal.res](../examples/residual/order_formal.res) tiene dos
reglas, consideradas para cada tiempo `t`:

```text
A ≤ G
A ⊗ B ≤ G
```

La consulta con evidencia vacía devuelve `H1 = A`, `H2 = A ⊗ B` y un resumen
`Join(H1, H2)`. Ambas pruebas siguen separadas.

## Interpretación y orden

`algebra::interpret_requirement(formula, valuation)` interpreta `Unit`, eventos,
`Tensor` y `Join` mediante las operaciones del modelo suministrado. La valoración
debe respetar la identidad lógica de los eventos, incluidos tiempo y polaridad,
sin depender de `transition`. Un átomo no asignado devuelve el error del caller:
no se lo identifica implícitamente con unidad o bottom.

Para razonar respecto de un programa, además se necesita que la valoración
satisfaga sus desigualdades. Los tests comprueban esa condición antes de utilizar
cada modelo como ejemplo o contraejemplo. Sus valoraciones son constantes por
verbo a través de todos los tiempos: esto extiende la interpretación a los
esquemas de esta fixture, sin identificar tiempos en el AST.

`Preorder::compare` consulta el mismo `le` en ambas direcciones:

| `H1 ≤ H2` | `H2 ≤ H1` | Comparación |
| --- | --- | --- |
| sí | sí | `Equivalent` |
| sí | no | `Below` estricto |
| no | sí | `Above` estricto |
| no | no | `Incomparable` |

Esto no crea un orden de preferencia. Requiere un `le` exacto en el modelo;
fallar en una búsqueda incompleta de pruebas no demuestra incomparabilidad.
La equivalencia de valores tampoco elimina los testigos de la consulta.

## Qué garantiza el join

Por la propiedad universal del lattice:

```text
H1 ∨ H2 ≤ G  ⇔  H1 ≤ G y H2 ≤ G
```

El resumen de alternativas sigue por debajo del objetivo. Es su least upper
bound; no implica que las dos ramas sean comparables ni que se deban aportar
ambas simultáneamente. `Tensor` combina requisitos dentro de una rama.

Los tests recorren todas las asignaciones de `A`, `B` y `G` en los dos modelos
finitos existentes, conservando sólo las que satisfacen el programa. Comprueban
el bound sobre el goal y la propiedad universal del join frente a cada elemento
del carrier. También comprueban la adjunción para la segunda regla y la
interpretación de distribución del tensor sobre join y de unidad bilateral.
Esto verifica el adaptador y sus interpretaciones finitas; no prueba que el
algoritmo de consulta calcule el residual general en todo `L`.

## Más requisitos no fija por sí solo la dirección

Todos estos ejemplos satisfacen las mismas dos reglas:

| Modelo y valoración | `H1 = A` | `H2 = A ⊗ B` | Relación |
| --- | --- | --- | --- |
| Łukasiewicz: `A=1`, `B=1`, `G=1` | `1` | `0` | `H2 < H1` |
| Powerset de Z/2Z: `A={1}`, `B={0,1}`, `G={0,1}` | `{1}` | `{0,1}` | `H1 < H2` |
| Powerset de Z/2Z: `A={1}`, `B={1}`, `G={0,1}` | `{1}` | `{0}` | Incomparables |
| Powerset de Z/2Z: `A={1}`, `B={0}=I`, `G={1}` | `{1}` | `{1}` | Equivalentes |

En el powerset, el tensor es suma de elementos del grupo, el orden es inclusión
y la unidad es `{0}`. No es el dominio de conocimiento `K`. En la cadena,
`a ⊗ b = max(0, a+b-2)` y `I=2`; el producto difiere del meet.

El tercer ejemplo refuta ambas desigualdades entre `H1` y `H2` en un modelo que
satisface el contrato y el programa. Por tanto, ninguna se deduce del contrato
general más esas dos reglas. Esto es un contraejemplo explícito, no una ausencia
de prueba. Los modelos son referencias experimentales, no semánticas elegidas
para las proposiciones de Residuum.

## La condición que permite descartar requisitos

Si `B ≤ I`, la monotonía y la unidad dan:

```text
A ⊗ B ≤ A ⊗ I ≃ A.
```

Si esto debe valer para todos los factores, se necesita `B ≤ I` para todo `B`:
integralidad. A la inversa, si `A ⊗ B ≤ A` vale universalmente, tomando `A=I`
obtenemos `B ≤ I`. La posibilidad universal de descartar factores es así
equivalente a esta condición; no se sigue del contrato residuado actual.
La definición de integralidad y su distinción de conmutatividad y contraction
se encuentran en [Galatos y Ono, sección 2](https://www.jaist.ac.jp/~galatos/research/slai.pdf).
El argumento anterior usa directamente las leyes de nuestro contrato.

Este experimento no adopta integralidad. Para un conjunto particular de
abducibles podría bastar justificar `B ≤ I` sólo para esos factores; esa sería
una decisión explícita del modelo o de la teoría, no una política oculta.

## Qué queda abierto para preferencia

La orientación de `≤_L` ya es entailment: `H1 ≤ H2` significa que `H1` implica
`H2`. Si el criterio elegido fuese «explicación lógicamente más débil», se
buscarían elementos maximales admisibles, módulo equivalencia. No necesariamente
habría un máximo único. Pero «más débil» no equivale automáticamente a «menos
factores»: los contraejemplos anteriores impiden identificar ambos criterios
con el contrato actual.

Además hay que definir qué elementos son explicaciones admisibles. El join que
resume dos ramas no es por defecto una tercera hipótesis concreta elegida por
el caller. Si se admiten también joins como explicaciones, la selección de
maximales debe considerar ese dominio ampliado. No se implementa esa política
en este paso.

La siguiente decisión es semántica: qué realización de `L` queremos para los
requisitos abducibles y qué leyes estructurales admite. Las comparaciones
experimentales sirven para tomarla; no cambian el comportamiento de la búsqueda.
