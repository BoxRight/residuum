# Interfaz e implementación de owns, sin construcciones nuevas

El experimento vive únicamente en fixtures y en
[interface_tests.rs](../src/experiments/interface_tests.rs). No modifica el
parser, los AST, close, el matcher, el runtime, el residual, defaults o negación.
No introduce interface, ensures, Contract o ProgramVerb en Formal. El checker
es un harness de tests, no una capacidad pública del compilador o del runner.

## Archivos separados

[owns_specification.res](../examples/experiments/owns_specification.res) expone
la firma de owns y la consecuencia notDo con su firma de cuatro argumentos.
Su única garantía es una desigualdad Horn ordinaria:

```text
rule ownsGuarantee(owner, object, t) =
    owns(owner, object) @ t
    <= notDo(charlie, molest, object, owner) @ after t
```

Charlie y molest son constantes concretas. owner, object y t son los parámetros
de la lambda existente; no se añade cuantificación sobre todos los terceros.

[owns_implementation.res](../examples/experiments/owns_implementation.res)
define una lambda privada y la especializa con charlie y molest. La
[implementación inline](../examples/experiments/owns_implementation_inline.res)
escribe directamente la cláusula especializada. Ninguna implementación recibe
la regla ownsGuarantee al elaborarse: sólo recibe los tipos, constantes y firmas
públicas de la especificación.

[owns_consumer.res](../examples/experiments/owns_consumer.res) obtiene owns a
partir de transfer y usa notDo para producir su propia consecuencia protected.
No referencia ownsProgram ni implementedProtection. La cadena es:

```text
transfer @ tau
    → owns @ after(tau)
    → notDo @ after(after(tau))
    → protected @ after(after(after(tau)))
```

## Criterio de verificación

El checker exige una garantía Horn con antecedente owns y consecuente notDo,
sin variables pendientes. Elabora la implementación sin incorporar la garantía,
comprueba su compatibilidad con las firmas públicas y busca una regla cuya lista
de parámetros y cuerpo tipado sean iguales a los de la garantía tras β-reduction.
El nombre de esa regla puede ser diferente.

Este es un criterio **suficiente y limitado**. Al existir esa misma cláusula en
la implementación, cada sustitución ground compatible que permite aplicar la
garantía permite aplicar la cláusula real con la misma conclusión, evidencia
y sustitución. No se deduce la validez del mero texto de la interfaz ni se toma
la conclusión prometida como un hecho inicial.

No es un checker completo de implicación entre programas. Rechazaría, por ejemplo,
una implementación equivalente mediante varios pasos si no contiene la misma
cláusula normalizada. Tampoco normaliza renombrados de parámetros, demuestra
entailment FOUR o verifica un Contract general. Su alcance permite comprobar
este caso sin introducir un nuevo sistema de prueba.

La implementación vacía, la lambda sin especializar, un tiempo incorrecto y un
beneficiario incorrecto deben fallar aunque la especificación siga declarando la
garantía. Cambiar la garantía también exige nueva verificación.

## Frontera visible para el consumidor

Sólo una verificación exitosa construye el objeto privado del harness
VerifiedInterface. Ese objeto contiene exclusivamente la especificación, sin el
cuerpo de implementación. Su método de elaboración recibe el consumidor y las
declaraciones de la interfaz verificada. La composición continúa siendo plana,
como en el experimento anterior; no es un sistema de imports o privacidad M2.

El resultado del consumidor contiene únicamente tres reglas: ownsGuarantee,
transferToOwnership y protectionRecognized. No contiene las reglas privadas de
implementación. Su derivación de notDo cita ownsGuarantee; la de protected cita
la evidencia notDo. La autorización para usar esa regla proviene de la comprobación
anterior, no de afirmar que una regla escrita en una interfaz es automáticamente
cierta. El runner genérico --library no realiza esta comprobación y no debe
confundirse con este workflow verificado.

## Comparaciones operacionales

Los tests comparan tres caminos:

1. Consumidor con la interfaz verificada.
2. Consumidor con las reglas reales encapsuladas y sin ownsGuarantee.
3. Consumidor con la implementación inline y sin ownsGuarantee.

Los eventos y sus índices temporales deben ser iguales. Las sustituciones y los
antecedentes se comparan exactamente. Para provenance sólo se aplica la
correspondencia implementedProtection / inlineProtection ↔ ownsGuarantee; los
sellos de relojes independientes y el orden de descubrimiento no identifican
una justificación. Sin garantía ni implementación, el consumidor no puede producir
notDo ni protected.

Se comparan todos los subconjuntos de dos transferencias ground de fuente y sus
16 pares de base/delta, usando close únicamente como oráculo independiente de
recomputación. Se comparan los deltas lógicos y las nuevas justificaciones y se
conservan los sellos previos. El experimento no incorpora ningún chequeo al motor
forward ni modifica su semántica.

```sh
python3 scripts/test_safe.py --test experiments::interface_tests::interface_checker_uses_implementation_not_the_written_guarantee --test experiments::interface_tests::consumer_uses_verified_specification_without_implementation_body --test experiments::interface_tests::interface_inline_and_encapsulated_paths_have_equal_incremental_behavior
```

El campo query residual del consumidor sólo nombra el goal observado por el
harness incremental; no se realiza búsqueda residual. Los tests se ejecutan
individualmente bajo 1 GiB total, sin swap, un job y los timeouts existentes.

## Resultado observado

Pasaron los tres tests del experimento y dos regresiones, individualmente. El
checker aceptó la implementación especializada y la inline; rechazó los cuerpos
vacíos, sin especializar, con tiempo o beneficiario incorrectos, la sustitución
de una firma pública y una garantía temporal modificada sin cambiar el cuerpo.

El consumidor produjo transfer, owns, notDo y protected, con tres justificaciones.
Su módulo tipado no contiene las reglas privadas. Los 16 pares de base/delta
coincidieron entre interfaz verificada, cuerpo encapsulado e inline, incluidos
eventos, sustituciones, evidencia y justificaciones bajo la correspondencia de
nombres indicada. La compilación alcanzó 555 MiB RSS y cada test unos 15 MiB.
No se ejecutó la suite completa ni se usó el runner genérico para incorporar
una garantía sin pasar por el checker.

Para esta forma normal, una propiedad garantizada puede representarse como una
regla existente y verificarse por comparación de cláusulas tipadas normalizadas.
El resultado no establece un mecanismo general de verificación modular en M2:
ese mecanismo continúa siendo responsabilidad explícita del harness.
