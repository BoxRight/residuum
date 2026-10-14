# Verificación individual

Compilación y tests exclusivamente mediante `scripts/test_safe.py`.
1 GiB total, swap 0, un trabajo de Cargo, un hilo; 45 s de compilación y 15 s por test.
No se ejecutó la suite completa.

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 python3 scripts/test_safe.py --test-target cli --compile-only
python3 scripts/test_safe.py --test-target cli --binary target/debug/deps/residuum-f5f0059bb1094ae1 --test NOMBRE_EXACTO
```

| Test exacto (`cli::tests::legal_geometry::riggs::…`) | Tiempo | Pico RSS | Pico cgroup | Informe |
|---|---:|---:|---:|---|
| `rule_learning::generated_relational_rules_learn_from_historical_outcome_preserving_reference` | 1.934 s | 16.03 MiB | 18.54 MiB | `20261011T231554767493Z-278718` |
| `rule_learning::learned_rule_partitions_and_counterfactual_questions_preserve_indetermination` | 1.587 s | 15.30 MiB | 11.50 MiB | `20261011T231602218352Z-278766` |
| `rule_learning::candidate_theories_check_geometry_extreme_bases_and_seed_frontiers` | 1.913 s | 18.38 MiB | 21.79 MiB | `20261011T231609083308Z-278942` |
| `rule_learning::learned_rules_generalize_bindings_and_preserve_distinct_justifications` | 1.057 s | 15.52 MiB | 11.70 MiB | `20261011T231633059948Z-279299` |
| `literal_reference_and_typed_counterfactuals_preserve_complete_provenance` | 0.407 s | 15.48 MiB | 10.50 MiB | `20261011T231646754529Z-279506` |
| `conversation::external_answers_update_argument_versions_without_conflating_premises` | 0.408 s | 15.17 MiB | 10.75 MiB | `20261011T231650880374Z-280366` |

Pasaron los seis tests de la tabla. La última compilación pasó en 3.501 s, 398.67 MiB RSS.

Una compilación intermedia rechazó tres referencias de variables fuera de ámbito
en el exportador de testigos geométricos del harness nuevo. Se corrigió ese bloque
y se recompiló; no se cambió ninguna operación del motor. Se conserva el registro
de esa compilación fallida en verification.json.

El snapshot inicial contiene 263 archivos. Todos mantienen sus
SHA-256 salvo el test padre de Riggs, donde únicamente se registra el submódulo
nuevo; al retirar ese registro, su SHA-256 vuelve exactamente al inicial.
Se verificaron también los refs de main y v1 y la rama experimental.
No hubo commit ni merge.
