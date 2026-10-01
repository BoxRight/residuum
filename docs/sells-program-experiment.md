# sells como seed abducible con comportamiento asociado

El programa está en
[sells_program.res](../examples/experiments/sells_program.res); el verificador está
en [sells_tests.rs](../src/experiments/sells_tests.rs). Todas las declaraciones,
reglas, constantes, datos iniciales y objetivos están en Formal. Rust sólo parsea,
ejecuta las operaciones existentes y compara sus resultados. No se cambia el motor
ni se añade sintaxis ProgramVerb, interface, ensures o abducible.

## Asociación mediante reglas existentes

La declaración `seeded verb sells` conserva la política existente: un Seeded puede
ser requisito abducible por defecto. Su comportamiento se expresa mediante una
regla parametrizada ordinaria:

```text
rule sellsProgram(seller, object, buyer, t) =
    sells(seller, object, buyer) @ t
    <= give(seller, object, buyer) @ after t

rule juanSale = sellsProgram(juan)
```

`give` es un Effect cuyo cuerpo quita el objeto de `subject.assets` y lo añade a
`recipient.assets`. Otras reglas de la misma fixture hacen que ese Effect produzca
`owns`, y que `owns` produzca la consecuencia normativa `notDo`. Para este programa
experimental, `notDo` es Derived, sin transformación de State.

También se reutiliza la lambda de protección mediante
`rule protectionFromCharlie = ownsProgram(charlie, molest)`.

No se almacena una lambda dentro del Event sells. La asociación es operacional:
las reglas declaradas del programa tienen sells como antecedente. No se producen
reglas nuevas en runtime ni se obtiene una certificación modular de implementación.

## Dos caminos de la misma instancia

Los bloques `real` y `hypothetical` tienen exactamente el mismo objetivo:

```text
give(juan, car, pedro) @ after tau
```

El primero contiene el seed `sells(juan, car, pedro) @ tau`; el segundo no contiene
ninguna evidencia. El runner externo observa:

| Caso | Objetivo en Horn | Requisito residual | Known tras la consulta/cierre original |
| --- | --- | --- | --- |
| real | Sí | I | sells, give, owns, notDo |
| hypothetical | No | sells(juan, car, pedro) @ tau | Vacío |

I es el requisito satisfecho/unidad lógica; no se usa para representar Known vacío.
El bloque hipotético utiliza `seeds {}`.

El residual retorna dos justificaciones, una por `sellsProgram` y otra por
`juanSale`. Ambas requieren el mismo seed, así que la fórmula requerida se consolida
en un único Event. Se conservan las dos ramas de prueba, sin política de preferencia.

El test toma la hipótesis **del resultado residual**, no fabrica un seed con Rust.
La suministra únicamente a un cierre hipotético separado. Ese cierre y el real
tienen la misma cadena:

```text
sells(juan, car, pedro) @ tau
  -> give(juan, car, pedro) @ after(tau)
  -> owns(pedro, car) @ after(tau)
  -> notDo(charlie, molest, car, pedro) @ after(after(tau))
```

El cierre original sin evidencia sigue vacío. La fixture y el input de la consulta
también permanecen intactos. Una hipótesis no se convierte en evidencia real.

## Qué se compara

Se comparan los eventos conocidos y las cuatro justificaciones: dos para give,
una para owns y una para notDo. Para cada justificación se exige exactamente la
misma regla de origen, antecedentes ordenados y sustitución; no se renombra
ninguna regla para hacer coincidir los caminos.

Además de la igualdad lógica de Event, se comparan explícitamente PropositionType
y StateTransitionType, ya que la igualdad lógica ignora metadata operacional.
El Effect give tiene entrada State(tau) y salida State(after(tau)) en ambos caminos.
La sustitución de cada prueba residual coincide con la instancia forward de esa
misma regla.

DerivationTime se suministra externamente. El cierre hipotético recibe sellos
`hypothetical-N`; el real usa el reloj habitual. Se comparan los registros de
provenance sin identificar esos relojes independientes. Los índices de PropositionTime
y de transición sí deben coincidir exactamente.

## Reutilización y controles

El bloque `reuse` contiene otra venta: `sells(pedro, car, juan) @ tau`. Produce la
misma estructura de consecuencias para juan. La regla genérica se reutiliza;
`juanSale` no aplica porque su vendedor está fijado a juan por β-reduction.

Se compara la regla aplicada con su versión inline usando la gramática actual.
Ambas tienen iguales parámetros residuales y cuerpo tipado, y todos los bloques
producen los mismos cierres, provenance y resultados residuales.

Dos controles muestran que el comportamiento depende de las reglas de la fixture:

- Al quitar las reglas, el seed real sólo produce sells. El objetivo Effect no
  puede inventarse abductivamente y la consulta hipotética falla como NotAbducible.
- Al renombrar sells a offers en toda la fuente, las mismas construcciones producen
  la misma cadena y un requisito offers. El motor no contiene un tratamiento de sells.

## Incrementalidad

Se utiliza directamente el IncrementalClosure existente para mantener eventos,
incluidos los Effect, sin ejecutar sus cuerpos. El wrapper experimental
`run_incremental_experiment` excluye Effects y no se modifica esa restricción.

Se comparan actualizaciones desde dos bases definidas en la fuente: vacía y con
la venta del bloque reuse ya cerrada. Para cada rama residual se compara añadir
el seed real con añadir su hipótesis a una instancia incremental hipotética:

- mismos eventos nuevos y nuevos registros de derivación;
- mismos tiempos, metadata y estadísticas de actualización;
- mismo cierre que recomputar con close;
- los eventos nuevos son exactamente la diferencia respecto de la base;
- se conservan las derivaciones previas y sus sellos;
- reinsertar la hipótesis no genera eventos, pruebas ni trabajo adicional.

No hay ejecución de State, incrementalidad de snapshots, defaults ni negación
por ausencia en este experimento.

## Ejecución y resultado

```sh
python3 scripts/test_safe.py --fixture examples/experiments/sells_program.res
```

Los tres tests específicos pasaron individualmente. La compilación inicial usó
~443 MiB RSS; el primer intento detectó una coma no admitida en la declaración
de entity y se detuvo. Sólo se corrigió la fixture. La compilación posterior usó
~344 MiB, y cada test ~15 MiB y 0.02–0.04 s. El runner externo también ejecutó
la fixture: ~225 MiB al compilar y ~15 MiB al ejecutar.

Se conservaron 1 GiB total, sin swap, un trabajo, un hilo y los timeouts. No se
ejecutó la suite completa. Logs de tests:
`target/test-diagnostics/20261001T031823089981Z-998413/`; del runner externo:
`target/test-diagnostics/20261001T031909242700Z-999351/`.

También pasaron formato y dos regresiones individuales: la cadena owns y la
política existente de Seeded abducible. Se reutilizó el binario de tests compilado.
Logs: `target/test-diagnostics/20261001T032105823716Z-1000914/`.

**La hipótesis queda confirmada para este caso:** la misma instancia Seeded puede
ser evidencia real o requisito abducible y activar, al suministrarse como evidencia
hipotética, el mismo comportamiento programático declarado mediante reglas.
Esto no demuestra una nueva categoría de verbos ni verificación interna de contratos.
