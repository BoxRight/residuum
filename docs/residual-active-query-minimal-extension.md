# Investigación de la extensión mínima de consulta residual activa

Este documento registra la etapa de prototipo. La
[implementación posterior de una API activa](residual-active-query-implementation.md)
cuenta ahora con tres tests individuales propios, después de resolver el coste
de compilación sin elevar límites. Los tests exitosos de esta etapa anterior
siguen siendo resultados del prototipo, no pruebas de esa API.

Rama: `experiment-legal-order-geometry`. Esta investigación continúa el
[contrato de activación](residual-activation-contract.md). No cambia el motor,
la API pública, las declaraciones, la resolución existencial ni los defaults.

## Dos operaciones necesarias

La API vigente recopila las declaraciones Rule de un Module y valida sus pruebas
mediante `replay_proof` e `instantiate_from_witness`. Esa validación certifica
instancias Horn, no la habilitación de las reglas en `close_program`.

Una extensión activa mínima necesita dos operaciones distintas:

1. **Generar candidatos conservando las guardas.** Para una asociación positiva
   de V con una regla r, un paso de r necesita tanto V como el body especializado
   de r. La búsqueda puede consultar esa condición como un producto de
   requisitos, manteniendo el vínculo con la asociación original.
2. **Certificar suficiencia operacional.** Para cada candidato H admisible,
   comprobar el objetivo en `close_program(K ∪ Leaves(H), Π)` con el módulo
   original y el mismo contexto operacional. Conservar las derivaciones reales
   obtenidas y mantener K intacto.

La segunda operación sola rechaza el candidato Unit incorrecto de ready/starts,
pero no descubre starts. La primera permite producir ese requisito; la segunda
comprueba que realmente activa el comportamiento que alcanza el objetivo.

No se propone cambiar retrospectivamente la garantía Horn de la API vigente.
La garantía activa debe nombrar su evaluador y distinguir candidatos de
respuestas certificadas:

\[
H\in CertifiedActiveQuery(\Pi,K,G)
\Longrightarrow G\in Close_{\Pi}(K\cup Leaves(H)).
\]

La implicación es inmediata de la comprobación cuando el cierre termina. No
afirma completitud, minimalidad, pertenencia al entailment FOUR ni capacidad de
resolver todas las variables de una cláusula.

## Prototipo exclusivamente en el harness

El verificador usa tres fixtures existentes, sin modificarlas:

- `examples/experiments/associated_program_waits.res`;
- `examples/experiments/sells_unit_associated_program.res`;
- `examples/experiments/sells_derived_associated_program.res`.

`guarded_search_view` construye una copia temporal del AST elaborado para la
búsqueda Horn. Obtiene la guarda de Verb.program, de la categoría del verbo y de
los parámetros tipados de la Rule referenciada. No contiene una tabla especial
starts → onStart ni aporta el objetivo como evidencia.

Si el body ya contiene la misma guarda estructural, no la duplica. Si el body
es Unit, la condición de búsqueda pasa a ser la guarda. En los demás casos es
guard & body. La polaridad de la guarda es Positive; su tiempo es el parámetro
de activación, no una constante inventada. Los orígenes se conservan en la copia.

Esta proyección es una representación experimental de las condiciones de
búsqueda, **no una reescritura del programa jurídico ni una nueva cláusula
declarada**. Nunca se usa esa copia como oráculo de activación: las hipótesis se
certifican exclusivamente mediante `close_program` del módulo original.

La prueba ResidualProof obtenida pertenece a esa vista proyectada. No debe
presentarse como si fuera un certificado del body original. El export conserva
separadamente la guarda, la prueba de búsqueda y los DerivationRecord reales.
Esta separación es esencial para una implementación futura de provenance.

## Resultados finitos

En la primera fixture el objetivo es done(juan) @ after tau:

| Evidencia inicial | Requisito del prototipo |
|---|---|
| Vacía | starts(juan) @ tau & signals(juan) @ tau |
| ready(juan) @ tau | starts(juan) @ tau |
| signals(juan) @ tau | starts(juan) @ tau |
| starts y ready, ambos en tau | Unit |
| starts en sigma, ready en tau | starts(juan) @ tau |
| ~starts y ready, ambos en tau | starts(juan) @ tau |
| starts, ~starts y ready en tau | Unit |

Cada candidato alcanza el objetivo en el evaluador activo original. Su
justificación real es onStart, con starts y ready como testigos ordenados.
El módulo, la evidencia y sus tiempos originales permanecen intactos.
P/~P coexisten sin retractación ni consecuencia global automática.

Para sellsProgram con body Unit, la consulta sin evidencia requiere sells,
en vez de tratar la definición como una regla incondicional. Aportar sells
hipotéticamente deriva give y conserva el trigger en su justificación.

Para el sells Derived de la tercera fixture, la hoja requerida es confirmedSale,
no sells. La regla confirm produce el trigger, que después activa sellsProgram.
Se conservan las dos ramas suficientes y las dos justificaciones reales de give:
sellsProgram y la especialización ordinaria juanSale. No se fusionan por producir
el mismo resultado.

El prototipo realizó **11 consultas ground**: siete en la primera fixture y
cuatro en las otras dos. Todas las respuestas fueron suficientes bajo el cierre
activo original. La regresión previa añade cinco contextos hipotéticos y su
comparación incremental. Estos son MODEL RESULT y observaciones del motor en
esas fixtures, no una demostración de completitud general.

## Frontera mínima de implementación futura

No hace falta modificar Rule, Program, el parser ni la representación de
existenciales para el fragmento anterior. Haría falta extender la infraestructura
residual, actualmente situada en `src/residual/program.rs`, en estos puntos:

- Seleccionar las reglas elegibles y registrar sus asociaciones y bindings,
  en lugar de tratar todas las declaraciones Rule como reglas obligatorias.
- Buscar la guarda positiva además del antecedente en cada paso asociado.
- Conservar una obligación/certificado de activación ligado al paso: verbo,
  trigger ground, comportamiento, RuleUse si corresponde, bindings y origen.
- Separar el certificado de búsqueda del testigo de suficiencia obtenido del
  evaluador activo. La presencia del objetivo certifica H, pero por sí sola no
  demuestra que la ruta de búsqueda propuesta fue la que produjo el objetivo;
  las afirmaciones sobre esa ruta necesitan contrastar sus pasos y testigos.
- No publicar una respuesta como suficiente si su comprobación operacional no
  concluyó o no derivó el objetivo.

En el forward, `prepare_programs` ya determina definiciones asociadas y reglas
ordinarias; `compile_program` conserva bindings compartidos y locales;
`finish_program_match` conserva el trigger. Una integración debe concordar con
esas operaciones. El prototipo no introduce otra política de activación.

Para una implementación pública, el certificado Horn actual no basta por sí
solo: debe acompañarse de metadata de activación o de un paso explícito de
certificado residual. No se decide aquí la forma definitiva ni se modifica
ResidualProof. Los certificados de consulta siguen sin DerivationTime; los
sellos de las derivaciones hipotéticas no se comparan entre ejecuciones.

## Restricciones deliberadas

El adaptador acepta sólo asociaciones a una Rule individual, con firma
compatible y asociación única por nombre de regla. Rechaza Program, defaults,
Effect asociados y múltiples asociaciones en vez de aproximarlos silenciosamente.
Es un prototipo de investigación, no una nueva API general.

Un Program con varios RuleUse exigiría proyectar cada uso compilado con su
scope, bindings y guarda, conservando la identidad de Program + Rule + módulo.
Dos verbos pueden asociarse al mismo comportamiento: esos pasos requieren
identidades de asociación separadas. No pueden colapsarse por el nombre de r.
Los parámetros locales no determinados por el goal siguen siendo una frontera
explícita; el experimento no enumera valores para ellos.

La búsqueda debe permitir que la hipótesis active un programa nuevo y que un
trigger Derived sea probado recursivamente. Congelar sólo Activated(K) perdería
ambas posibilidades. Si el objetivo ya es evidencia de K, no se exige demostrar
de nuevo una activación para aceptar su presencia.

La ausencia de respuesta certificada no debe presentarse como imposibilidad
matemática si hubo una frontera de parámetros, ciclo o presupuesto. Los límites
de SearchLimits vigentes acotan búsqueda, **no** la duración ni el tamaño del
cierre hipotético. `close_program` requiere un cierre finito y no ofrece aquí
un presupuesto interno de validación. Los experimentos se ejecutan en procesos
limitados por test_safe.py; un timeout deja el candidato sin certificar y no
constituye evidencia de que sea insuficiente. No se elevan límites.

Defaults quedan fuera del prototipo. Una consulta futura con defaults necesitaría
recibir el contexto seleccionado fijo, como precisa el contrato anterior. No se
introduce reevaluación de applicability, blocking, priority o consistency bajo H.

## Verificación reproducible

Tres tests individuales mediante `python3 scripts/test_safe.py`; no suite:

- guarded_residual_candidates_are_certified_by_the_unchanged_active_closure;
- guarded_search_preserves_unit_guards_and_derives_instead_of_abducing_derived_triggers;
- hypothetical_trigger_obligations_preserve_dynamic_activation_and_evidence.

Todos pasaron. Compilación: 5,3616 s, 414,48 MiB RSS. Tests: 0,0214 s / 15,18 MiB;
0,0414 s / 15,42 MiB; 0,0211 s / 15,54 MiB. Límites conservados: 1 GiB total,
sin swap, un trabajo de Cargo y un hilo. Informe:
`target/test-diagnostics/20261010T022107880051Z-718615`.

Verificador:
`src/bin/residuum/cli/legal_geometry_residual_activation_tests.rs`.
Exports:
[candidatos con guardas](legal-geometry/data/guarded-residual-candidates.tsv) y
[Unit y trigger Derived](legal-geometry/data/guarded-residual-trigger-kinds.tsv).

La conclusión del experimento es suficiencia primero: conservar guardas y
certificar operacionalmente resuelve ready/starts en el fragmento examinado.
Las doce variables pendientes de compliance permanecen fuera de este incremento.
