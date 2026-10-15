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
| Qué cambió entre hipótesis | la geometría del harness | en C, leer las diferencias que recibió; en A y B, declarar expectativas, no hechos | el harness, con fronteras y particiones reales |
| Por qué investiga | el modelo | su razón y la distinción que espera examinar | nadie la corrige; se compara después con lo que discriminó |

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
candidatas, hazlo en `claims`. Cada claim es una frase breve con la
referencia exacta al identificador de tu historial que la sostiene
(acción, evento, regla, candidata, partición). No refieras nada que no
hayas recibido. Si no tienes referencia, no afirmes; formula una
expectativa en `expected_observation`.

No expliques por qué una regla es jurídicamente adecuada. Las
valoraciones llegan de fuera y nunca las produces tú. Una candidata
compatible con una valoración no es una candidata correcta.

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
  "claims": {
    "type": "array",
    "maxItems": 8,
    "items": {
      "type": "object",
      "properties": {
        "kind": {"type": "string", "enum": ["residuum_result", "hypothesis_change", "investigation_reason"]},
        "text": {"type": "string", "maxLength": 300},
        "ref": {"type": ["string", "null"], "maxLength": 80}
      },
      "required": ["kind", "text", "ref"],
      "additionalProperties": false
    }
  }
}
```

`ref` apunta a un identificador que el controlador asignó y que figura en el
historial proyectado de esa condición: `a12` (acción), `a12.components[1]`,
`c3` (candidata), `p2.classes` (partición), `e:testamentaryEntitlement@a7`
(evento dentro de un resultado). El formato exacto lo fija el manifiesto; el
modelo no inventa identificadores.

En `finish_episode` se pide además un claim de tipo `hypothesis_change` por
cada candidata que referencie, resumiendo qué la distingue de las demás
supervivientes **según lo recibido**.

## 4. Cotejo y registro

Paso 7 del turno (protocolo, apartado 6): tras obtener el resultado completo
y antes de proyectar, el controlador evalúa cada claim:

| Verdicto | Condición |
|---|---|
| `match` | `ref` existe en el historial proyectado de la condición y el texto es consistente con el valor registrado según una regla de cotejo declarada para ese tipo de referencia |
| `mismatch` | `ref` existe pero el valor registrado contradice el texto |
| `unreferenced` | `ref` nulo o no presente en el historial proyectado |
| `out_of_view` | `ref` existe en el registro completo pero no en la vista de esa condición |

Las reglas de cotejo son deterministas y versionadas: igualdad de estado,
pertenencia de un evento a un resultado, valor de un componente de `D`,
número de clases de una partición. Lo que no tenga regla se registra como
`unchecked`, no como `match`. El cotejo no usa un segundo LLM.

Se escribe en `claims.jsonl`: `episode_id`, `action_id`, `kind`, `text`,
`ref`, `verdict`, `rule_version`. Los claims nunca entran en `progress.jsonl`
ni en `valuations.jsonl`.

## 5. Métrica

Fidelidad explicativa: fracción de claims `match` sobre claims con referencia,
por `kind` y por condición; `unreferenced` y `out_of_view` se informan
aparte. Es independiente de las cinco métricas del contrato: mide si el
modelo describe correctamente lo que el sistema hizo, no si acierta
normativamente. Un `out_of_view` en A es la señal de que el modelo afirma
algo que su vista no le dio; se cuenta, no se penaliza en la métrica
normativa.

## 6. Lo que no se pide

- Cadena de pensamiento interna. Sólo razón declarada, expectativa y claims.
- Explicación de adecuación jurídica. Es la valoración externa.
- Claims sobre el registro completo del auditor o sobre certificados que la
  condición no entrega.
- Que el modelo corrija sus claims tras el cotejo. El verdicto se registra;
  no se devuelve al modelo en la primera versión, para no convertir el cotejo
  en una herramienta de feedback adicional que diferenciaría condiciones.
