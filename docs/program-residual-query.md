# Consulta residual sobre un programa

La [semántica operacional consolidada](monotonic-core-specification.md)
sitúa esta consulta fuera del cierre forward y conserva su garantía de
suficiencia para reglas explícitas, sin prometer completitud general.

`query_program_residual(program, evidence, goal, limits)` añade búsqueda
entre reglas al API local `query_residual(rule, goal, known)`, que se conserva.
Recibe un `Module` tipado y evidencia inmutable. No añade sintaxis al parser.
`Verb::is_abducible` establece `Seeded ⇒ Abducible` por defecto. No hay una nueva
estructura matemática ni sintaxis `abducible` en el parser. Se admiten hipótesis
seeded positivas o evidencialmente negativas: son propuestas explícitas separadas
de los hechos reales, nunca negación inferida por ausencia.

El API opcional `query_program_residual_with_abducibles(program, evidence, goal,
abducibles, limits)` conserva la restricción externa por verbo y polaridad.

La vista de reglas recoge las declaraciones Rule del módulo como reglas
explícitas, incluidas las referenciadas por Verb.program. No aplica los guards
de activación de close_program; las garantías de ambos modos no son intercambiables.

Para cada evento pendiente:

1. Si está en la evidencia, devuelve `I` y conserva el evento como testigo.
2. Si es un seed ausente, sólo lo propone si su verbo y polaridad están permitidos.
3. Para un Derived o Effect, examina todas las reglas, unifica cada head con el
   objetivo y sustituye el antecedente con esos bindings.
4. Expande sus eventos recursivamente; una conjunción combina el producto
   cartesiano de las explicaciones de sus operandos.
5. Filtra los requisitos finales contra los abducibles y verifica sus pruebas
   mediante el matcher forward, antes de resumir las ramas con `Join`.

Las cláusulas residuales usan su vista Horn equivalente. Las identidades runtime
que coincidan con nombres de parámetros no se vuelven variables después de
sustituir. Los tiempos `after` conservan su estructura, y las transiciones de
estado no participan en la identidad lógica ni se resuelven durante la búsqueda.

## Resultado y alternativas

`ProgramResidualQuery` contiene el objetivo, las explicaciones con sus árboles
de prueba, métricas y una fórmula resumen `required`:

| Campo o constructor | Significado |
| --- | --- |
| `required: None` | No se encontró una explicación admisible |
| `failure: NotAbducible` | No hay explicación permitida mediante seeds |
| `failure: NotForwardSupported` | Había candidatos, pero no una prueba ejecutable por el matcher forward |
| `Unit` | No falta evidencia en esa explicación |
| `Event(P)` | Falta el seed explícitamente admitido P |
| `Tensor(H1, H2)` | Ambos requisitos pertenecen a la misma rama |
| `Join(H1, H2)` | Alternativas de explicación |

`RequirementFormula` representa este fragmento lógico; no es un evento ni una
unión de `Known`. Cada explicación conserva su `Antecedent` y `ResidualProof`:
reglas aplicadas, objetivos intermedios, sustituciones, evidencia o hipótesis.
No tiene `DerivationTime`: una consulta no es una derivación.

El resumen consolida únicamente fórmulas con el mismo árbol, por idempotencia
del join. Conserva todas sus pruebas separadas. La simplificación del producto
elimina `I`, pero no elimina repeticiones de eventos. No compara fórmulas mediante
`≤_L`, no absorbe alternativas por preferencia y no selecciona explicaciones
mínimas. Por ejemplo, conserva tanto `A` como `A ⊗ B`, y también `I ∨ B` si dos
reglas producen esas ramas. Si el objetivo mismo está en la evidencia, la consulta
termina directamente en `I`, conforme a la reducción por evidencia anterior.

## Alcance de la búsqueda

Esta versión requiere objetivos y evidencia ground en el sentido runtime, y que
todos los parámetros de cada regla cuyo head haga match queden determinados por
ese head. Los identificadores y sus tipos runtime son una precondición del caller;
la API verifica declaración, aridad y clasificación de los verbos. No enumera
dominios de objetos para resolver variables libres en el body.

Si termina con éxito, conserva todas las ramas del procedimiento descrito que
pasan los filtros de admisibilidad y certificado forward, después de simplificar
eventos ya evidenciados. No enumera hipótesis superfluas añadidas
arbitrariamente, ni todos los elementos de `L`. Tampoco demuestra todavía una
realización del adjunto general. En particular, `close` no enumera valores para
parámetros presentes sólo en el head: la búsqueda puede instanciar esa lambda
desde el goal, pero su candidato se descarta si no pasa el replay forward.
El matcher forward y su replay reciben el scope de la regla. Comprueban que las
referencias de su plantilla estén ligadas antes de sustituir; una identidad runtime
ligada puede compartir nombre con un parámetro sin convertirse otra vez en una
variable pendiente. Un tiempo fuera del scope se compara como literal. La
[segunda auditoría](monotonic-core-audit-followup.md) registra las regresiones de
estas correcciones, sin ampliar la búsqueda ni prometer completitud general.

## Suficiencia como condición permanente

Cada explicación devuelta lleva una prueba finita. Sus hojas son eventos de
la evidencia `K`, hipótesis admisibles de `H`, o `I`. Para cada aplicación de regla,
`instantiate_from_witness` procesa exactamente los antecedentes en el orden
del cuerpo, con las mismas operaciones de matching, sustitución y comprobación
de parámetros que `instantiate_from_known`. No busca combinaciones ni suministra
bindings desde el goal para completar una instancia forward.

El replay exige que el evento resultante sea lógicamente el objetivo de ese paso.
Así, por inducción sobre la prueba:

```text
H devuelto por query(P,K,G) ⇒ G pertenece al least Horn closure de K ∪ H.
```

Cuando `close` termina, ese goal está en `close(K ∪ H, rules(P)).known`. Si el
cierre completo sería infinito por otras reglas, el certificado sigue siendo
una derivación finita del goal; la consulta no ejecuta ese cierre. No asigna
`DerivationTime`, modifica evidencia real ni ejecuta efectos. Este filtro no
elige una explicación mejor: conserva todas las pruebas que puede certificar.

Se distingue falta de explicación admisible de falta de soporte forward. No se
promete completitud fuera del fragmento certificado ni se confunde un requisito
`I` válido con un fallo.

La búsqueda falla explícitamente ante una variable no determinada, un ciclo de
objetivos lógicamente iguales o un límite agotado. Descarta la evaluación completa
con `Err`; nunca devuelve un prefijo de resultados como si fuese completo. Un
programa con defaults se rechaza: aquí no se asume una política de selección o
consistencia. Tampoco se ejecutan efectos ni se modifica `State`.

`SearchLimits` limita profundidad, nodos y ramas intermedias/finales. Los valores
por defecto son 32, 4096 y 4096; la profundidad admitida es de 1 a 64. Un ciclo
que genere tiempos distintos `t, after(t), ...` se detiene por profundidad aunque
no repita la misma identidad. Estos límites algorítmicos acompañan los límites
de proceso de `scripts/test_safe.py`; no los sustituyen.

Las métricas cuentan visitas a eventos/antecedentes, reglas examinadas, heads
compatibles y ramas construidas. Una regla examinada cuenta un intento de
unificación, no cada comparación individual de términos. Las ramas incluyen
las intermedias, no sólo las explicaciones finales. No hay caché ni indexación
de heads en esta versión. `rejected_not_forward_supported` cuenta los candidatos
descartados por el certificado.

## Verificación

La fixture `examples/residual/alternatives_formal.res` produce cuatro combinaciones:
`a ⊗ c`, `a ⊗ d`, `b ⊗ c` y `b ⊗ d`. Un oráculo forward finito recorre las 16
evidencias y las 16 extensiones posibles de sus cuatro seeds: el goal pertenece
al cierre hipotético exactamente cuando alguna explicación queda satisfecha.
Esto comprueba suficiencia y cobertura en ese modelo, sin asumir preferencia.

La fixture original `Legal.sales_delivery` se consulta como programa completo:
evidencia vacía requiere ambos acuerdos; sólo precio requiere objeto; ambos
acuerdos requieren `I`.
Ese `I` es un requisito neutro: no identifica el conocimiento disponible con
conocimiento vacío, ni con el valor de la unidad en un modelo FOUR.
La lambda especializada `delivery` conserva su origen;
`traditio` genérica se descarta porque su `conduct` no se resuelve en forward.
Otras fixtures siguen comprobando que dos reglas válidas con requisitos iguales
conservan pruebas distintas. Cada explicación Legal se valida en un cierre
hipotético separado. Ningún requisito o efecto se agrega a la evidencia real.

Otros tests cubren polaridad explícita, permiso de abducción, multiplicidad del
producto, cláusulas ground con `I`, ausencia de explicación, equivalencia entre
Horn y residual currificado, scope, ciclos y agotamiento de límites. Las pruebas
se ejecutan individualmente con el runner limitado; CNL y `Legal` no se modifican.

## Cierre de este bloque

Quedan congelados búsqueda y abducción monotónicas en este estado. Se conservan
el contrato algebraico y sus modelos, `Join`, `Tensor`, provenance de consulta,
límites y separación de hipótesis/evidencia. No hay `minimal`, `preferred`,
cardinalidad, otro orden ni selección mediante `≤_L`. El
[puente de valoración y satisfacción](satisfaction-bridge-experiment.md) comprueba
soporte positivo frente a matching y cierre ground. El posterior
[puente de teorías/modelos](theory-model-bridge.md) establece, en el fragmento
declarado, conservación de modelos al cerrar evidencia; ni Known ni el modelo
mínimo sustituyen la clase de modelos para fórmulas generales. La
[derivada aditiva de close](discrete-derivative.md) tiene una caracterización
propia. Esos resultados no amplían la búsqueda ni demuestran su completitud:
la relación objetivo general del residual sigue abierta.
