# Consulta residual activa: verificación individual

Rama `experiment-legal-order-geometry`. Se prioriza conservar la activación
dinámica; la API local query_residual y la resolución existencial no cambian.

## Contratos separados

Se añade `query_active_program_residual` y su variante
`query_active_program_residual_with_abducibles`. La consulta existente
`query_program_residual` conserva su contrato de cláusulas Horn explícitas;
su documentación ahora remite a la API activa para consultar comportamiento
ejecutable. No se cambia silenciosamente la garantía histórica de esa API.

La nueva operación recibe módulo, evidencia, objetivo ground, límites de
búsqueda y reloj externo de derivación. El reloj sella únicamente las
derivaciones hipotéticas obtenidas por close_program; los requisitos no adquieren
DerivationTime. La evidencia y el módulo se reciben mediante referencias
inmutables. No se ejecuta State.

La CLI y el runner de experimentos conservan sus rutas existentes. **Añadir esta
API no significa que toda llamada anterior a query o a la API Horn pase
automáticamente a tener semántica activa.** Su adopción por esos consumidores
queda separada, después de validar esta extensión.

## Construcción y certificación

1. Resolver las asociaciones de Verb.program a Rule o Program.
2. Excluir sus definiciones del conjunto obligatorio, como hace el forward.
3. Compilar los RuleUse asociados mediante compile_program, conservando sus
   parámetros compartidos/locales, capturas y especializaciones existentes.
4. Construir una vista privada de búsqueda con la guarda positiva de cada
   asociación. Cada paso exige guard & body; si body es I, exige guard. Una
   guarda estructural ya presente no se duplica.
5. Usar el buscador Horn existente sobre esa vista, con su misma política de
   abducibles y sus mismos errores para parámetros no determinados.
6. Para cada candidato admisible H, ejecutar el close_program original sobre
   K ∪ Leaves(H). Publicar la explicación sólo si ese cierre contiene el objetivo.

Los nombres privados de pasos proyectados incluyen la asociación: dos verbos
referidos al mismo comportamiento no se identifican accidentalmente. Se conserva
el producer real y el origen de Rule/Program/Module por separado. No se modifican
las cláusulas originales ni se trata la mera declaración/import como activación.

Cada `CertifiedActiveExplanation` contiene:

- `candidate`: requisitos y prueba de la vista de búsqueda;
- `activations`: triggers especializados, sustituciones, referencias reales y
  orígenes que explican los pasos proyectados;
- `hypothetical`: cierre y DerivationRecord reales del evaluador original.

La prueba proyectada no se presenta como si fuera una prueba del body original.
La presencia del objetivo certifica suficiencia de H; no afirma, por sí sola, que
la ruta propuesta sea la única ruta forward. El cierre conserva todas sus
justificaciones. No se comparan literalmente sellos derivedAt entre ejecuciones.

Si la validación termina y una explicación se devuelve, por construcción:

\[
G\in Close_{\Pi}(K\cup Leaves(H)).
\]

Éste es un argumento condicional de suficiencia, no una prueba de completitud
del buscador. La verificación individual se registra más abajo.

## Casos regresivos escritos

Se usan fixtures existentes, sin modificarlas:

- ready/starts: evidencia vacía, ready, signals, trigger en tiempo incorrecto,
  trigger negativo, trigger positivo, coexistencia de polaridades y restricción
  explícita de abducibles;
- sells con body I: el trigger sigue siendo obligatorio;
- sells Derived: se propone confirmedSale y se conserva la derivación confirm
  que obtiene el trigger, sin abducir sells;
- Chain con dos RuleUse: middle habilita completed en el mismo cierre, y las
  asociaciones starts/activated al mismo Program permanecen separadas.

Los tests comprueban requisitos, guardas, tiempos, sustituciones, orígenes y
registros reales; también immutabilidad de módulo/evidencia y ausencia de una
respuesta Unit cuando falta un trigger no abducible.

Los tres tests individuales pasaron después de compilar sin símbolos de
depuración, conservando los demás flags relevantes y los límites del runner.
No se afirma completitud general ni validación de la suite completa.

## Límites que permanecen

- Objetivos ground; todo parámetro de una regla candidata debe estar determinado
  por su head. No se enumeran existenciales ni las variables de compliance.
- Defaults no admitidos: no se diseña una selección ni una reevaluación bajo H.
- Effect no activa comportamiento asociado ni ejecuta un StateTransform.
- SearchLimits acota búsqueda; no acota internamente close_program. Como en esa
  API, el llamador debe garantizar un cierre hipotético finito. Un timeout no
  certifica ni refuta la suficiencia de una hipótesis.
- Ninguna preferencia, minimalidad o política global P/~P.

## Estado de verificación

Se formateó mediante `python3 scripts/test_safe.py --format`.

La verificación de tres tests individuales del target CLI se detuvo durante
compilación por `near_memory_limit`, antes de ejecutar cualquier test. El cgroup
alcanzó 805.449.728 bytes (~768,14 MiB), cruzando el umbral preventivo existente
de 768 MiB dentro del límite total de 1 GiB. No hubo OOM ni swap, no se elevaron
límites y no se repitió la compilación tras esa detención.

El log contiene un artefacto de biblioteca residuum generado, pero no un binario
CLI de tests completado. Eso **no** sustituye las regresiones de comportamiento.
En esa primera ejecución, la extensión no quedó validada.

Informe:
`target/test-diagnostics/20261010T023522662354Z-738005`.

Implementación: `src/residual/active.rs`.
Tests: `src/bin/residuum/cli/legal_geometry_residual_activation_tests.rs`:
`active_query_api_requires_the_matching_trigger_and_certifies_every_answer`,
`active_query_api_keeps_unit_guard_and_proves_derived_triggers`,
`active_query_api_preserves_shared_program_rule_uses_and_distinct_associations`.

La [investigación previa](residual-active-query-minimal-extension.md) sí contiene
resultados verificados del prototipo anterior. Esos resultados no deben
atribuirse a esta implementación posterior.

**Verificación posterior.** La [investigación de compilación](residual-compilation-cost.md)
permitió compilar con CARGO_PROFILE_TEST_DEBUG=0, sin elevar límites ni cambiar
opt_level=0, debug_assertions=true u overflow_checks=true. Los tres tests nuevos
pasaron mediante --binary, sin recompilar, en
`target/test-diagnostics/20261010T025239591029Z-757093`.
Esta es validación individual de la API activa, no completitud ni adopción por
la CLI. Una regresión antigua de Unit reveló además un fixture temporal
incoherente, documentado separadamente.

**Cierre posterior de la regresión y activación encadenada.** Se corrigió sólo
la fuente del test ground de Unit: declara `const tau : PropositionTime` y ya
no borra binders del AST elaborado. Pasaron ese test y las otras dos regresiones
de Unit/Tensor. Dos nuevos tests de Programs compuestos/activación encadenada y
compatibilidad Horn pasaron, junto con los tres tests activos anteriores.
Son ocho tests individuales; no se ejecutó la suite completa ni se amplió la API.

La cadena nueva contiene dos Programs con dos RuleUse cada uno, importados por
el escenario consumidor. Se verificaron 20 entradas contra el cierre de
referencia, incluyendo la multiplicidad de registros de provenance. Véase
[activaciones encadenadas](residual-active-query-chained-programs.md) para los
casos, informes y la limitación separada del replay Horn de cláusulas Unit
parametrizadas. La CLI y la consulta Horn no se modificaron.

La [revisión para integración](residual-active-query-integration-review.md)
documenta los estados del resultado, las ramas mixtas I/pending y las rutas de
consumidores pendientes. Añade una regresión verificada del contrato sin ampliar
existenciales ni cambiar selección, forward o la API Horn.
