# owns como experimento de comportamiento programable

[owns_program.res](../examples/experiments/owns_program.res) usa únicamente
entidades, constantes, verbos Seeded/Derived, reglas Horn, aplicación parcial
de reglas y los bloques experimentales existentes.

```sh
python3 scripts/test_safe.py --fixture examples/experiments/owns_program.res --incremental
```

La entrada es `transfer(alice, car, bob) @ tau`. La regla `transferToOwnership`
produce `owns(bob, car) @ after(tau)`. Esa proposición Derived vuelve a entrar
como antecedente de `protectionFromCharlie`, que produce:

```text
notDo(charlie, molest, car, bob) @ after(after(tau))
```

`molest : Conduct` es la conducta; `car : Thing` se admite en el argumento
`indirectObject : Thing?`; bob es beneficiario y charlie es quien debe abstenerse.
`notDo` es un verbo positivo declarado Derived en este módulo separado. No es
EvidentialNot, negación por ausencia ni una ejecución de StateTransform. No se
modifica la definición Effect de `notDo` en Legal.

## Qué representan la proposición y la lambda

El evento `owns(bob, car)` afirma propiedad. La regla `ownsProgram` representa
el comportamiento asociado a esa propiedad:

```text
ownsProgram : Person → Conduct → Person → Thing → PropositionTime → Rule
```

Su antecedente determina owner, object y t; subject y conduct deben suministrarse
antes de aplicarla operacionalmente. La declaración existente:

```text
rule protectionFromCharlie = ownsProgram(charlie, molest)
```

realiza β-reduction durante elaboración. Su firma residual es:

```text
Person → Thing → PropositionTime → Rule
```

El resultado tipado es igual a escribir directamente la regla con charlie y
molest como constantes. La lambda sin especializar no permite que el matcher
invente los dos parámetros pendientes. No se genera una norma para otras personas
por cuantificación universal.

Esto muestra que una regla-lambda existente basta para representar este
comportamiento y su composición. No convierte el evento owns en un valor función.
La β-reduction produce una regla especializada durante elaboración. El matching
forward produce consecuencias Event; no devuelve reglas como valores runtime
ni modifica el conjunto de reglas. Esa frontera permanece abierta para el diseño
futuro de ProgramVerb, sin necesitarla para este experimento.

## Interpretación sobre conocimiento e incrementalidad

Para las reglas fijas R del archivo:

```text
T_owns : K → K
T_owns(F) = Known(close_R(F))
```

K es el dominio de conjuntos de eventos, ordenado por inclusión. La cadena es
add-only: la evidencia de transfer se conserva al añadir owns y notDo. No hay
defaults, negación por ausencia, retractaciones o consultas residuales ejecutadas.
La dependencia entre verbos es acíclica y la cadena sólo añade dos índices after;
no construye una secuencia temporal infinita.

El runner externo compara el cierre actualizado con una recomputación sobre
la unión. Reutiliza `base` vacío y `delta` con la transferencia. El campo
`query residual` expresa únicamente el goal observado por `--incremental`, como
en el experimento incremental anterior; no se modifica la gramática. Ese goal
es `owns(bob, car) @ after tau`: la sintaxis superficial actual sólo escribe un
`after` por índice. La composición de las reglas sí construye
`notDo @ after(after(tau))` internamente, y se observa en Known y en los tests.

El resultado esperado es tres eventos nuevos (transfer, owns, notDo), dos
justificaciones, igualdad lógica y de justificaciones. Los sellos de derivación
proceden del reloj externo, independientemente de PropositionTime.

Los tests comprueban la equivalencia de β-reduction con una regla explícita,
provenance y tiempos de la cadena, y monotonicidad sobre todos los subconjuntos
de dos transferencias ground definidas en fuente. Para los 16 pares de entradas
y deltas se compara:

```text
T_owns(F ∪ ΔF) = T_owns(F) ∪ DT_owns(F, ΔF)
```

Se contrasta tanto el conjunto lógico como todas las justificaciones y la
conservación de los sellos anteriores. Es un experimento finito y no demuestra
una semántica general de funciones o un cálculo de orden superior.

## Resultado observado

El archivo ejecutado produjo exactamente transfer, owns @ after(tau) y
notDo @ after(after(tau)), con dos justificaciones. Reportó igualdad lógica y
de justificaciones entre actualización y recomputación. El objetivo observado
owns pasó de false a true. Se usaron tres rondas, tres pivotes compatibles y
dos matches completos.

Pasaron tres tests nuevos y dos regresiones, individualmente bajo el runner
seguro. Se comprobaron todos los pares de subconjuntos de dos seeds (16 casos
incrementales), además de todas sus inclusiones para monotonicidad. La primera
compilación alcanzó 488 MiB RSS; la recompilación tras corregir la fixture,
327 MiB; ejecutar el programa utilizó unos 15 MiB. Se mantuvieron 1 GiB total,
swap cero, un job y los timeouts vigentes. No se ejecutó la suite completa.

La primera fixture incluía comentarios // que el lexer actual rechaza. Se
retiraron los comentarios y se colocó la explicación en este documento. También
se expresó el goal con un solo after, manteniendo la composición temporal real
en las reglas. No se modificaron parser, TypedAST, matcher, close, runtime,
residual ni defaults para realizar el experimento.
