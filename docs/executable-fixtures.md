# Programas experimentales con datos y consultas

Los experimentos ahora se definen en archivos Formal. Rust los parsea, elabora
y ejecuta; los tests comprueban resultados sin fabricar sus eventos de entrada.
`examples/run_fixture.rs` acepta un archivo de experimento, sin construir una CLI
general. En modo incremental puede recibir una biblioteca de declaraciones con
`--library`, para el [experimento de reutilización](owns-reuse-experiment.md).

```text
archivo .res → parser → SurfaceAST → TypedAST → consulta/cierre → salida
```

## ResidualProof

[residual_proof.res](../examples/experiments/residual_proof.res) contiene entidades,
constantes, verbos seeded, el cuerpo real de `give`, `sales` y tres escenarios.

```text
experiment empty {
    seeds {}
    query residual give(alice, car, bob) @ after tau
}
```

`priceOnly` y `both` escriben sus eventos dentro de `seeds {...}`. Cada escenario
tiene entrada independiente. Las comas internas son obligatorias; sólo la final
es opcional. `seeds {}` es conocimiento vacío, no `I`.

```sh
python3 scripts/test_safe.py --fixture examples/experiments/residual_proof.res
```

Con A como acuerdo de precio y B como acuerdo de objeto, la salida observada es:

| Experimento | Seeds | Requisito | Horn deriva give |
| --- | --- | --- | --- |
| `empty` | `∅` | `A ⊗ B` | No |
| `priceOnly` | `{A}` | `B` | No |
| `both` | `{A,B}` | `I` | Sí |

La consulta utiliza las seeds originales y se calcula un cierre separado para
mostrar qué se deriva. No se afirman requisitos, no se mezclan los escenarios
y no se ejecutan efectos ni se modifican snapshots.

Los argumentos deben ser constantes declaradas con tipos compatibles. El
índice se declara con `const tau : PropositionTime`. Tiempo sin declarar,
argumentos desconocidos, aridad incorrecta o tipos incompatibles fallan durante
elaboración. Las entradas `seeds` admiten exclusivamente verbos Seeded.

## Comparación FOUR desde el propio programa

[four_counterexample.res](../examples/experiments/four_counterexample.res) escribe
P, Q, la regla y la teoría consultada:

```text
const tau : PropositionTime
derived verb P()
derived verb Q()

rule implication() = P() @ tau <= Q() @ tau

experiment contraposition {
    premises { ~Q() @ tau, }
    query fourFusion ~P() @ tau
}
```

La constante temporal no se infiere como parámetro de esa regla. `premises`
identifica hipótesis lógicas de una comparación; no afirma que una proposición
Derived haya sido derivada externamente. El runner mantiene esa distinción:
por ahora residual requiere `seeds` y fourFusion requiere `premises`.

```sh
python3 scripts/test_safe.py --fixture examples/experiments/four_counterexample.res
```

Salida observada:

```text
experiment contraposition
  Premises: {~Q() @ tau}
  goal: ~P() @ tau
  Horn: false
  Known after close: {~Q() @ tau}
  FOUR fusion: true (3 models / 16 interpretations)
```

El evaluador enumera todas las interpretaciones del universo descrito por el
archivo. Usa fusión FOUR, filtro de I y reglas como desigualdades de verdad,
según el [experimento semántico](four-model-entailment.md). No incorpora la
conclusión algebraica a `Known` ni introduce contraposition en `close`.

## Alcance y ejecución segura

`fourFusion` nombra explícitamente el modelo de la consulta, sin adoptarlo como
semántica operacional general de M2. Admite hasta cuatro átomos ground,
argumentos constantes, reglas sin parámetros y un único tiempo común. Esta
última restricción evita confundir el matching actual de nombres temporales
con un nuevo chequeo de constantes literales entre índices distintos. El matcher
no se cambia en este incremento.

El modo residual exige dependencias acíclicas entre verbos, una condición
suficiente de terminación para estas fixtures. FOUR admite ciclos ground dentro
de su universo finito. Se rechazan defaults. Estos modos no añaden `!`, derivada discreta,
preferencia abductiva, ejecución de snapshots ni drafter. CNL permanece pausado.

La entrada es un archivo regular de hasta 64 KiB. `test_safe.py --fixture` compila
el ejemplo y lo ejecuta en fases y cgroups separados: 1 GiB total, sin swap, un
trabajo de Cargo, timeout de compilación de 45 s y ejecución de 15 s. Conserva la
detención por crecimiento anormal y cercanía al límite. `--fixture` no se combina
con opciones de tests ni dispara la suite completa.

Se verificaron seis tests nuevos y tres regresiones individualmente. Ambos
programas también se ejecutaron desde sus archivos. El pico RSS fue 638 MiB al
compilar tests y aproximadamente 15 MiB al ejecutar las fixtures.

El modo externo `--incremental`, documentado en
[incremental-close.md](incremental-close.md), compara la actualización forward
con una recomputación completa sobre los bloques `base` y `delta`. Reutiliza
la gramática de experimentos y no ejecuta sus consultas residuales.
