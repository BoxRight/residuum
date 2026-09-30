# Residuum

El pipeline activo es:

```text
Formal → SurfaceAST → TypedAST → derivation/runtime
```

CNL se conserva como frontend experimental pausado. Sus tests históricos están
marcados `ignored`; Formal/CNL equality deja de ser requisito de cierre mientras
evoluciona el lenguaje. La línea futura es un drafter en dirección
`TypedAST → controlled natural language / legal prose`.

Para verificar cambios, usa el runner con límites de recursos:

```sh
python3 scripts/test_safe.py --test tests::close_uses_beta_reduced_delivery_rule
```

El runner requiere Linux, cgroup v2 y una sesión de systemd del usuario. Compila
sin ejecutar tests, con un trabajo de Cargo y timeout de 45 segundos. Después
ejecuta cada test seleccionado en un proceso y cgroup separados, con 1 GiB total,
sin swap, un hilo del harness y timeout de 15 segundos.

Se detiene ante un fallo, timeout, consumo de 768 MiB o crecimiento de al menos
64 MiB en un segundo cuando el test supera 128 MiB. Si no puede imponer los
límites, retorna error sin compilar ni ejecutar el binario. Los registros de exit
status, tiempo y pico RSS quedan en `target/test-diagnostics/`.

Puedes repetir `--test` para seleccionar varios tests, usar `--binary PATH` para
mantener un binario ya compilado, o usar `--compile-only`, `--fmt-check` y
`--format` para separar las fases. Sin `--test`, ejecuta todos los tests activos
individualmente; esa opción debe respetar la autorización vigente del usuario.

El runner limita sus propios procesos; una invocación directa de `cargo test`
no adquiere esos límites. No uses esa invocación para las verificaciones del
proyecto.
