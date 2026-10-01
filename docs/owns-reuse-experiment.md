# Reutilización de la lambda asociada a owns

El experimento separa definición y uso:

- [Property](../examples/experiments/property.res) declara los tipos, molest,
  owns, notDo y la lambda ownsProgram.
- [Protection](../examples/experiments/protection.res) declara sus constantes,
  transfer, la regla que deriva owns y `protection = ownsProgram(charlie, molest)`.
  No copia el cuerpo de ownsProgram.
- [ProtectionInline](../examples/experiments/protection_inline.res) es la
  referencia independiente: reemplaza la aplicación por el cuerpo especializado
  escrito directamente, con el nombre inlineProtection.

La firma de la biblioteca ordena primero los parámetros que se aplican:

```text
ownsProgram : Person → Conduct → Person → Thing → PropositionTime → Rule
              thirdParty conduct owner    object  t
```

Esto permite `ownsProgram(charlie, molest)`, con firma residual
Person → Thing → PropositionTime → Rule. El orden owner, object, thirdParty, t
del bosquejo no permitiría esa aplicación prefija.

## Composición experimental de los archivos

No se añade sintaxis import, namespaces, exports o una categoría ProgramVerb.
`elaborate_composed_fixture` recibe dos SurfaceAST ya parseados, concatena las
declaraciones de la biblioteca antes de las del consumidor y utiliza el elaborador
ordinario. Los nombres comparten un espacio plano y las declaraciones de estas
fixtures son disjuntas. Se rechaza una biblioteca con bloques experimentales,
para no mezclar datos con definiciones reutilizables. No es un sistema de módulos
de producción; Protection aislado no resuelve los símbolos de Property.

La referencia a la lambda y su β-reduction sí utilizan las construcciones actuales
del compilador: SurfaceRuleBody::Application y la elaboración de Rule. Un cambio
en el cuerpo de Property cambia la regla especializada en Protection sin cambiar
el consumidor. El test comprueba esta dependencia explícitamente.

```sh
python3 scripts/test_safe.py --fixture examples/experiments/protection.res --incremental --library examples/experiments/property.res
python3 scripts/test_safe.py --fixture examples/experiments/protection_inline.res --incremental --library examples/experiments/property.res
```

Cada invocación tiene las mismas fases y límites del runner. La suma de los dos
archivos no puede superar 64 KiB. --library requiere --fixture y --incremental;
no inicia tests o una suite implícita.

## Comparaciones del experimento

La β-reduction de protection y la regla inline producen exactamente los mismos
parámetros y cuerpo tipado. Sus nombres son distintos intencionalmente.

Para comparar los cierres, se aplica exclusivamente la correspondencia de nombres
inlineProtection ↔ protection en DerivationRecord. Los eventos, sus tiempos,
las sustituciones y los antecedentes se comparan sin normalizaciones adicionales.
Se ignoran únicamente el orden de descubrimiento y los sellos de un reloj de
derivación independiente. Esa es la isomorfía de provenance usada aquí; el TypedAST
actual no registra un historial separado de expansión de aplicaciones o archivos
de origen, y este experimento no lo agrega.

Los tests también comparan la composición por etapas:

```text
close(transferencias, transferToOwnership)
    → Known con owns
close(Known, protection)
    → Known con notDo
```

Se conservan las justificaciones de ambas etapas para compararlas con el cierre
conjunto. La aplicación individual de protection recibe el evento owns derivado;
no afirma owns externamente como seed.

Sobre todos los subconjuntos de dos transferencias ground parseadas de fuente:

- se compara cierre aplicado e inline, incluidas las sustituciones;
- se comprueba monotonicidad por inclusión de Known en ambos caminos;
- se comparan los 16 pares de entradas y deltas con recomputación desde cero;
- se comparan los deltas de eventos y de justificaciones entre ambos caminos;
- se exige conservar las derivaciones y sellos anteriores.

El archivo observa owns @ after(tau); Known también expone
notDo(charlie, molest, car, bob) @ after(after(tau)). El campo query residual
se usa sólo como goal observado en el modo incremental, sin búsqueda residual.

## Resultado observado y alcance

La equivalencia demuestra reutilización de un comportamiento mediante
Rule + lambda + β-reduction, composición forward y mantenimiento incremental
para este caso finito. owns sigue siendo una proposición; su comportamiento se
expresa mediante una lambda nombrada y referenciable. No demuestra que los eventos
sean funciones, que Horn produzca reglas como valores runtime, ni una semántica
general de orden superior. La ausencia de imports es una frontera distinta del
significado de la lambda y queda visible en el harness.

Pasaron los cuatro tests nuevos y dos regresiones seleccionadas, cada uno en
un proceso limitado. Ambos caminos se ejecutaron desde sus archivos con
--library: tres eventos, dos justificaciones, tres rondas y delta lógico idéntico.
Los tests confirmaron igualdad de sustituciones y la isomorfía de provenance,
también al componer los cierres por etapas y en los 16 casos incrementales.
La compilación de tests alcanzó 446 MiB RSS; la del ejemplo 288 MiB; las ejecuciones
unos 15 MiB. Se mantuvieron 1 GiB total, sin swap y los timeouts vigentes. No se
ejecutó la suite completa.

No se modifican parser, SurfaceAST, TypedAST, matcher, close, runtime, residual,
defaults ni negación por ausencia para realizar este experimento.
