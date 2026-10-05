# LegalMonotonicCase: referencia congelada de v1

El tag `v1` conserva esta ontología y sus escenarios de regresión. No se mueven
ni reescriben reglas al congelarla: la extracción modular ya está verificada.

| Archivo | Responsabilidad |
| --- | --- |
| [legal_monotonic.res](../examples/legal/modules/legal_monotonic.res) | Ontología y reglas generales no específicas de compraventa; módulo LegalMonotonicCase, sin datos de escenarios ni experiments. |
| [sales_monotonic_baseline.res](../examples/legal/modules/sales_monotonic_baseline.res) | Reglas de compraventa monotónica del baseline, con reciprocity Seeded. |
| [scenario.res](../examples/legal/modules/scenario.res) | Datos concretos y los 16 experiments; importa el baseline monotónico. |
| [case_monotonic_formal.res](../examples/legal/case_monotonic_formal.res) | Referencia equivalente inline para regresión. |

Sales y SaleDefaults son capas distintas de esta referencia: selectedReciprocity
es Derived y reciprocity es Seeded, sin puente automático. Las reglas de
compraventa y la selección de paquetes no pertenecen a la ontología general.
obligationCreatesRelation permanece en LegalMonotonicCase, fuera de los RuleUses
de NormalSale y RetentionSale.

Se conservan los planos evidencia histórica Act/~Act, conclusiones jurídicas
y Effects conocidos, y State tras ejecución explícita. La oposición no borra
hechos. do/notDo usan los campos ordinarios de Relation, incluido mode
Doing/Abstaining; ninguna ejecución acredita automáticamente un Act.

La comparación de los 16 escenarios exige eventos, tipos y tiempos, reglas,
testigos ordenados, sustituciones y multiplicidad de justificaciones iguales.
Sólo el valor concreto del sello derivedAt puede variar; los orígenes de módulo
se verifican separadamente.

## Identidad del snapshot

SHA-256 de los archivos, sin modificaciones durante el cierre de v1:

```text
24655c98ede7b128b8cd82df3a4473c114428b987bb0fe6a6d370c2b37f5575e  examples/legal/modules/legal_monotonic.res
71ce63eb0ba3a98c846c5532c39898500e01699d3c7d72104db488c7dfd23269  examples/legal/modules/sales_monotonic_baseline.res
47808ce3a22f70e222f45ad9d15e4dcd3cd8b25e82e4d39e4d3f04b4f5e73a8a  examples/legal/modules/scenario.res
e98d59ae5af18c7a898e4f664fc485312b43267ab104b3e4055e88b02a68f891  examples/legal/case_monotonic_formal.res
```

Smoke test de equivalencia, individual y limitado:

```sh
python3 scripts/test_safe.py --test-target cli \
  --test cli::tests::extraction::complete_monotonic_extraction_preserves_all_sixteen_closures_and_provenance
```

Un cambio posterior a esta referencia requiere declarar una revisión semántica
o conservarla y crear otra versión. El próximo proyecto no necesita ampliar
Legal para usar el núcleo de propósito general de Residuum.
