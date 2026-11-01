# Coste de compilación residual y presupuesto de verificación

Investigación en `experiment-legal-order-geometry`. No se modificaron límites,
Cargo.toml, el runner, el motor forward ni los genéricos de active.rs para
obtener estos resultados. Las configuraciones de Cargo fueron variables de
entorno temporales, no una nueva configuración persistente del proyecto.

## Qué estaba configurado

test_safe.py ya pasa `--jobs 1` a Cargo. Los tests se ejecutan individualmente
con `--test-threads=1`, en cgroups separados. Se conservan 1 GiB total,
sin swap, detención preventiva en 768 MiB, 45 segundos para compilación y
15 segundos por test. `CARGO_BUILD_JOBS=1` se añadió explícitamente durante el
diagnóstico, sin ampliar el paralelismo permitido.

Los artefactos Cargo anteriores muestran debuginfo=2, opt_level=0,
debug_assertions=true y overflow_checks=true. Con
`CARGO_PROFILE_TEST_DEBUG=0`, los nuevos artefactos de biblioteca y CLI muestran
debuginfo=0 y los otros tres valores permanecen iguales. No se hizo un build
release ni se desactivaron aserciones o comprobaciones de overflow.
Se pierde información de depuración para herramientas externas, no las
aserciones empleadas en estas pruebas. Eso no demuestra equivalencia de todos
los comportamientos posibles de cualquier código sensible al perfil.

## Compilación y ejecución separadas

Se compiló cada target con `--compile-only`. Después se pasó el binario producido
a `--binary`, con nombres exactos de tests individuales; esas ejecuciones no
invocaron Cargo.

| Compilación | Debug | Tiempo | Pico del cgroup | Resultado |
|---|---:|---:|---:|---|
| Biblioteca, intento anterior | 2 | 6,65 s | 770,09 MiB | Detención preventiva |
| CLI, extensión activa, intento anterior | 2 | 8,58 s | 768,14 MiB | Detención preventiva |
| CLI | 0 | 15,84 s | 733,03 MiB | Completada |
| Biblioteca | 0 | 11,39 s | 723,97 MiB | Completada |
| CLI, compilación de diagnóstico en target temporal nuevo | 0 | 17,28 s | 745,85 MiB | Completada |

La última compilación usó `CARGO_TARGET_DIR=/tmp/residuum-profile-debug0` para
observar trabajo real con dependencias compiladas nuevamente, sin borrar ni
alterar la caché existente. No compara un artefacto cached con una compilación
activa para atribuir costes. Los intentos abortados no son ejecuciones completas,
y las cachés/contextos difieren: la tabla no permite calcular un porcentaje
universal de mejora ni comparar directamente los tiempos de terminación.

Informes:

- `target/test-diagnostics/20261010T025112078900Z-755087`;
- `target/test-diagnostics/20261010T030646116097Z-769718`;
- `target/test-diagnostics/20261010T031139053082Z-775162`.

## Procesos y caché de archivos

Los primeros intentos de monitorización no identificaron procesos: el /proc del
sandbox sólo veía su propio espacio de procesos, y otra observación no encontró
el scope durante su ventana. No se infieren nombres de procesos de esos intentos.

La medición válida lanzó observador y runner en la misma sesión y recorrió los
miembros visibles del cgroup cada 50 ms. Hubo como máximo un rustc simultáneo
en las muestras. Los mayores high-water RSS observados fueron:

| Proceso | RSS high-water observado |
|---|---:|
| rustc: dependencia zerocopy | 394,11 MiB |
| rustc: biblioteca residuum | 369,84 MiB |
| rustc: target CLI de tests | 376,18 MiB |
| ld: enlace final | 120,02 MiB |

El compilador es el mayor consumidor individual en esta medición. El enlace
también aporta consumo, y el pico total no es el RSS de un único proceso.
En la mayor muestra del cgroup hubo aproximadamente 390 MiB de file cache y
340 MiB de memoria anónima. La caché pertenece al presupuesto de memory.max.
No se suman los máximos individuales, que pueden corresponder a momentos
distintos y compartir páginas. Tampoco se atribuye retrospectivamente un proceso
exacto al instante de las detenciones anteriores, que carecían de este muestreo.

Datos: [residual-compile-profile.json](legal-geometry/data/residual-compile-profile.json).
El pico cgroup del informe del runner es más preciso que las muestras de 50 ms.
El campo peak_rss_bytes del informe no basta para identificar por nombre al
descendiente que consumió memoria, especialmente tras terminar el grupo.

## Revisión estática de active.rs

La extensión añade dos funciones públicas genéricas sobre DerivationClock,
forwardeando el reloj a close_program. Usa estructuras de datos existentes y
iteradores finitos; no introduce tipos recursivos sin indirección ni una nueva
gramática Chumsky. Las llamadas de sus tres tests usan el mismo tipo Clock.
La recursión de certificados sigue la estructura ya existente de ResidualProof.

No se midió una atribución por función, instancia genérica o fase de rustc.
Por tanto, no se demostró que active.rs, sus iteradores o sus monomorfizaciones
sean la causa dominante. El fallo de biblioteca previo ocurrió antes de añadir
esa implementación. No se aplicaron .boxed(), cambios al AST ni reescrituras
por una hipótesis de coste no verificada.

## Tests y discrepancia separada

Los tres tests nuevos de la API activa pasaron con el binario CLI ya compilado:

- trigger correcto, tiempos, polaridades y abducibles;
- guarda de un body I y prueba de un trigger Derived;
- múltiples RuleUse y asociaciones distintas al mismo Program.

Tiempos: 0,0210 / 0,0413 / 0,0210 s; RSS máximo entre ellos 15,56 MiB.
Informe `target/test-diagnostics/20261010T025239591029Z-757093`.

Dos tests antiguos sobre multiplicidad de Tensor y ramas satisfechas pasaron,
también sin recompilar, en
`target/test-diagnostics/20261010T031308405204Z-778001`.

El test antiguo `no_explanation_is_distinct_from_ground_unit_rule` falló:
espera Some(Unit), pero recibe None. Su construcción elimina los parámetros de
una Rule cuyo tau no declarado fue elaborado como TimeExpr::Var. Borrar el
binder no transforma ese tiempo en TimeExpr::At; el objetivo utiliza At(tau).
Esta incoherencia se identifica por inspección del test y del matching temporal.
El test y ese matching no se modificaron en esta rama; no se ejecutó una segunda
versión debug=2 de ese test para comparar perfiles. No se clasifica su fallo como
una prueba de que reducir debug altere la semántica. En esa ejecución quedó
pendiente la corrección separada del fixture.

Informe de ese fallo:
`target/test-diagnostics/20261010T030815161898Z-772029`. El runner detuvo esa
invocación antes de ejecutar los dos siguientes; se ejecutaron posteriormente
de forma individual tras clasificar el fallo.

No se ejecutó la suite completa. Resultado agregado de los seis tests distintos
seleccionados: cinco pasaron y uno falló. Los tres de la API nueva pasaron.

**Actualización posterior:** la regresión temporal se cerró declarando tau como
constante en la fuente del test, sin cambiar Unit ni el matcher. Se verificaron
ocho tests individuales, incluyendo los tres anteriores de la API activa y dos
nuevos sobre Programs compuestos y compatibilidad. El fallo registrado arriba
describe la ejecución anterior a esa corrección; no permanece abierto. Los
resultados están en [activaciones encadenadas](residual-active-query-chained-programs.md).
No se continuó optimizando compilación ni se modificó su presupuesto.

## Decisión respaldada

Para esta verificación no fue necesario elevar el presupuesto: bastó retirar
símbolos de depuración manteniendo flags semánticos relevantes y usar binarios
ya compilados para los tests individuales. El margen sigue siendo limitado;
no se promete que todo crecimiento futuro compile bajo el mismo umbral.

Se conserva el perfil por defecto del repositorio. La compilación controlada
usó explícitamente:

```
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 python3 scripts/test_safe.py --test-target cli --compile-only
```

Para ejecución se emplea test_safe.py con `--binary` y `--test` exactos. No se
invocan Cargo ni binarios de tests directamente fuera del runner.

Las palancas posteriores y la medición que separa memoria anónima de caché
están en [compile-cost-optimizations.md](compile-cost-optimizations.md); desde
el 2026-11-01 `debug = 0` para tests está fijado en `Cargo.toml`.
