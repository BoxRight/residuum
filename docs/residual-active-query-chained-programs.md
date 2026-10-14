# Consulta activa: Unit y Programs encadenados

Verificación en `experiment-legal-order-geometry`. Este incremento cambia sólo
tests, fixtures experimentales y documentación. Conserva la API activa que ya
estaba implementada, la consulta Horn histórica, el forward, defaults y la CLI.
No se modifica main ni el tag v1.

## Regresión temporal de I

`no_explanation_is_distinct_from_ground_unit_rule` construía una regla con tau
no declarado y después borraba sus parámetros. Las apariciones temporales
seguían siendo `TimeExpr::Var`, mientras el objetivo era `TimeExpr::At`.

La fuente ahora declara `const tau : PropositionTime`. El test comprueba que
el elaborador produce cero parámetros y At(tau), sin modificar el AST. Verifica
dos resultados distintos:

- La regla ground `I <= goal() @ tau` devuelve Some(Unit) y permite derivar goal.
- Una conclusión sin prueba devuelve None y cero explicaciones.

Unit sigue siendo la unidad del producto de requisitos; no es evidencia vacía
ni una respuesta por defecto cuando falta una prueba. El matcher no cambia.

## Experimento fuente

Definición:
[residual_activation_chain.res](../examples/experiments/legal_geometry/residual_activation_chain.res).
Consumidor mediante import:
[residual_activation_chain_scenario.res](../examples/experiments/legal_geometry/residual_activation_chain_scenario.res).

```
starts(item) @ tau
  activa Prepare
    prepareFirst:  I      -> middle(item) @ tau
    prepareSecond: middle -> handoff(item) @ tau
  handoff activa Finish
    finishFirst:   I         -> processed(item) @ after tau
    finishSecond:  processed -> completed(item) @ after tau
```

Cada Program comparte object/t entre sus dos RuleUse. Las instancias se
especializan usando los mecanismos existentes. Las dos reglas de cada programa
y las activaciones encadenadas participan en un mismo close_program. Declarar
o importar estos Programs no obtiene ninguna consecuencia sin un trigger.

## Resultado finito

Se examinó cada uno de los 16 subconjuntos de:

```
{ starts(item)@tau, middle(item)@tau,
  handoff(item)@tau, processed(item)@after tau }
```

Los eventos Derived usados como evidencia en ese universo están tipados y
proceden del cierre real; el experimento no los presenta como nuevos Seeded.
Se añadieron cuatro entradas: trigger negativo, tiempo incorrecto, objeto
incorrecto y objetivo ya evidenciado. Total: 20 consultas.

| Evidencia | Soporte pendiente, como conjunto de hojas |
|---|---|
| Sin starts ni handoff ni objetivo | starts(item) @ tau |
| Sólo middle o processed | starts(item) @ tau |
| starts positivo, o handoff conocido | Ninguno: I |
| Trigger negativo, de otro objeto o tiempo | starts(item) @ tau |
| Objetivo conocido | I, sin pasos de activación |

Cada entrada produce una explicación certificada y alcanza el objetivo. La
tabla resume las hojas como conjunto; no elimina la multiplicidad de Tensor
en la fórmula ni los pasos repetidos de la prueba residual.

Desde evidencia vacía, el cierre hipotético coincide con el seed real: cinco
eventos y cuatro justificaciones. La consulta conserva inmutables módulo y
evidencia; el soporte sólo se incorpora al cierre hipotético.

En las 20 entradas, se comparó el certificado con close_program_reference sobre
exactamente la misma evidencia ampliada. Coinciden:

- eventos completos, incluidos tipo, tiempo y metadata operacional;
- registros completos ordenados para comparar, sin convertirlos en un conjunto:
  misma multiplicidad, reglas, antecedentes ordenados y sustituciones;
- orígenes Program/Rule/Module y triggers especializados.

Sólo se excluye el sello derivedAt de relojes independientes. Los cuatro
productores reales son Prepare::0::prepareFirst, Prepare::1::prepareSecond,
Finish::0::finishFirst y Finish::1::finishSecond. Todos conservan origen en el
módulo importado ResidualActivationChain, no en el escenario consumidor.

Datos: [active-residual-chained-programs.tsv](legal-geometry/data/active-residual-chained-programs.tsv).

## Compatibilidad y límite separado

El test de compatibilidad conserva estas diferencias:

| Consulta sin abducibles | Horn histórico | Activa |
|---|---|---|
| Evidencia vacía | None / NotForwardSupported | None / NotAbducible |
| Sólo processed(item) @ after tau | I | None / NotAbducible |

En el segundo caso, processed determina todos los parámetros de finishSecond
para la prueba Horn, pero falta handoff para activar Finish. La consulta activa
no certifica esa prueba como ejecutable.

El primer caso expuso una expectativa incorrecta del test nuevo: se esperaba I
de la consulta Horn. Su replay no instancia parámetros determinados sólo desde
el objetivo si el cuerpo es I y no proporciona testigos para esos bindings.
Se corrigió la expectativa tras inspeccionar ese replay, sin modificarlo.
Esta limitación local queda separada de la activación y de la regla Unit ground
de la regresión. No demuestra que I tenga otra semántica algebraica.

## Verificación

Compilación separada con CARGO_BUILD_JOBS=1 y CARGO_PROFILE_TEST_DEBUG=0; ejecución
mediante test_safe.py --binary con nombres exactos. Límites intactos: 1 GiB,
sin swap, umbral preventivo 768 MiB, un trabajo Cargo y un hilo del harness.

- Biblioteca: tres tests de Unit/Tensor pasaron.
  Informe `target/test-diagnostics/20261010T032416651077Z-793413`.
- CLI: dos tests nuevos y tres regresiones activas previas pasaron.
  Informe `target/test-diagnostics/20261010T032644278164Z-796751`.
- El test de 20 entradas terminó en 0,0414 s, con 15,32 MiB de pico RSS.
- No se ejecutó la suite completa, ni se hizo merge o commit.

La primera compilación del test modificado falló por un import de RuleBody
faltante; se corrigió ese import. La primera ejecución del test de compatibilidad
falló por la expectativa Horn descrita arriba. Ambos fallos quedaron registrados
en el runner y se resolvieron antes de la verificación final.

Estos son resultados finitos. No prueban completitud general, resolución
existencial, terminación de cualquier cierre hipotético ni soporte de defaults.
La certificación sigue siendo condicional a que close_program termine.
La adopción de la consulta activa por CLI u otros consumidores queda separada;
esta verificación proporciona una base para evaluarla, sin cambiar sus rutas.
