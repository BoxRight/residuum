# Preparación de integración de la consulta residual activa

Revisión en `experiment-legal-order-geometry`. La etapa de corrección de
activación queda verificada en su fragmento experimental. No se hace merge,
no se cambia main/v1 y no se amplían existenciales. La biblioteca ya exporta
la API activa; adoptarla en un consumidor es un cambio separado.

## Contratos distintos

| API | Resultado | Garantía | Reloj |
|---|---|---|---|
| query_residual | Vec de ResidualRequirement | Reducción local de una Rule | Ninguno |
| query_program_residual | ProgramResidualQuery | Replay de cláusulas Horn explícitas | Ninguno |
| query_active_program_residual | ActiveProgramResidualQuery | close_program hipotético del módulo original | Externo |

Las variantes with_abducibles sólo restringen las hipótesis por verbo/polaridad.
No hay alias ni conversión que haga intercambiables las dos consultas sobre
módulos. Las firmas históricas siguen intactas.

La consulta activa exige las guardas de Verb.program y conserva productores
reales, Program/Rule/Module, antecedentes ordenados y sustituciones canónicas.
Su publicación se condiciona a que el cierre original derive el objetivo; no
afirma que la ruta residual sea la única ruta forward. Las hipótesis no entran
en Known real ni se ejecuta State.

La consulta Horn recoge las declaraciones Rule, incluidas definiciones referidas
por Verb.program, sin comprobar activación. Sus certificados no son certificados
de ejecución de un Program. La API local conserva otra ambigüedad histórica:
un Vec vacío puede significar head incompatible o antecedente satisfecho. No se
rediseña esa API para trasladarle los estados de las consultas sobre módulos.

## Estados del resultado activo

Para un retorno Ok producido por la API:

| Estado | required | explanations | failure |
|---|---|---|---|
| Sin explicación certificada | None | Vacío | Some |
| Explicación sin hipótesis pendientes | Some(Unit) | No vacío | None |
| Explicaciones pendientes | Some(Event/Tensor/Join) | No vacío | None |
| Ramas satisfechas y pendientes | Join con Unit y otros requisitos | Ambas clases | None |

Unit significa que esa rama no necesita hipótesis adicionales, no Known vacío
ni necesariamente un objetivo ya explícito en la evidencia. Un cuerpo Unit de
una regla asociada no exime de acreditar su trigger.

El consumidor debe examinar cada `candidate.required`: comprobar sólo
`required == Some(Unit)` no detecta una rama Unit dentro de Join. No se absorben
las alternativas pendientes ni se eliminan pruebas por preferencia.

None significa que este procedimiento no certificó una explicación en su
fragmento, no imposibilidad lógica general. NotAbducible señala ausencia de
explicación admitida; NotForwardSupported señala candidatos sin certificado
forward. Ninguno equivale a I.

Ciclos, parámetros no determinados, límites agotados y errores de validación
devuelven Err. No deben convertirse en None/I o en una búsqueda negativa completa.
No se publican resultados truncados. Esto no introduce un nuevo enum.

## Precondiciones conservadas

- El llamador proporciona eventos ground y correctamente tipados. La validación
  de firma no reelabora argumentos externos.
- Los parámetros de cada regla candidata deben estar determinados por su head.
  No se enumeran variables existenciales ni dominios para compliance.
- Sólo triggers positivos Seeded/Derived activan asociaciones. EvidentialNot
  sigue siendo evidencia explícita, no ausencia ni desempate.
- Defaults siguen rechazados. Effect no activa comportamiento asociado ni
  ejecuta State en esta consulta.
- SearchLimits acota búsqueda, no el cierre hipotético. El llamador debe
  garantizar que cada close_program termine; timeout no es None.
- Se conservan alternativas y multiplicidad de pruebas sin minimalidad ni
  identificación con el orden algebraico.

## Adopción pendiente en los consumidores

Los exports en src/lib.rs y src/residual.rs ya hacen disponible la API activa.
Las rutas históricas no cambiaron:

- Command::Query delega en run_experiments.
- run_experiments, en src/experiments.rs, utiliza query_program_residual y
  rechaza módulos con Program.
- ExperimentResult.residual sigue siendo Option<ProgramResidualQuery>.
- run/explain mantienen sus rutas forward/defaults.

Por tanto, exportar la API no hace que `residuum query` ejecute el experimento
de Programs. La adopción exige un adaptador explícito que distinga ambos
contratos, conserve certificados activos, use reloj externo y mantenga controles
de terminación/recursos. No basta sustituir una llamada y perder campos del
resultado. Esta revisión no decide sintaxis, flag o comando para ese adaptador.

## Verificación individual

Base previa: ocho tests individuales sobre Unit/Tensor, ready/starts, triggers
Derived, Program compartido y Programs encadenados. Véase
[activaciones encadenadas](residual-active-query-chained-programs.md).

La regresión nueva usa associated_multi_rule_program.res, sin cambiarla:

1. Evidencia vacía: explicaciones pendientes certificadas.
2. starts conocido: Unit junto con hipótesis requests, cuatro pruebas separadas.
3. Objetivo evidenciado: Unit sin requisitos pendientes.
4. Evidencia vacía y sin abducibles: None, cero explicaciones, NotAbducible.
5. Presupuesto insuficiente: Err, sin resultado parcial.

Comprueba required/failure frente a la existencia de explicaciones, suficiencia
de cada certificado e inmutabilidad del módulo y la evidencia.

Pasaron tres tests individuales: contrato nuevo y regresiones de Programs
encadenados/compatibilidad Horn. Se compiló una vez con CARGO_BUILD_JOBS=1 y
CARGO_PROFILE_TEST_DEBUG=0; después se ejecutaron mediante test_safe.py --binary
con nombres exactos. Límites intactos: 1 GiB, sin swap, umbral preventivo 768 MiB,
un trabajo Cargo y un hilo del harness.

Informe: `target/test-diagnostics/20261010T035643626110Z-831518`.
El contrato nuevo terminó en 0,0413 s con 15,50 MiB RSS.

También pasó individualmente la regresión histórica de la CLI
`query_reports_residual_requirements_and_the_four_counterexample` en
`target/test-diagnostics/20261010T035925162732Z-834548`. Confirma las salidas
previas de query sin adoptar silenciosamente la API activa. Pasó fmt-check.

No se ejecutó la suite completa. Son cuatro tests individuales en esta revisión
y diez tests distintos cubiertos entre ésta y la etapa previa, no una afirmación
de validación global ni de completitud.

Datos: [active-residual-result-contract.tsv](legal-geometry/data/active-residual-result-contract.tsv).

## Regreso a las fronteras geométricas

El siguiente contraste mantendrá programa, universo finito, evidencia, objetivo
y abducibles iguales. Los soportes mínimos por inclusión se calcularán mediante
close_program real, y se compararán con las hojas de las respuestas activas sin
alterar las fórmulas ni su multiplicidad de provenance en el motor.

Se deben clasificar por separado suficiencia, cobertura y ausencia de minimalidad.
Una respuesta suficiente puede tener un soporte no mínimo; un soporte mínimo
geométrico puede quedar fuera de la consulta por variables pendientes. No son
la misma discrepancia. La geometría estudia el cierre ejecutable, no la vista
Horn sin guardas. No se amplían existenciales para hacer coincidir tablas.

**Contraste posterior completado:** el [oráculo jurídico de la API activa](legal-ontology-active-residual-oracle.md)
verifica 16 fronteras controladas con cierre independiente del buscador y
clasifica los 146 objetivos de las ontologías completas. No amplía existenciales
ni convierte esos resultados en completitud general. La biblioteca exporta la
API activa y mantiene la consulta Horn; la adopción por la CLI sigue separada.
