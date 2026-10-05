# Program mínimo

Estado: implementado en Formal, SurfaceAST, TypedAST, elaboración y activación
forward y selección nativa por defaults. Las verificaciones descritas abajo son IMPLEMENTED + TESTED en los
casos seleccionados; no constituyen un teorema de equivalencia general.

## Representación

Rule permanece como una sola cláusula parametrizada (Horn o residual).
Program contiene identity, parameters, rule_uses y defining_module. Cada RuleUse
contiene una RuleRef resuelta y bindings tipados por el elaborador. No existe un
segundo AST de cláusulas, ni un bloque environment, shared o captures.

```text
program Chain(object : Thing, t : PropositionTime) {
    use first { local = object, when = t }
    use second { local = object, when = t }
}
program Partial = Chain(item)
program Ground = Partial(tau)
```

Los bindings nombran parámetros de la Rule a la izquierda. A la derecha pueden
usar parámetros de Program, constantes visibles, none o records existentes.
Las listas usan comas internas obligatorias y permiten una coma final opcional.
Cada producción repetida consume input. Un programa requiere al menos un use.

Los parámetros de Program tienen tipos explícitos. El elaborador comprueba el
tipo de cada binding contra la firma de la Rule. Rechaza reglas inexistentes,
parámetros de Rule inexistentes, bindings duplicados, tipos incompatibles,
parámetros de Program duplicados y aplicaciones inválidas. Las referencias de
use siguen la resolución ordenada existente: la Rule debe estar elaborada o
importada al definir el programa. Verb.program se resuelve al final, como antes.

## Ámbito y β

El módulo de definición proporciona el ámbito léxico mediante import. No se
buscan símbolos nuevos en el consumidor. Las aplicaciones parciales conservan
ese ámbito aunque su nueva declaración tenga origen en otro módulo.

La aplicación de Program utiliza el prefijo posicional de constantes tipadas,
igual que la aplicación existente de Rule. Sustituye todos los bindings,
recursivamente dentro de records, y retiene los parámetros no aplicados. Las
constantes capturadas permanecen constantes; los parámetros locales de cada
RuleUse permanecen locales. No se enlazan parámetros por coincidencia de nombre.

compile_program baja todos los RuleUse a Rules individuales. Usa nombres internos
higiénicos distintos para parámetros compartidos y locales. Aplica sustitución
tanto a términos como a índices temporales y StateTransitionType, utilizando
la sustitución existente. Elimina los binders locales que fueron vinculados.
Las reglas resultantes se entregan juntas a un único close, sin cierres
independientes por miembro. Declarar o especializar Program no ejecuta nada.

La sintaxis fuente temporal conserva su alcance anterior: @ t y @ after t.
Los TimeExpr internos siguen admitiendo anidamiento; esta extensión no cambia
la gramática temporal.

## Program autónomo y asociación con Verb

El runner/verificador puede seleccionar un Program declarado en Residuum,
obtener compile_program(program, module) y entregar ese vector completo a close.
El significado de los miembros y bindings vive en la fixture, no en una tabla
especial del runner. Definir un Program autónomo no cambia por sí solo la
participación ordinaria de Rules ya existentes en el módulo.

Verb.program ahora utiliza BehaviorRef, que distingue una Rule individual de
un Program compuesto. Las asociaciones anteriores siguen admitidas. La firma
del destino debe coincidir con los argumentos del verbo más PropositionTime.

```text
seeded verb starts(object : Thing) = program Chain
```

Una instancia positiva conocida de un verbo Seeded o Derived activa todos los
miembros asociados con los mismos argumentos e índice temporal. Esos miembros
permanecen activos en el mismo fixpoint y sus conclusiones pueden activar otros
miembros. La instancia disparadora se conserva como antecedente de provenance,
incluso si el cuerpo de una regla es I. EvidentialNot no dispara la asociación.
Effect conserva su comportamiento anterior: no se activa su asociación como
Seeded/Derived ni se ejecuta StateTransform al derivar.

La asociación controla las definiciones referenciadas: igual que antes una
Rule asociada no era obligatoria, los miembros de un Program asociado se excluyen
del conjunto ordinario de close_program. Pueden pertenecer a más de una
asociación y conservar las justificaciones correspondientes. Esto no introduce
una política de propiedad exclusiva de Rules. El runner de defaults controla
separadamente los miembros de sus programas para que no se activen antes de
la selección; rechaza su asociación simultánea a Verb.program en ese runner.

## Provenance

Cada Rule compilada tiene un identificador Program::índiceDeUse::Rule. El índice
distingue usos repetidos de una misma regla. DerivationRecord sigue intacto.
program_rule_origin resuelve ese identificador a Program, módulo de definición,
Rule, módulo/archivo de origen y posición del RuleUse. La CLI explain presenta
esta información. La instancia activadora sigue formando parte de la evidencia.

β elimina binders aplicados y los binders restantes reciben nombres higiénicos;
por eso comparar literalmente los nombres de sustituciones con reglas inline
no sería correcto. La prueba de NormalSale restaura las sustituciones mediante
la Rule original y los mismos antecedentes, y compara esa provenance con la
referencia. Sólo el valor del sello operacional derivedAt queda fuera de la
comparación semántica. Los índices proposicionales y transiciones se comparan.

## Casos verificados

- examples/legal/modules/sales.res define normalSaleProgram con las Rules reales
  normalSale, reciprocalTransferPermission y plainTransferObligation.
  programs/normal_sale.res expone su alias NormalSale para la prueba de aplicación.
- normal_sale_scenario.res verifica una aplicación parcial, una aplicación
  completa en dos pasos y una aplicación completa directa. Los tres miembros y
  la regla general del entorno obligationCreatesRelation alcanzan do mediante
  el mismo fixpoint y coinciden con las cuatro reglas originales.
- examples/experiments/associated_multi_rule_program.res verifica parámetros
  locales, constants capturadas, β temporal, records anidados y activación desde
  hechos Seeded/Derived, ausencia de activación y evidencia negativa.
- multi_rule_program_consumer.res verifica asociación y especialización desde
  otro módulo, orígenes y no ejecución de experiments importados.
- Se comprobaron además regresiones de β temporal, imports, asociación anterior
  y la selección anterior de defaults.

Pasaron nueve tests individuales del target CLI mediante scripts/test_safe.py.
La última compilación tuvo pico RSS de 488,91 MiB y las ejecuciones individuales
15,03–15,50 MiB. No se ejecutó la suite completa ni se compiló el target de tests
de biblioteca. Se mantuvieron 1 GiB total, sin swap y los límites preventivos.

## Frontera

DefaultBody admite RuleRef y ProgramRef. La integración posterior se describe en
[Program y defaults](program-defaults.md). Competencias,
familias y consistency siguen pendientes. Tampoco se
extiende residual, FOUR, incrementalidad de asociaciones ni State/runtime.
El runner de consultas experimentales rechaza módulos con Program para evitar
tratar silenciosamente sus miembros como una consulta sobre Rules ordinarias.
El pipeline nuevo es forward; las consultas existentes de módulos sin Program
conservan su comportamiento.
