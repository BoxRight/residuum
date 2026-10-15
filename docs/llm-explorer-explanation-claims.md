# Campos de explicación del explorador: propuesta de instrucciones y esquema

Estado: **PROPUESTA, NO IMPLEMENTADA**. Complementa el
[protocolo de invocación](llm-explorer-invocation-protocol.md) (apartados 4 y 5)
y el [contrato](llm-guided-normative-exploration-contract.md) (4.4). No cambia
`D`, el perfil, las herramientas ni las vistas. Define cómo pedir al modelo
tres explicaciones y cómo registrarlas y cotejarlas sin convertirlas en
fuente de verdad sobre Residuum.

## 1. Principio

| Explicación | Quién es la fuente | Qué se le pide al modelo | Quién coteja |
|---|---|---|---|
| Qué ocurrió en Residuum | la traza del evaluador | dar cuenta de resultados que ya recibió, por referencia a identificadores visibles | el controlador, contra el registro completo |
| Qué cambió entre hipótesis | los resultados y, en B/C, la geometría del harness | cualquier condición puede afirmar una diferencia que sus propias observaciones permitan comprobar (A ejecutó dos candidatas y recibió resultados distintos). C añade relaciones certificadas: cobertura, particiones, discriminantes | el controlador, contra los resultados entregados; el harness, para las relaciones geométricas |
| Por qué investiga | el modelo | su razón, la distinción que espera examinar y, si la tiene, una **hipótesis jurídica** con sus fundamentos declarados | nadie la corrige; se registra como hipótesis y se compara después con lo que discriminó y con las valoraciones externas |

Una hipótesis jurídica del explorador es legítima y deseable: proponer por
qué una regla sería adecuada, señalar en qué se apoya y presentarla como
hipótesis. Lo que el explorador no puede hacer es certificar su propia
adecuación ni suministrar la valoración independiente que la validaría. La
distinción es entre *proponer* y *valorar*, no entre hablar o callar de
derecho.

Las instrucciones son idénticas en A, B y C. Lo único que cambia es qué
identificadores existen en el historial proyectado de cada condición. Si un
modelo en A habla de soportes que nunca recibió, eso es un dato del
experimento, no un error de prompt.

## 2. Bloque de instrucciones (texto para `explorer_instructions`)

```text
Explicaciones.

En cada acción rellena `rationale` y `expected_observation`. Son tuyas:
di por qué actúas y qué distinción esperas que el resultado separe.

Cuando describas lo que ocurrió en Residuum o lo que cambió entre
candidatas, hazlo en `claims`. Cada claim tiene una parte estructurada,
que se coteja automáticamente, y una explicación en lenguaje natural.
La parte estructurada nombra un identificador de tu historial (acción,
evento, regla, candidata, partición), un atributo y un valor. Afirma
sólo lo que hayas recibido; una diferencia entre dos resultados que tú
mismo obtuviste es un hecho observado y puedes afirmarla. Si no tienes
referencia, no afirmes; formula una expectativa en `expected_observation`.

Puedes proponer hipótesis jurídicas: por qué una regla sería adecuada y
en qué se fundamenta. Preséntalas como hipótesis en `legal_hypothesis`.
No las presentes como valoración: las valoraciones independientes llegan
de fuera y nunca las produces tú. Una candidata compatible con una
valoración no es por ello una candidata correcta.

Los claims se cotejan con el registro. Un claim que no coincida no
invalida tu acción; queda registrado como no coincidente.
```

## 3. Campos de esquema

Se añaden a todas las funciones del catálogo y a `finish_episode`. Modo
estricto, `additionalProperties: false`, longitudes acotadas por el
manifiesto.

```json
{
  "rationale": {"type": "string", "maxLength": 400},
  "expected_observation": {"type": ["string", "null"], "maxLength": 400},
  "legal_hypothesis": {
    "type": ["object", "null"],
    "properties": {
      "statement": {"type": "string", "maxLength": 400},
      "grounds": {"type": "array", "maxItems": 4, "items": {"type": "string", "maxLength": 200}},
      "status": {"type": "string", "enum": ["hypothesis"]}
    },
    "required": ["statement", "grounds", "status"],
    "additionalProperties": false
  },
  "claims": {
    "type": "array",
    "maxItems": 8,
    "items": {
      "type": "object",
      "properties": {
        "kind": {"type": "string", "enum": ["residuum_result", "hypothesis_change", "investigation_reason"]},
        "assertion": {
          "type": ["object", "null"],
          "properties": {
            "ref": {"type": "string", "maxLength": 80},
            "attribute": {"type": "string", "enum": ["status", "component", "contains_event", "classes", "compatible", "differs_from"]},
            "value": {"type": "string", "maxLength": 120}
          },
          "required": ["ref", "attribute", "value"],
          "additionalProperties": false
        },
        "text": {"type": "string", "maxLength": 300}
      },
      "required": ["kind", "assertion", "text"],
      "additionalProperties": false
    }
  }
}
```

La `assertion` es lo que se coteja: `ref` es un identificador que el
controlador asignó y que figura en el historial proyectado de esa condición
(`a12`, `c3`, `p2`); `attribute` es uno de los atributos con regla de cotejo;
`value` es el valor afirmado en la forma que la regla espera. Ejemplo:

```text
assertion:  ref = a12, attribute = component, value = "1=true"
text:       «La candidata produjo el impedimento en el expediente histórico.»
```

La frase es explicación; la `assertion` es lo que `match` significa. Un claim
con `assertion` nula es sólo explicación y queda `unchecked`. `legal_hypothesis`
nunca se coteja: se registra con `status = hypothesis` y se relaciona después
con las valoraciones externas.

En `finish_episode` se pide además un claim de tipo `hypothesis_change` por
cada candidata que referencie, resumiendo qué la distingue de las demás
supervivientes **según lo recibido**.

## 4. Cotejo y registro

Paso 7 del turno (protocolo, apartado 6): tras obtener el resultado completo
y antes de proyectar, el controlador evalúa cada claim:

| Verdicto | Condición |
|---|---|
| `match` | la `assertion` tiene `ref` en el historial proyectado de la condición y la regla de cotejo de su `attribute` confirma `value` contra el registro |
| `mismatch` | `ref` existe en la vista pero la regla contradice `value` |
| `unreferenced` | `assertion` nula, o `ref` no presente en ningún registro |
| `out_of_view` | `ref` existe en el registro completo pero no en la vista de esa condición |
| `unchecked` | `attribute` sin regla en esa versión, o claim sin `assertion` |

Reglas de cotejo deterministas y versionadas, por atributo: `status`
(igualdad con el estado registrado), `component` (índice y valor de `D`),
`contains_event` (pertenencia al resultado), `classes` (número de clases de
la partición), `compatible` (valor de `κ`), `differs_from` (dos referencias
cuyos resultados registrados no son iguales; válido en cualquier condición
porque compara resultados que el modelo recibió). El cotejo verifica la
`assertion`, no la frase: la fidelidad semántica del texto permanece
`unchecked` salvo que proceda de una plantilla controlada. El cotejo no usa
un segundo LLM.

`out_of_view` identifica una referencia no entregada en esa vista. Por sí
solo no demuestra acceso a información reservada: puede ser una conjetura
acertada, un identificador adivinado por su forma o un error. Se cuenta y se
inspecciona; no se interpreta como fuga.

Se escribe en `claims.jsonl`: `episode_id`, `action_id`, `kind`, `text`,
`ref`, `verdict`, `rule_version`. Los claims nunca entran en `progress.jsonl`
ni en `valuations.jsonl`.

## 5. Métrica

Fidelidad explicativa: fracción de claims `match` sobre claims con
`assertion` cotejable, por `kind`, `attribute` y condición; `unreferenced`,
`out_of_view` y `unchecked` se informan aparte. Es independiente de las cinco métricas del contrato: mide si el
modelo describe correctamente lo que el sistema hizo, no si acierta
normativamente. Un `out_of_view` en A es la señal de que el modelo afirma
algo que su vista no le dio; se cuenta, no se penaliza en la métrica
normativa.

## 6. Lo que no se pide

- Cadena de pensamiento interna. Sólo razón declarada, expectativa, hipótesis
  jurídica y claims.
- Que el modelo certifique la adecuación jurídica de su propia regla o
  suministre la valoración independiente. Proponer una hipótesis jurídica
  fundamentada sí se pide.
- Claims sobre el registro completo del auditor o sobre certificados que la
  condición no entrega.
- Que el modelo corrija sus claims tras el cotejo. El verdicto se registra;
  no se devuelve al modelo en la primera versión, para no convertir el cotejo
  en una herramienta de feedback adicional que diferenciaría condiciones.
