# Residuum v1: alcance congelado

Fecha: 2026-10-05. El tag Git `v1` identifica esta versión del lenguaje y sus
fixtures de referencia. Es un hito semántico; la versión del paquete Cargo sigue
siendo `0.1.0`. No se publica un paquete ni se declara estabilidad general de API.

## Dentro de v1

| Componente | Alcance |
| --- | --- |
| Horn y Close | Núcleo monotónico, conjunción `&`, unidad `I` y cierre por fixpoint. Retornar requiere estabilización; no hay terminación universal. |
| EvidentialNot | `~P` requiere evidencia explícita. `P` y `~P` coexisten; no generan por sí solos explosión, retractación, bloqueo ni `inconsistent`. |
| Residual y abducción | `-o`/`⊸`, requisitos, alternativas y suficiencia certificada en el fragmento documentado. No incorpora hipótesis a Known ni decide una explicación preferida. No integra consultas generales sobre Program o activación asociada. |
| Incrementalidad | Adición de seeds con reglas fijas en el fragmento especificado; delta lógico y nuevas justificaciones separados. No se presupone aditividad de cambios independientes. |
| Effect y State | Un Effect derivado puede pertenecer a Known. Ejecutarlo es otra operación, funcional sobre snapshots; no genera automáticamente Act ni ~Act. |
| Rule | Una cláusula parametrizada, β-reduction y aplicación parcial por prefijos de constantes tipadas. No es una función arbitraria de orden superior. |
| Módulos | `import "./relative/path.res"`, dependencias transitivas sin duplicación, errores por ciclos/rutas/símbolos/colisiones, origen de declaraciones y experiments locales. |
| Program | Parámetros, RuleUses con bindings y módulo de definición; aplicación parcial, sustitución en todos los miembros y un único cierre con el entorno. Rule sigue siendo una cláusula. |
| Verb.program | Asociación a Rule o Program, activada por instancias positivas conocidas de Seeded/Derived. No ejecuta StateTransform. |
| Defaults | Seleccionan RuleRef o ProgramRef completo mediante applicability → blocking → priority. Bloqueo global de lambda; entorno y miembros elegidos participan juntos en Close. Consistency sigue NotChecked. |
| Provenance | Regla, testigos, sustitución y DerivationTime; identificación Program + Rule + Module. El valor concreto de derivedAt depende del reloj de la ejecución. |
| CLI | `run`, `query`, `explain` sobre experimentos Formal. run/explain derivan; query respeta las restricciones de residual/FOUR existentes. |

El pipeline activo es Formal → SurfaceAST → TypedAST → derivation/runtime.
CNL permanece conservado y pausado. El launcher y la verificación requieren
Linux/cgroup v2 y systemd de usuario: 1 GiB total, sin swap y controles preventivos.

## Fuera de v1

- Semántica definitiva de consistency, competes, familias/decisiones de defaults
  y resolución completa de conflictos entre programas. La oposición experimental
  no excluye candidatos ni alimenta el selector.
- `!`, eliminaciones/retractaciones, reglas dinámicas e incrementalidad general
  de asociaciones; optimizaciones de matching.
- Completitud residual general, identificación formal completa con FOUR y
  afirmaciones de Turing-completeness.
- Ejecución automática de State desde run, CNL como frontend activo y un drafter.

## Referencias y disciplina

[LegalMonotonicCase](legal-monotonic-v1.md) es la referencia jurídica modular
congelada, no una definición universal del lenguaje. Nuevos proyectos pueden
usar Residuum sin importar Legal. Las extensiones posteriores deben declarar
su cambio de alcance; los resultados de un proyecto no deben redefinir v1
silenciosamente.

La [auditoría del núcleo](monotonic-core-final-audit.md), el
[contrato de Program/defaults](program-defaults.md) y el
[experimento de consistency](program-consistency-contract.md) distinguen
implementación limitada, resultados finitos y argumentos matemáticos. Los tests
seleccionados no constituyen una certificación general ni una ejecución de toda
la suite.

Para este cierre pasaron cinco tests individuales del target CLI mediante
test_safe.py: run del expediente, query residual/FOUR, equivalencia de los 16
escenarios modulares, los cuatro casos de selección de Program y derivación/
ejecución explícita de efectos. También pasaron fmt-check y diff-check.
Registros locales: target/test-diagnostics/20261005T203326365793Z-104818/.
