# Revisión de integración de API activa e investigación jurídica

Preparación desde `experiment-legal-order-geometry`, con base local
`7b14cb8ac86d07c8d101dc36ca9f1161580f6c22`. Se separan dos commits:

1. API residual activa, contrato y regresiones de biblioteca independientes.
2. Harness jurídico, fixtures, informes, fuentes reproducibles y certificados
   experimentales. No implementa aprendizaje normativo ni Candidate Elimination.

La integración utiliza merge explícito. No cambia el tag `v1`, cuyo objeto
permanece `f6a83e33fa1edb606748a47394e2bde8b4ce635d`. El push se dirige a
`origin/main`, sin force ni actualización de tags.

## Compatibilidad comprobada

Se ejecutaron **36 tests individuales**, ocho de biblioteca y 28 de CLI, con
`python3 scripts/test_safe.py`, `CARGO_BUILD_JOBS=1` y
`CARGO_PROFILE_TEST_DEBUG=0`. No se ejecutó la suite completa. CNL permanece
pausado y sus tests ignorados no son criterio de integración.

| Área | Verificación seleccionada |
|---|---|
| API activa | Guardas, triggers negativos/en otro tiempo, resultados Unit/pendientes/ausencia, límites, programas encadenados |
| Consulta Horn | Regresión ground de I, oráculo finito, identidad de bindings, límites y currying |
| CLI histórica | run del expediente, query residual/FOUR y explain con multiplicidad |
| Imports | Diamantes, experiments locales, ámbitos, rutas, ciclos, duplicados, límites y beta |
| Effects | Records Doing/Abstaining, coexistencia/idempotencia, derivación separada de ejecución |
| Programs/defaults | Cuatro ventas, selección completa, aislamiento, imports pasivos y provenance |
| Tiempo | Constantes en argumentos/índices/transiciones; beta de records y binders locales |
| Ontología | Equivalencia de las 16 clausuras originales; inventario general/patrimonial y referencia |
| Investigación | Expedientes Riggs/tipados, fronteras activas, alcance residual y generación estructural |

La compilación de biblioteca alcanzó 521,55 MiB RSS y la de CLI 402,28 MiB.
El test más largo de la selección terminó en 5,37 segundos. No hubo OOM,
timeouts ni controles desactivados. Se conservaron 1 GiB total, sin swap, un
trabajo de Cargo y un hilo del harness. El formato final pasó mediante el mismo
runner. Nombres exactos, resultados y recursos:
[verification.json](legal-geometry/integration/verification.json).

## Portabilidad de las fuentes

Las fixtures experimentales importaban el repositorio hermano `12hectareas`.
Para que el checkout sea reproducible se incluyen copias literales en
`examples/experiments/legal_geometry/ontology_reference/`:

| Fuente | SHA-256 |
|---|---|
| legal_general.res | `639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc` |
| legal_patrimonial.res | `f7daf4d5eb937a7fde8534dba5618acc50d5aaa5b05ff86c9656892eb3a03fbf` |

Las dos copias son byte a byte iguales a sus fuentes. Sólo se redirigen cuatro
imports experimentales y las rutas de referencia de dos verificadores y del
script de análisis. Los módulos conservan sus nombres, firmas, reglas y efectos.
La referencia `examples/legal/modules/legal_monotonic.res` permanece intacta.

Los archivos de origen operacional ahora apuntan a la carpeta local. La igualdad
de declaraciones se verifica separadamente de esos orígenes. Los certificados
históricos pueden conservar la ruta externa usada en su ejecución; no se
reescriben como si hubieran sido producidos en la ubicación nueva. Las
revalidaciones actuales y los hashes permiten identificar el mismo contenido.
No se incorpora ningún expediente concreto de `12hectareas`.

Los certificados TSV conservan los bytes y los campos vacíos finales; algunas
tablas usan CRLF. `.gitattributes` evita conversiones de fin de línea en esas
tablas y en las copias ontológicas verificadas. El diagnóstico de espacios de
Git señala esos finales y tres líneas vacías terminales de fixtures históricas;
se preservan para no cambiar columnas ni hashes por una limpieza cosmética.
El formato de Rust sí se verifica con el runner.

## Límites conservados

La API activa es adicional: la consulta Horn y la CLI mantienen sus contratos.
No se amplían existenciales, defaults, consistency, prioridades ni ejecución
automática de State. SearchLimits no acota internamente el cierre hipotético;
su terminación sigue siendo una precondición del llamador.

Los teoremas de geometría están condicionados a las hipótesis declaradas y las
comprobaciones del harness son finitas. Las propuestas de reglas no modifican
la ontología de referencia ni resuelven automáticamente una exclusión sucesoria.

La generación estructural certifica un subperfil. El contrato amplio necesita
acotar o normalizar las inserciones de I antes de afirmar finitud de AST; el
contraejemplo queda documentado. Publicar esos experimentos no aprueba sus
revisiones normativas ni presenta como completo un generador del lenguaje.
