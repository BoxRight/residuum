# Partición literal de LegalMonotonicCase: etapa v1.4

Esta etapa separa la biblioteca de referencia en estructura jurídica general y
una regla de adquisición patrimonial concreta. No revisa doctrina ni modifica
tipos, firmas, reglas, temporalidad, polaridades, efectos o política de defaults.
La clasificación procede del inventario aprobado para este encargo; la prueba
comprueba la extracción, no la corrección doctrinal de las reglas.

La rama parte de `v1-12h.1` (`de6b71b`). `main`, `v1` y `v1-12h.1` permanecen
intactos. [legal_monotonic.res](../examples/legal/modules/legal_monotonic.res)
sigue siendo la especificación de referencia: no se redirigen sus imports ni
se reemplaza el archivo. Sus consumidores originales también permanecen intactos.

| Archivo / módulo | Contenido local |
| --- | --- |
| [legal_general.res](../examples/legal/modules/legal_general.res), `LegalGeneral` | 76 declaraciones: 8 entidades, 10 constantes, 12 verbos Seeded, 16 Derived, 3 Effect y 27 reglas. |
| [legal_patrimonial.res](../examples/legal/modules/legal_patrimonial.res), `LegalPatrimonial` | Importa `legal_general.res`; declara `deliver`, `molest` y `complianceAsset`. |

La reunión de ambos módulos conserva las 79 declaraciones y las 28 reglas de
la referencia. Cada declaración pertenece a exactamente un módulo. Se conserva
el orden relativo de las declaraciones de cada partición; no se reordenan para
acomodar el elaborador. El inventario compara los textos de las declaraciones
literalmente y también sus SurfaceAST y declaraciones tipadas.

Las reglas generales son:

- `obligationCreatesRelation`, `prohibitionCreatesAbstention`, `historicalTransferEffect`;
- `filedObligation`, `filedPermission`;
- `obligationCompliance`, `rightCompliance`, `obligationViolation`, `rightEnforceability`;
- `prohibitedAct`, `prohibitedOmission`, `conflictingActEvidence`, `performanceDispute`;
- `violationLiability`, `existingAsset`, `patrimonyTitle`;
- `transferOccurrence`, `transferTitle`, `ownershipCapacity`;
- `grantedCapacity`, `capacityPower`, `enactLaw`, `statutoryObligation`;
- `permissionAnnulsProhibition`, `prohibitionAnnulsPermission`, `obligationAnnulsProhibition`, `prohibitionAnnulsObligation`.

Todas las entidades y los verbos, incluidos `give`, `do` y `notDo`, permanecen
en el módulo general. También permanecen `Doing`, `Abstaining`, `transfer`,
`enact`, `use` y las cinco constantes de autoridad. Ninguna regla general necesita
las dos constantes del módulo patrimonial ni `complianceAsset`.

Importar ambos módulos conserva el comportamiento de la biblioteca completa.
Importar solamente `legal_general.res` excluye deliberadamente la adquisición
por cumplimiento de cualquier deber: conserva `compliance`, pero no añade
`patrimonialAsset` mediante `complianceAsset`. La transmisión, la titularidad
por patrimonio existente y la responsabilidad por violación conservan sus reglas.
“Patrimonial” identifica aquí el régimen separado, no una nueva visibilidad
privada de símbolos; el sistema de imports mantiene su semántica existente.

## Oráculo de equivalencia

Los tests crean un grafo temporal con los mismos archivos y nombres de módulo.
En cada import de `legal_monotonic.res` sustituyen sólo esa línea por dos imports
reales de `legal_general.res` y `legal_patrimonial.res`. No concatenan archivos ni
reescriben cuerpos. El grafo temporal ni siquiera contiene la biblioteca antigua.
Los originales se cargan separadamente como oráculo.

| Archivo auditado | Consumidor que ejercita el cierre | Experimentos |
| --- | --- | ---: |
| `scenario.res` | El propio archivo | 16 |
| `sales_monotonic_baseline.res` | `scenario.res` | 16 |
| `sales.res` | `defaults_scenario.res` | 8 |
| `associated_program.res` | `sells_scenario.res` | 3 |
| `defaults_scenario.res` | El propio archivo | 8 |
| `default_joint_opposition_scenario.res` | El propio archivo | 1 |
| `default_comparison/partial_normal_sale_scenario.res` | El propio archivo | 1 |

Los tres módulos de declaraciones no tienen experiments locales. Para ellos se
comparan tanto las declaraciones y sus orígenes como los cierres de sus consumidores
existentes; importar no ejecuta experiments ajenos.

Cada comparación exige los mismos eventos, tipos proposicionales, tiempos de
proposición y transición, reglas productoras, testigos ordenados, sustituciones
y multiplicidad de justificaciones, siguiendo el oráculo de
`complete_monotonic_extraction_preserves_all_sixteen_closures_and_provenance`.
Se comparan además los contextos obligatorios y las extensiones de defaults,
incluidas selección, bloqueo, prioridad y Programs resultantes.

Sólo puede variar el valor concreto de `derivedAt`, sello del reloj externo.
Los orígenes se verifican separadamente: `deliver`, `molest` y `complianceAsset`
pasan exactamente a `LegalPatrimonial`; las otras 76 declaraciones pasan a
`LegalGeneral`. Los demás nombres de módulo y rutas relativas permanecen iguales.
El prefijo físico del directorio temporal se verifica mediante una correspondencia
exacta de rutas. Se comprueban también los orígenes Program + Rule de los productores
compilados; no se descarta provenance para obtener equivalencia.

Los siete informes quedan en `target/legal-partition-*.txt`. El runner registra
tiempo, memoria y resultado de cada test en `target/test-diagnostics/`.

## Prueba del módulo general aislado

[partition_general_scenario.res](../examples/legal/modules/partition_general_scenario.res)
importa exclusivamente `legal_general.res`. Usa `use`, distinta de `transfer`,
con autoridad y vigencia explícitas:

- `obligationFiled` + `~Act` deriva `obligation`, `violation`, `patrimonialLiability`
  y el Effect `do`, conservando la evidencia negativa;
- `obligationFiled` + `Act` deriva `obligation`, `compliance` y el Effect `do`,
  sin `patrimonialAsset`, `owns`, `transferred` ni `give`.

El test comprueba el conjunto exacto de verbos de cada cierre y que ningún Effect
inventa evidencia histórica. No ejecuta State ni modifica la semántica del motor.

## Verificación limitada

Toda compilación y ejecución usa `python3 scripts/test_safe.py`: 1 GiB total,
sin swap, un trabajo de Cargo, un hilo del harness y los mismos controles
preventivos y timeouts. La suite completa se ejecuta por separado para los
targets CLI y biblioteca; cada test se ejecuta en un proceso limitado propio.
Los nueve tests CNL pausados siguen ignorados.

Resultado: inventario literal y tipado correcto, siete equivalencias correctas,
prueba negativa correcta y suite completa de 277 tests activos (60 CLI + 217
biblioteca) sin fallos. Una primera fase de compilación CLI fue detenida por el
umbral preventivo; la fase restante terminó reutilizando artefactos ya compilados,
sin OOM ni modificación de controles. No hubo dificultades semánticas pendientes.

```sh
python3 scripts/test_safe.py --test-target cli
python3 scripts/test_safe.py --test-target lib
python3 scripts/test_safe.py --fmt-check
```

## Identidad de los archivos

SHA-256, sobre los bytes completos de cada archivo:

```text
24655c98ede7b128b8cd82df3a4473c114428b987bb0fe6a6d370c2b37f5575e  examples/legal/modules/legal_monotonic.res
639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc  examples/legal/modules/legal_general.res
f7daf4d5eb937a7fde8534dba5618acc50d5aaa5b05ff86c9656892eb3a03fbf  examples/legal/modules/legal_patrimonial.res
```

Los cambios de tipos o reglas pertenecen a etapas posteriores. No se actualiza
`~/12hectareas` ni se hace push como parte de esta partición.
