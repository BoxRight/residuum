# Asociación tipada de una declaración Verb con una Rule

Este documento registra el incremento inicial de asociación pasiva. La etapa
posterior de [activación forward](active-verb-programs.md) añade `close_program`
para instancias Seeded y Derived; el `close` que recibe sólo reglas conserva la
política descrita aquí.

Este incremento añade únicamente el vínculo de declaración. La nueva fixture es
[sells_associated_program.res](../examples/experiments/sells_associated_program.res),
comparada con [sells_program.res](../examples/experiments/sells_program.res).
Los tests están en
[associated_program_tests.rs](../src/experiments/associated_program_tests.rs).

## Sintaxis y representación

Se conserva la categoría explícita del verbo y se añade una cláusula opcional:

```text
seeded verb sells(subject : Person, object : Thing, recipient : Person) =
    program sellsProgram
```

En SurfaceAST, `FormalVerb.program` es un `Option<String>`. En TypedAST,
`Verb.program` es un `Option<RuleRef>`, reutilizando el tipo de referencia existente.
Apunta a la regla parametrizada completa; no almacena una instancia ground, una
copia de su cuerpo ni una nueva categoría ProgramVerb.

La misma cláusula es admitida para las categorías Derived y Effect. Para un
Effect con operaciones, se escribe después de su cuerpo de estado. El campo
`effect` sigue representando exclusivamente la transformación State → State;
`program` es una relación independiente con una lambda de reglas.

El parser Chumsky y el parser manual de control producen el mismo SurfaceAST.
La cláusula empieza consumiendo `=` y es opcional fuera de cualquier repetición.
La producción de declaración de verbo conserva su prefijo obligatorio, así que
cada iteración consume input. No se modifican separadores ni se añaden ramas
recursivas. No se amplía el parser CNL: su constructor TypedAST inicializa el nuevo
campo en None para conservar compatibilidad; ese frontend sigue pausado.

## Comprobación de tipos y resolución

La firma asociada debe tener exactamente los tipos de los argumentos del verbo,
en el orden de declaración, seguidos por PropositionTime:

```text
sells:        Person, Thing, Person
sellsProgram: Person, Thing, Person, PropositionTime -> Rule
```

Los nombres de parámetros de la lambda no tienen que coincidir con las etiquetas
de roles del verbo. En esta primera versión se exige igualdad exacta de tipos;
no se añade subtipado de funciones ni inferencia de argumentos suplementarios.

La referencia se resuelve después de elaborar las declaraciones. Esto permite
declarar sells antes de sellsProgram, necesario porque la propia regla utiliza la
firma de sells. Sólo este vínculo admite esa resolución diferida; el orden previo
de las aplicaciones de reglas y sus demás referencias permanece igual.

Se rechazan referencias inexistentes, referencias a un const o a un verb en lugar
de una rule, nombres de regla ambiguos y firmas incompatibles. Los tests incluyen
orden equivocado de tipos, cantidad incorrecta de parámetros, tiempo ausente y
tiempo en una posición incorrecta.

Una regla ya especializada es un destino válido si su firma residual coincide:
`juanSale = sellsProgram(juan)` puede asociarse a una declaración de dos argumentos
Thing, Person, conservando el tiempo implícito. Se prueba ese caso sólo como typing;
no se afirma todavía que el cuerpo de esa lambda procese automáticamente eventos
de la nueva declaración. La comprobación de asociación no es una verificación de
contrato ni impone que el cuerpo tenga cierto antecedente.

## Referencia asociada frente a referencia explícita

El test sigue `sells.program` en el AST, resuelve su Rule y usa su nombre como
destino de la producción de aplicación parcial existente. La especialización se
realiza mediante el mismo elaborador que procesa:

```text
rule juanSale = sellsProgram(juan)
```

Se compara con la referencia explícita de la fixture anterior. Los parámetros
residuales y la regla normalizada coinciden. Después se comparan los tres escenarios
real, hypothetical y reuse: mismos eventos, índices temporales, metadata de
transición, sustituciones, antecedentes, registros de derivación y resultados
residuales. La metadata temporal se compara expresamente porque Event equality
no la incluye toda.

Una segunda comprobación selecciona explícitamente la regla a partir del RuleRef
y pasa ese conjunto de reglas al close existente. También conserva los eventos y
los registros de derivación. Esa selección es una operación visible del harness,
no una política de activación añadida al motor. No existe una tabla especial de
Rust para sells: el destino se obtiene de la declaración fuente y del AST.

La cadena se conserva tal como está en la fixture anterior, con pedro como comprador:

```text
sells(juan, car, pedro) @ tau
  -> give(juan, car, pedro) @ after(tau)
  -> owns(pedro, car) @ after(tau)
  -> notDo(charlie, molest, car, pedro) @ after(after(tau))
```

La consulta sin evidencia sigue devolviendo sells como requisito sin incorporarlo
a Known. Se preservan ambas justificaciones de give, genérica y especializada.

## Límite deliberado

La relación representada y comprobada es **Verb → Rule parametrizada**. No se
cambia la identidad lógica de Event, el matcher, close, el residual ni el runtime.
No se introducen reglas generadas, imports, interfaces, certificados de contrato,
defaults ni una categoría nueva de verbo.

Las categorías y la política actual de abducibles permanecen iguales. Un test
comprueba que, si se llama close con el seed pero sin reglas, sólo queda el seed:
la asociación no carga ni activa reglas automáticamente. El runner externo sigue
ejecutando las declaraciones Rule ordinarias que recibe.

Este resultado permite investigar posteriormente qué debe ocurrir cuando una
instancia del verbo se vuelve conocida. Esa decisión queda fuera de este incremento.

## Verificación

Pasaron tres tests nuevos y cuatro regresiones individuales: equivalencia de
parsers en las fixtures existentes, el caso sells real/abducido, sus deltas
incrementales y la especialización Legal.delivery. También se ejecutó la nueva
fixture mediante el runner externo.

La compilación de tests usó ~651 MiB RSS (~691 MiB en el cgroup), dentro de los
controles existentes. Los tests usaron ~15 MiB. El runner de la fixture compiló
con ~513 MiB y ejecutó con ~15 MiB. Se mantuvieron 1 GiB total, sin swap, un job,
un hilo y los timeouts; no se ejecutó la suite completa.

Logs de tests: `target/test-diagnostics/20261001T035346496658Z-1025033/`.
Regresiones: `target/test-diagnostics/20261001T035550401145Z-1026833/`.
Runner: `target/test-diagnostics/20261001T035559062489Z-1026860/`.
La comparación positiva se repitió después de añadir comprobaciones explícitas de
metadata temporal: `target/test-diagnostics/20261001T035908835436Z-1029425/`.
