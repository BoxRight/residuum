# Certificación de generación ontológica acotada

Experimento en `experiment-legal-order-geometry`. Se utiliza
[el contrato de referencia](legal-relational-search-space-contract.md) para una
prueba técnica de generación. `compliance` sigue siendo la cabeza del perfil;
no se identifica este catálogo con el espacio completo de aprendizaje normativo.
No se asignan etiquetas externas ni se modifica un contrato de decisión `D`.

El motor, las ontologías y las fixtures originales permanecen intactos. Se añade
un módulo del harness y su registro en `legal_geometry_tests.rs`.

**Resultado:** se enumeraron completamente 22.621 configuraciones de un
subperfil explícito. Se admitieron 10.850 y se rechazaron 11.771. Cada fila
conserva su regla, el orden de antecedentes, las condiciones aplicadas y la
comprobación del matcher real. La enumeración comprimida está en
[enumeration.tsv.gz](legal-geometry/structural-generation/enumeration.tsv.gz),
con [conteos por longitud](legal-geometry/structural-generation/counts.tsv) y
[ejemplos de control](legal-geometry/structural-generation/examples.tsv).

## 1. Corrección de finitud del contrato de referencia

La gramática del contrato permite `I`, literales y `Antecedent & Antecedent`.
El presupuesto limita el número de átomos, pero no los nodos `I` ni los nodos
de conjunción. Por tanto, tal como está escrita, **no determina un conjunto
finito de AST**. La afirmación de finitud del apartado 5 necesita otra hipótesis.

Sea `q` una instancia ground de `compliance` con constantes del manifiesto.
Definimos `A1 = I` y `A(n+1) = A(n) & I`. Las reglas `A(n) <= q`:

* tienen cero átomos y cero variables;
* tienen una sola cláusula y términos dentro de la profundidad permitida;
* conservan la misma cabeza ground y no necesitan un binding pendiente;
* tienen AST distintos para cada `n`.

**THEOREM:** esta construcción proporciona infinitas formulaciones dentro de
los límites escritos. Su interpretación de soporte coincide, pero su estructura
fuente es distinta. No pueden contarse como un catálogo finito de formulaciones
sin decidir qué estructura se acota o normaliza.

**FINITE RESULT:** el test verifica los primeros 32 miembros con el matcher
existente. Todos están admitidos por las restricciones restantes y producen una
instanciación, sin antecedentes de evidencia. Se conserva el
[certificado](legal-geometry/structural-generation/unit-budget-counterexample.tsv).
El argumento de infinitud es matemático, no una extrapolación de 32 casos.

La corrección mínima puede tomar dos formas:

1. Acotar también los nodos de `I` y de `&`, manteniendo todas las formulaciones
   dentro de ese presupuesto.
2. Enumerar cuerpos como secuencias ordenadas, usar `I` sólo para la secuencia
   vacía y fijar una asociación de `&`; conservar otras formulaciones como
   representaciones originales asociadas, sin borrarlas.

Este experimento declara la segunda opción **sólo para su subperfil**. El
contrato previo permanece como referencia y no se corrige silenciosamente.
La conservación de antecedentes repetidos y de su orden es parte del subperfil.

## 2. Tamaño de una subfamilia del perfil amplio

Incluso después de resolver los nodos neutros, no es razonable materializar
todo el perfil amplio. Las constantes del manifiesto y del módulo elaborado son:

| Tipo | Constantes |
|---|---:|
| Person | 3 |
| Thing | 1 |
| Conduct | 4 |
| Fact | 9 |
| Authority | 5 |
| PropositionTime | 5 |

Con profundidad de `after` hasta 2 hay 15 términos temporales ground distintos.
La firma de `right` admite 364.500 átomos ground y la de `compliance` admite
820.125 cabezas ground. Cuatro antecedentes `right` ordenados, con repeticiones,
y una de esas cabezas proporcionan:

\[
364500^4\cdot820125
=14\,476\,720\,225\,405\,007\,812\,500\,000\,000.
\]

Todos esos AST tienen argumentos tipados y cabeza ground. En este vocabulario
hay una sola constante Thing, `gObject`, compartida entre cuerpo y cabeza, por
lo que también pasan la conexión directa por identidades. Esto no demuestra
que sean razones jurídicas adecuadas ni que cada cuerpo tenga soporte en un
expediente del dominio.

El cálculo se comprueba con los tipos y constantes del `Module` real; se conserva
[la cota](legal-geometry/structural-generation/parent-lower-bound.tsv).
Es una cota inferior de una subfamilia normalizada, no el cardinal exacto del
perfil padre ni un certificado de enumeración de ese perfil. Los índices
temporales anidados se cuentan como AST; su escritura fuente tiene el límite
documentado en el apartado 5.

## 3. Subperfil enumerado completamente

Se fija `T = LegalGeneral`, cargada por la fixture sintética `GeometryGeneral`.
Las firmas proceden del AST elaborado, con los nombres y tipos extraídos en el
contrato previo. El alfabeto tiene doce literales:

```text
give, do, notDo, Act, ~Act,
obligationFiled, obligation, right,
permissionFiled, permission, prohibition, omissionProhibition
```

Cada literal tiene una plantilla relacional con bindings compartidos. El mapa
de composición está declarado en el harness:

* `subject`, `object` y `beneficiary` se comparten.
* `recipient` de `give` se conecta con `beneficiary`.
* `indirectObject` de `do/notDo` se conecta con `object`, de tipo Thing,
  asignable a Thing?. No se enumera `none` en este subperfil.
* Los siete canales normativos utilizan el identificador `norm`.
* `Act/~Act` utiliza `act`, separado de `norm`.
* Normas y actos comparten `conduct`, pero mantienen autoridades y vigencias
  independientes: `authority/effective` y `actAuthority/actEffective`.
* Los literales normativos y históricos tienen índice `@ t`; los efectos
  tienen `@ after t`.

Estos bindings son una política estructural del subperfil, no una operación
inferida automáticamente por el motor ni una afirmación sobre suficiencia
normativa. Identificar todo identificador normativo con `norm` reduce el espacio;
no enumera todas las configuraciones con normas distintas.

La cabeza es:

```text
compliance(norm, act, subject, object, beneficiary, authority, effective) @ t
```

Los cuerpos son todas las palabras ordenadas de longitud 0 a 4 sobre ese
alfabeto, con repeticiones. La palabra vacía se representa por `I`; las restantes
por asociación izquierda de `&`. Cada regla declara únicamente las variables
utilizadas en su cuerpo o cabeza. No hay capturas de constantes, records,
renombrados adicionales, separaciones/identificaciones nuevas de variables,
programas compuestos ni activación por defaults dentro de esta enumeración.

El universo de 128 entradas Seeded del contrato se conserva como contexto
experimental. **No se calculan aquí 128 cierres por cada una de las 22.621
reglas.** La certificación actual verifica generación y condiciones estructurales,
no cobertura normativa exhaustiva sobre ese producto de dominios.

## 4. Certificado de completitud y admisión del subperfil

Para cada longitud `n`, un contador base 12 recorre exactamente los rangos
`0 .. 12^n`. Su representación de `n` dígitos identifica una única palabra.
La recuperación del rango a partir de la palabra es única. Esto demuestra
completitud y ausencia de duplicación de palabras en **este subperfil**.

El validador comprueba firma, aridad, tipos, existencia de términos, polaridad,
presupuesto, seguridad de la cabeza y conexión directa por bindings. Es un
validador del perfil ordinary/Horn sin records ni subtipos; no pretende comprobar
todo AST posible del lenguaje. No implementa la conexión por dependencias
inferenciales del perfil amplio ni subsunción simbólica.

En este alfabeto, `act` sólo obtiene binding a través de `Act/~Act`; `norm`,
la autoridad normativa y la vigencia normativa sólo lo obtienen a través de
alguno de los siete canales normativos. Por tanto:

\[
admitted(w)\iff
w\text{ contiene un literal Act/~Act y un canal normativo}.
\]

Esto es una consecuencia de los bindings de la cabeza elegida, no un filtro
que declare esos pares razones jurídicamente suficientes.

Por inclusión-exclusión, para longitud `n`:

\[
accepted(n)=12^n-10^n-5^n+3^n.
\]

El segundo término elimina palabras sin `Act/~Act`; el tercero, palabras sin
canales normativos; el cuarto recupera las palabras que sólo contienen efectos.

| Longitud | Enumeradas | Admitidas | Rechazadas |
|---|---:|---:|---:|
| 0 | 1 | 0 | 1 |
| 1 | 12 | 0 | 12 |
| 2 | 144 | 28 | 116 |
| 3 | 1.728 | 630 | 1.098 |
| 4 | 20.736 | 10.192 | 10.544 |
| Total | **22.621** | **10.850** | **11.771** |

La fórmula y el validador se comprueban separadamente contra todas las palabras.
El matcher real `instantiate_from_known` se aplica además a cada regla:

* se suministra un banco de doce instancias ground, una por plantilla;
* toda admisión produce exactamente una instanciación con la cabeza esperada;
* todo rechazo carece de instanciación por falta de binding de salida;
* cada prueba conserva el productor y todos los antecedentes, incluidos su
  orden y repeticiones.

Este banco contiene premisas Derived/Effect e históricas explícitas. No se
presenta como doce seeds admisibles ni como un cierre obtenido de la teoría.
La cabeza `compliance` no está en ese banco: el matcher la construye mediante
sustitución. Se certifica instanciabilidad con testigos ground, no existencia
de esos testigos en cualquier expediente jurídico.

## 5. Ejemplos aceptados y rechazados

Los 16 casos de control se comparan también con `load_formal_module` y el
elaborador actuales. Los archivos fuente correspondientes se generan en
`target/legal-structural-generation/`; sus reglas y diagnósticos completos
quedan archivados en `examples.tsv`.

| Ejemplo | Perfil | Elaboración | Condición |
|---|---|---|---|
| Obligación + Act | Admite | Acepta | Binding completo y tipos correctos |
| Permiso + ~Act | Admite | Acepta | Misma seguridad estructural; adecuación normativa sin valorar |
| Intercambiar personas en Act | Admite | Acepta | Person sigue siendo Person; bindings explícitos diferentes |
| Obligación sin binding de `act` | Rechaza | Acepta | `S_UNBOUND_HEAD` |
| Thing en posición Person | Rechaza | Rechaza | `S_TYPE` |
| Cabeza con aridad incorrecta | Rechaza | Rechaza | `S_ARITY` |
| Constante Person como índice temporal | Rechaza | Rechaza | `S_TYPE` |
| Constante inexistente | Rechaza | Rechaza | `S_UNKNOWN_TERM` |
| Cabeza negativa de compliance | Rechaza | Acepta | `P_HEAD_INTERFACE`, `P_POLARITY` |
| Cinco antecedentes | Rechaza | Acepta | `B_ATOMS` |
| Efecto ground desconectado de la cabeza | Rechaza | Acepta | `S_DISCONNECTED` |
| Parámetro de salida no declarado | Rechaza | Rechaza | `S_UNKNOWN_TERM`, `S_UNBOUND_HEAD` |
| `after` de profundidad 3 en argumento | Rechaza | Acepta | `B_AFTER_DEPTH` |
| Diecisiete binders tipados | Rechaza | Acepta | `B_VARIABLES` |
| Obligación negativa en el cuerpo | Rechaza | Acepta | `P_POLARITY` del perfil |
| Índice temporal anidado de profundidad 2 | Admite el AST | No acepta esa escritura fuente | Límite de sintaxis temporal |

Intercambiar dos posiciones Person no vulnera un sistema nominal de roles:
tal sistema no existe en `Arg`. El ejemplo conserva los bindings escritos y
puede describir otro vínculo relacional; su evaluación jurídica permanece abierta.
Tampoco se excluye `permission & ~Act <= compliance` por una valoración
normativa anticipada. Ambas admisiones ilustran la separación metodológica.

El último ejemplo tiene `TimeExpr::After(After(Var(t)))`, permitido por los
tipos, la seguridad y la profundidad del perfil. El parser fuente sólo admite
`@ nombre` y `@ after nombre`; `@ after(after t)` falla. Se registra admisibilidad
del AST separadamente de expresabilidad fuente. No se amplía el parser ni se
declara esa estructura temporal semánticamente inválida.

Para obligación + Act se calcula además un cierre completo original y uno
hipotético independiente con la regla añadida. Coinciden los eventos; todas
las justificaciones originales permanecen y aparece la nueva justificación,
con origen `StructuralExample`. No se comparan literalmente los sellos
`derivedAt`. Esta prueba verifica la conservación de la referencia en ese caso,
no los cierres de todas las configuraciones enumeradas.

## 6. Verificación y alcance

Implementación experimental:
`src/bin/residuum/cli/legal_geometry_structural_generation_tests.rs`.
Se registra como `structural_generation` en el harness de geometría existente;
no se añade a las APIs del motor.

Tests individuales ejecutados con `python3 scripts/test_safe.py`, con
`CARGO_BUILD_JOBS=1` y `CARGO_PROFILE_TEST_DEBUG=0`:

```text
cli::tests::legal_geometry::structural_generation::complete_role_shared_slice_has_structural_certificates
cli::tests::legal_geometry::structural_generation::structural_examples_distinguish_profile_checks_from_elaboration
```

La última ejecución de cada prueba y los recursos del runner se conservan en
[verification.json](legal-geometry/structural-generation/verification.json).
Se mantienen 1 GiB total, sin swap, un trabajo de Cargo, un hilo del harness y
los límites de tiempo. No se ejecuta la suite completa. Tras la interrupción se
leyeron los resultados finalizados; no se atribuye un éxito a una ejecución
pendiente. Se repitió únicamente la prueba de ejemplos tras corregir la
presentación fuente del índice temporal anidado y reforzar la comparación de
tipos, transiciones y orígenes de todas las declaraciones importadas.

Clasificación del resultado:

| Afirmación | Estado |
|---|---|
| Omisión de finitud por nodos `I` | THEOREM; 32 casos verificados |
| Completitud del contador de palabras | THEOREM para el subperfil declarado |
| Fórmula de admisiones | THEOREM para esas plantillas y cabeza |
| Tipos, bindings y matching de 22.621 reglas | FINITE RESULT |
| Elaboración de 16 controles | FINITE RESULT |
| Conservación del cierre original en obligación + Act | FINITE RESULT |
| Enumeración del perfil amplio de variables/capturas/programas | No realizada |
| Subsunción y operadores mínimos completos | Pendientes |
| Adecuación normativa de candidatos | Sin valorar |
| Decisión efectiva y filtros convexos | Pendientes |

El siguiente paso depende de resolver el presupuesto de estructura de cuerpos
y escoger qué variaciones tipadas ampliar: variables separadas/identificadas,
capturas, constructores o RuleUses. El presente resultado no autoriza una
revisión de la teoría jurídica ni una ampliación silenciosa del perfil.
