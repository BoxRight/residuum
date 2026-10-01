# Refinamiento finito de implementación respecto de spec

Este experimento reemplaza el criterio de igualdad normalizada por la presencia
de consecuencias en el cierre Horn **sobre un dominio explícito finito**. El
experimento anterior se conserva como control; su checker ya permitía reglas
adicionales cuando encontraba una cláusula igual a la garantía. La diferencia
nueva es aceptar una garantía obtenida mediante varios pasos, sin esa cláusula
directa en la implementación.

Todo el checker vive en
[refinement_tests.rs](../src/experiments/refinement_tests.rs). No se modifican
close, matcher, AST, algebra::Preorder, residual, defaults o negación. No se añade
Contract, interface o ProgramVerb al lenguaje.

## Programas de entrada

- [Spec débil](../examples/experiments/owns_refinement_spec.res): owns @ t
  garantiza protected @ after(t).
- [Spec fuerte](../examples/experiments/owns_refinement_strong_spec.res): exige
  protected e insured @ after(t).
- [Implementación exacta](../examples/experiments/owns_refinement_exact.res):
  contiene la cláusula directa de protección.
- [Implementación ampliada](../examples/experiments/owns_refinement_extra.res):
  owns deriva entitlement; entitlement deriva protected. Además, owns deriva
  insured. No contiene la cláusula directa de protección.
- [Implementación insuficiente](../examples/experiments/owns_refinement_missing.res):
  sólo deriva insured.
- [Dominio](../examples/experiments/owns_refinement_domain.res): dos propietarios
  (alice, bob), dos cosas (car, house) y dos tiempos (tau, sigma): ocho casos ground.

El dominio se obtiene de seeds transfer y una regla de soporte que deriva owns.
Sus valores están definidos y tipados en fuente. Esa regla de soporte no recibe
las garantías. Cada comprobación posterior toma una proposición owns como
premisa hipotética, sin insertar ninguna hipótesis en el conocimiento del usuario.

## Relación que se comprueba

Para el conjunto Ω de ocho premisas y cada cláusula A <= C de la spec:

```text
Impl satisface_Horn,Ω Spec
    iff, para cada premisa P de Ω y cada garantía:
        θ = match(A, P)
        θ(C) pertenece a Known(close({P}, reglas(Impl)))
```

La spec sólo construye los objetivos esperados con la sustitución. Sus reglas
se eliminan antes de elaborar la implementación y nunca participan en ese cierre.
Las declaraciones públicas se utilizan para tipar el cuerpo; se permiten verbos
Derived privados y se rechaza sustituir las firmas públicas. Las garantías deben
ser Horn, tener un antecedente owns, quedar instanciadas en todo Ω y no dejar
variables pendientes. Un dominio vacío se rechaza en vez de aceptar por vacuidad.

Cada obligación guarda premisa, objetivo, cierre real y, si existe, la derivación
que justifica el objetivo. Una consecuencia adicional no es una obligación
incumplida. La ausencia de la consecuencia esperada proporciona un contraejemplo
ground, junto con los eventos que realmente se derivaron.

Se exige un grafo acíclico de dependencias de la implementación antes de llamar
close. Es una restricción conservadora suficiente de terminación para este
experimento, no una conclusión de monotonicidad. No se ejecutan efectos.

## Orientación entre specs

El mismo checker puede tratar las reglas de una spec como programa candidato,
sin volver a declarar las firmas. Se comparan los dos sentidos sobre el mismo Ω.
Si escribimos R ≼Ω S para «R satisface las garantías de S»:

```text
Spec_fuerte ≼Ω Spec_débil
Spec_débil no ≼Ω Spec_fuerte
```

Por esta convención, las garantías más fuertes quedan **por debajo**. La inclusión
de hechos derivados va en el sentido contrario: las consecuencias de la spec
débil están incluidas en las de la fuerte. Esto separa el refinamiento de programas
del orden de acumulación de Known.

Esa orientación es candidata para un orden de entailment. No se identifica aún
≼Ω con ≤L: falta dar una interpretación de programas/specs en L y justificar
que el orden algebraico conserva esta relación. El checker usa la consecuencia
Horn dirigida, no todos los modelos FOUR ni entailment algebraico completo.

La [comparación posterior con FOUR](refinement-four-order-comparison.md) define
una interpretación explícita de esos programas y encuentra contraejemplos a la
identificación con el orden punto a punto, aunque coincida la tabla de tres casos.

## Alcance de la aceptación

Una aceptación certifica sólo las obligaciones ground de Ω. No constituye una
demostración para todas las personas, cosas o PropositionTime, ni una autorización
para entregar una garantía universal a consumidores arbitrarios. Este experimento
no sustituye el certificado estructural anterior por un certificado universal
basado en muestreo. Una generalización requeriría pruebas o un dominio cerrado
declarado y controlado.

No se comprueban igualdad de los cierres completos: la implementación ampliada
debe poder conservar insured y entitlement. La propiedad relevante es suficiencia
de las consecuencias exigidas y provenance real de la implementación.

```sh
python3 scripts/test_safe.py --test experiments::refinement_tests::finite_refinement_accepts_exact_and_extra_but_rejects_missing_guarantee --test experiments::refinement_tests::finite_spec_order_points_from_stronger_guarantees_to_weaker --test experiments::refinement_tests::finite_refinement_records_scope_and_preserves_ground_substitutions
```

Todos los tests se ejecutan individualmente con los límites vigentes. El runner
genérico de fixtures no se presenta como checker de refinamiento.

## Resultados observados

| Programa candidato | Spec exigida | Obligaciones satisfechas | Resultado |
| --- | --- | --- | --- |
| Implementación exacta | protected | 8/8 | Acepta |
| Implementación con entitlement e insured | protected | 8/8 | Acepta |
| Implementación sólo insured | protected | 0/8 | Rechaza |
| Spec fuerte como programa | Spec débil | 8/8 | Acepta |
| Spec débil como programa | Spec fuerte | 8/16 | Rechaza |
| Implementación con entitlement e insured | Spec fuerte | 16/16 | Acepta |

Ambas specs satisfacen sus propias garantías sobre Ω. El checker también acepta
una garantía owns <= owns cuando la premisa ya soporta el objetivo, sin crear una
derivación ficticia. Tener un witness Derived no es requisito cuando el evento ya
está en la entrada.

Contraejemplo ground con una sola premisa para la implementación insuficiente:

```text
premisa: owns(alice, car) @ tau
real:    owns(alice, car) @ tau, insured(alice, car) @ after(tau)
falta:   protected(alice, car) @ after(tau)
```

La implementación ampliada justifica protected mediante completeProtection, cuyo
antecedente real es entitlement. La spec no aparece en sus registros de derivación.
Las sustituciones conservan owner, object y t para cada caso, incluidos tau y sigma.

Pasaron tres tests nuevos y dos regresiones seleccionadas, individualmente. Los
tres tests nuevos se repitieron tras ajustar el caso de objetivo ya soportado.
La compilación inicial alcanzó 561 MiB RSS y las ejecuciones unos 15 MiB. No se
ejecutó la suite completa y se conservaron los controles de 1 GiB, swap cero y
timeouts. El alcance observado es refinamiento Horn finito, no ≤L ya implementado.
