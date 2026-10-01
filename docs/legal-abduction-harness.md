# LegalAbduction: harness de tres escenarios

El programa Formal que usa el harness es la fixture existente
[sales_delivery_formal.res](../examples/legal/sales_delivery_formal.res).
Conserva su módulo `Legal` y la especialización `delivery = traditio(deliver)`;
no se modifica la fixture congelada ni se añaden `abducible` o `query` al parser.

El código está en [legal_abduction_tests.rs](../src/legal_abduction_tests.rs).
Recibe como configuración las lambdas `delivery` y `sales`, evidencia ground y
los nombres abducibles `agreesPrice` y `agreesObject`. Ambas declaraciones se
comprueban como `Seeded`. En este harness sólo se admiten hipótesis positivas
de esos verbos; un `Effect` requerido se resuelve mediante reglas, no se abduce.

Para el goal:

```text
do(alice, deliver, car, bob) @ after(after tau)
```

definimos:

```text
A = agreesPrice(alice, car, bob) @ tau
B = agreesObject(bob, car, alice) @ tau
```

| Evidencia en `Known` | Requisito lógico |
| --- | --- |
| `[]` | `A ⊗ B` |
| `[A]` | `B` |
| `[A, B]` | `I` |

`Known=[]` es el mínimo de conocimiento, no la unidad lógica. El producto de
cero factores de evidencia utilizada sí es `I`. En el último escenario, devolver
`I` significa que no queda requisito adicional; no significa que `Known` esté
vacío. Una valoración atómica FOUR asigna U a cada átomo ausente, mientras que
la valoración de la constante lógica `I` depende del producto elegido.

El requisito se representa mediante `Antecedent`, conservando producto y unidad.
No se inserta en `Known` ni recibe `DerivationTime`. Cada consulta conserva la
regla, el goal, las sustituciones y la evidencia utilizada. Si una consulta no
devuelve pendientes, el harness comprueba si la regla puede instanciar realmente
ese goal con la evidencia disponible; un head incompatible no se interpreta como
éxito. Esa comprobación produce un registro sin stamp y no ejecuta efectos.

La cadena parte de `delivery`, obtiene `give @ after(tau)` y lo consulta contra
`sales`. Ese requisito intermedio conserva `StateTime::Unresolved` como entrada
de su interpretación. La composición funciona por identidad lógica del evento,
sin resolver esa metadata ni eliminarla como workaround.

Después de obtener una explicación, el test construye un cierre hipotético con
copias de la evidencia y las hipótesis. Comprueba que derive `give` y el goal `do`
con sus tiempos estructurales y metadata operacional correspondientes. Ese cierre
usa su propio reloj de derivación; el programa y la evidencia reales permanecen
intactos. No se llama a `execute_effect` desde el harness.

Para cada hipótesis propuesta, el test vuelve a verificar la cadena omitiéndola.
Así comprueba minimalidad por inclusión en estos ejemplos positivos monotónicos.
No identifica esa preferencia con el orden lógico, ni implementa un selector de
elementos maximales en `≤_L`. La lista de eventos utilizada para validar hipótesis
no sustituye la estructura del requisito.

El harness está limitado a dos niveles de expansión de reglas y una sola solución;
una búsqueda más profunda o alternativas múltiples producen error explícito.
Cada uno de los tres escenarios realiza cuatro consultas de regla. Un cuarto test
comprueba que quitar un acuerdo de la lista abducible impide la explicación y que
`after(after(tau))` no puede colapsarse a `after(tau)`.

No implementa búsqueda general, joins de explicaciones, comparación algebraica,
consistencia programable ni sintaxis de consultas. Esas extensiones tienen
contratos propios; el harness prueba composición de los mecanismos existentes.

Los tres escenarios se verifican individualmente, junto con regresiones de
identidad lógica y runtime, usando el runner con 1 GiB total, sin swap y timeout:

```sh
python3 scripts/test_safe.py \
    --test legal_abduction_tests::empty_evidence_requires_both_agreements \
    --test legal_abduction_tests::price_evidence_requires_only_object_agreement \
    --test legal_abduction_tests::complete_evidence_requires_unit
```
