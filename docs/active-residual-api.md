# Consulta residual activa

`query_active_program_residual` busca requisitos para un objetivo ground y
certifica cada respuesta con el `close_program` original sobre evidencia
hipotética. La variante `with_abducibles` restringe las hipótesis por verbo y
polaridad. Ambas reciben un reloj externo; no alteran Known ni ejecutan State.

La consulta histórica `query_program_residual` conserva su firma y su semántica
de cláusulas Horn explícitas. La API activa incorpora las guardas positivas de
`Verb.program`; una definición con cuerpo `I` sigue necesitando su trigger.
La CLI mantiene sus llamadas históricas: exportar esta API no cambia `query`.

## Resultado

| Estado | Representación |
|---|---|
| Rama certificada sin hipótesis pendientes | `candidate.required = Unit` |
| Rama con hipótesis pendientes | Antecedente con eventos/conjunciones |
| Ninguna explicación certificada | `required = None`, cero ramas y `failure` |
| Búsqueda o validación no completada | `Err`, sin respuestas truncadas |

Un resumen `Join` puede conservar ramas `Unit` y ramas pendientes. No se elimina
su multiplicidad ni se introduce una preferencia. Ausencia de explicación en
este fragmento no significa imposibilidad lógica general.

Cada `CertifiedActiveExplanation` conserva el candidato de la vista con guardas,
las activaciones especializadas y el cierre hipotético original. Los productores
reales de Rule/Program/Module se distinguen de los nombres privados de búsqueda.
La garantía, condicionada a que termine el cierre, es:

\[
G\in Close_\Pi(K\cup Leaves(H)).
\]

No se afirma completitud general, ni que la ruta residual sea la única prueba
forward del objetivo.

## Límites

- Objetivos y evidencia correctamente tipados y ground, suministrados por el
  llamador; la API no reelabora los valores externos.
- Parámetros de cada regla determinados por su cabeza; no hay enumeración
  existencial nueva.
- Triggers positivos Seeded/Derived; Effect no activa programas asociados.
- Defaults rechazados; no cambia su selección ni se introduce consistency.
- `SearchLimits` limita la búsqueda, no el cierre hipotético. Su terminación es
  responsabilidad del llamador y del entorno de ejecución limitado.
- Derivación y ejecución de efectos permanecen separadas.

## Regresiones independientes

`src/residual/active/tests.rs` verifica la API usando las fixtures fuente de
activación y programas compuestos, sin depender del harness jurídico:

1. Guardas, triggers negativos/en tiempo incorrecto y compatibilidad Horn.
2. Estados pendientes, `Unit`, ausencia, mezcla de ramas y límites agotados.
3. Imports, programas encadenados, productores reales y multiplicidad completa
   de justificaciones frente a `close_program_reference`.

Se conserva la regresión ground de `I` declarando la constante temporal en
fuente y comprobando `TimeExpr::At`, sin borrar parámetros después de elaborar.
La compilación y cada test se ejecutan mediante `scripts/test_safe.py` con los
límites existentes. La revisión experimental ampliada y sus certificados se
publican separadamente de esta API.
