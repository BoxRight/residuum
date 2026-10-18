# Protocolo de invocación y control del explorador LLM

Estado: **ESPECIFICACIÓN PROPUESTA, PARCIALMENTE IMPLEMENTADA**. Redactada
sobre la base `f3091c7` del worktree `experiment-llm-normative-exploration-spec`
e incorporada al historial más tarde con las rutas actualizadas. Lo que ya
existe en código, con su verificación en
[`docs/legal-geometry/exploration/verification.md`](legal-geometry/exploration/verification.md):
las etapas separadas de propuestas (§5.1 resuelto), el trabajador en proceso
(`residuum::experiments::worker`), los DTOs de frontera y el transporte por
líneas en el crate `residuum-boundary` (`boundary/src/wire.rs`,
`boundary/src/transport.rs`), el binario mínimo `residuum-worker` sin
ejecutar, y la selección `--test-target boundary` del runner. Siguen fuera
el modo acotado de ejecución del proceso (pendiente de revisión), el
controlador Python, el ledger y el proveedor. No se realizaron llamadas de
inferencia.

Este documento concreta la invocación del modelo dentro de
[la especificación experimental](llm-guided-normative-exploration-specification.md)
y [el contrato de exploración](llm-guided-normative-exploration-contract.md).
No sustituye las restricciones de esos documentos ni modifica la interpretación
de recalcitrancia, las reglas jurídicas o el contrato de decisión `D`.

## 1. Decisión arquitectónica y ubicación de la llamada

Un controlador externo en Python invoca un único modelo por episodio mediante
una API. Un trabajador Rust conserva las sesiones y llama a las operaciones
existentes de Residuum. El modelo no es un subagente de Codex/Claude Code en
esta primera comparación: cambiar de proveedor o añadir subagentes constituiría
otra condición experimental.

```text
Manifiesto del episodio + valoraciones independientes
                         |
                         v
                Controlador Python
              /         |          \
      registro       proveedor      puente de herramientas
      privado        de modelo                |
                         |                    v
                         v             trabajador Rust acotado
                    API del LLM        sesiones, propuestas,
                         |             certificados y geometría
                         v                    |
                 petición de acción           v
                         |             resultado completo
                         v                    |
              validación del controlador <---+
                         |
                 proyección A/B/C
                         |
                 siguiente llamada
```

Distribución propuesta; ninguno de estos archivos Python existe todavía:

| Archivo futuro | Responsabilidad |
|---|---|
| `tools/normative_explorer/controller.py` | Estado del episodio, autorización de acciones y ciclo de interacción. |
| `tools/normative_explorer/provider_openai.py` | Único lugar que crea el cliente e invoca `client.responses.create(...)`. |
| `tools/normative_explorer/schemas.py` | Esquemas versionados de herramientas y respuestas; sin inferencia jurídica. |
| `tools/normative_explorer/projections.py` | Construir las vistas A/B/C desde resultados completos mediante listas explícitas de campos. |
| `tools/normative_explorer/bridge.py` | IPC con el trabajador Rust (`residuum-worker`, crate `boundary/`): líneas JSON de `boundary/src/wire.rs`; no implementa matching, sustitución, cierre ni fronteras. |
| `tools/normative_explorer/ledger.py` | Registro append-only, referencias de artefactos y reconstrucción de historia. |
| `tools/normative_explorer/manifest.py` | Validar y congelar la configuración antes del primer turno. |

La llamada de red no pertenece a `derive`, `residual`, `elab`, `typed`, al
runtime ni a `experiments::exploration`. El proceso del modelo tampoco recibe
un shell, un editor o acceso directo al worktree. Sus acciones pasan por las
herramientas del protocolo. Estas son restricciones del experimento, no
capacidades que debamos eliminar del lenguaje.

Codex/Claude Code pueden construir o revisar esta infraestructura fuera de
los episodios medidos. Sus conversaciones y propuestas no entran como ayuda
no registrada a una condición experimental.

## 2. Qué manda el contrato y qué puede elegir el modelo

Antes de invocar la API, el controlador valida un manifiesto local. Debe fijar:

- Teoría y módulo de problema, archivos permitidos, hashes y versiones.
- Gramática/perfil, cotas, constantes, cabezas y planes de instalación.
- Expedientes admitidos y cómo se validan expedientes propuestos nuevos.
- Interfaz de resultados, `D`, criterios `kappa` y sus versiones.
- Fuente independiente de valoraciones, estado inicial de observaciones y
  reserva de casos no accesibles al explorador.
- Modelo exacto, proveedor, versión del cliente, parámetros soportados,
  política de contexto, herramientas, condición A/B/C y presupuestos.
- Versiones del protocolo, proyectores y formatos de auditoría.

Si `D` requiere instancias ground por expediente, se congela su plantilla y
el procedimiento de bindings; cada instancia queda registrada. No se permite
cambiar qué se considera éxito al cambiar de candidato.

Estos campos no son argumentos de herramientas del LLM. El controlador
rechaza una solicitud que intente cambiar la condición, los imports, el perfil,
`D`, los seeds reales o la fuente de valoraciones. Una modificación autorizada
por una persona inicia un episodio nuevo con otro manifiesto; no reescribe el
historial del episodio anterior.

El modelo sí puede elegir consultas y bindings dentro del perfil, construir
candidatos, sugerir movimientos de búsqueda, solicitar análisis permitidos y
proponer casos para valoración externa. Elegir una propuesta no le permite
aprobarla normativamente ni convertir una hipótesis abductiva en evidencia real.

No se necesita que el manifiesto contenga todo el espacio de candidatos
enumerado. Debe contener una gramática finita verificable y sus restricciones.
La generación puede ampliar el archivo de candidatos durante el episodio.

## 3. Proveedor inicial y llamada de API

Para la primera implementación se propone OpenAI Responses con funciones
personalizadas. En este flujo el modelo solicita una función y la aplicación
la ejecuta y devuelve su resultado; registrar una función no delega su
ejecución automáticamente al proveedor.
[Documentación oficial de function calling](https://developers.openai.com/api/docs/guides/function-calling).

Esqueleto conceptual de `provider_openai.py`; no es código instalado ni un
contrato completo del cliente:

```python
def request_next_action(client, manifest, visible_history, tools, cap):
    return client.responses.create(
        model=manifest.model_id,
        instructions=manifest.explorer_instructions,
        input=visible_history,
        tools=tools,
        tool_choice="required",
        parallel_tool_calls=False,
        max_output_tokens=cap,
        stream=False,
        store=False,
    )
```

Decisiones del protocolo:

1. Un turno solicita una acción. Hay como máximo una llamada de herramienta
   por respuesta; el controlador verifica también esa cardinalidad. El proveedor
   documenta `parallel_tool_calls=False` para limitar las llamadas a cero o una.
2. `finish_episode` es una herramienta explícita para terminar. Un mensaje
   libre no instala una regla, no ejecuta una operación ni certifica éxito.
3. Los esquemas usan modo estricto, campos obligatorios y
   `additionalProperties: false`. Las restricciones de identificadores,
   presupuestos, scopes y tipos siguen requiriendo validación local.
4. Modelo y configuración se fijan antes de A/B/C. No hay sustitución silenciosa
   de modelo, proveedor, parámetros o tipo de API ante un error.
5. No se ofrecen herramientas integradas de shell, web, archivos, búsqueda de
   documentos, ejecución de código o delegación a subagentes. La lista contiene
   exclusivamente las funciones de este experimento.
6. La llamada no transmite credenciales en el prompt. El cliente las recibe
   mediante la configuración local prevista; los logs nunca contienen claves
   ni cabeceras de autorización.

Los parámetros y esquemas concretos deben validarse contra el modelo elegido
antes de un episodio medido. No se presupone que todos los modelos admitan
temperatura, semilla o parámetros de razonamiento idénticos. La referencia de
[creación de respuestas](https://developers.openai.com/api/reference/python/resources/responses/methods/create)
documenta los campos de la API, pero no convierte el esqueleto en una ejecución
verificada. Elegir y pagar un modelo no forma parte de esta etapa documental.

## 4. Contexto inicial e historia conversacional

El controlador construye el contexto inicial a partir del manifiesto y de
datos autorizados, no de una copia indiscriminada del repositorio. Incluye:

- Vocabulario, firmas, reglas y programas de la teoría autorizada, con la misma
  representación en A/B/C; identificadores concretos y tiempos se conservan.
- Perfil de construcción, objetivos, `D`, límites y significado de los estados.
- Expedientes iniciales y observaciones externas de entrenamiento con sus
  fundamentos y alcance; las etiquetas no aparecen como seeds.
- Resultado basal disponible según la vista y referencias consultables por
  herramientas; los certificados privados no se adjuntan a A.
- Instrucciones para expresar una razón breve y una expectativa contrastable
  en cada acción. No se solicita una cadena de pensamiento interna.

El catálogo de 64 reglas, documentos de soluciones, transcripciones de otros
episodios, certificados reservados y valoraciones de prueba no se incorporan
por estar en el mismo repositorio. Toda exposición adicional queda registrada.
La teoría común puede permitir reconstruir alguna información geométrica;
la comparación mide el efecto del feedback suministrado, no una imposibilidad
de que A lo razone por su cuenta.

En el primer protocolo se mantiene la conversación de forma explícita en el
controlador: entrada inicial, items de salida del proveedor y resultados de
funciones por `call_id`. No se mezcla `previous_response_id` de otro episodio
ni se comparte una conversación del proveedor entre condiciones.

La API permite historia gestionada por el cliente con `store=False`; deben
conservarse los items de salida necesarios para continuar, no sólo el texto
visible. Cuando un modelo requiera contenido opaco de continuación, el
adaptador seguirá su mecanismo documentado y guardará esos items sin
interpretarlos como explicación jurídica.
[Documentación oficial de estado conversacional](https://developers.openai.com/api/docs/guides/conversation-state).

`store=False` no se presenta como garantía general de ausencia de retención
del proveedor. La política de datos aplicable se registra por separado antes
de enviar datos. Este protocolo inicial utiliza expedientes experimentales
autorizados, no expedientes privados de `12hectareas`.

Inicialmente no hay resumen automático ni compactación silenciosa. Si el
contexto no cabe, se termina con `context_limit`. Otra política de reducción
de contexto requiere versión propia, transcripción de lo retirado y el mismo
tratamiento en las condiciones comparadas.

## 5. Acciones y puente con el contrato existente

Cada función recibe sus argumentos específicos más `rationale` y
`expected_observation` como textos breves acotados. Los identificadores de
episodio, condición, request, acción, candidato nuevo y artefacto los asigna
el controlador; el LLM sólo referencia objetos ya visibles. La primera versión
no ofrece un campo genérico para comandos o rutas arbitrarias.

| Acción propuesta | Operación y control |
|---|---|
| `inspect_theory(theory_ref)` | Inventario del módulo autorizado. No lee documentación o rutas elegidas libremente por el modelo. |
| `begin_forward(dossier_id, theory_ref)` | Crea `ForwardSession::for_module`; devuelve `session_id`. Todavía no declara cierre completo. |
| `begin_residual(dossier_id, theory_ref, goal, scope, bindings, limits, allowed)` | Valida con la infraestructura existente y crea una sesión residual con bindings. |
| `begin_active_residual(...)` | Usa la sesión activa y su proyección guardada existente. Los bindings no son triggers ni seeds. |
| `advance_session(session_id, max_steps)` | Avanza una sesión autorizada. No cambia sus entradas, reglas o bindings. |
| `propose_candidate(source, parent_id, move)` | Parsea y elabora reglas Formal en copia privada, comprueba perfil, conserva texto y etapas. No aprueba jurídicamente la propuesta. |
| `begin_candidate_evaluation(candidate_id, dossier_id)` | Prepara la observación basal e hipotética, decisión y certificación según su alcance; el trabajo avanza mediante la sesión. |
| `request_valuation(case_spec, expected_discrimination)` | Registra la pregunta y devuelve `pending`; el LLM no suministra la respuesta independiente. |
| `minimal_supports(theory_ref, goal, domain_id)` | B/C: análisis sobre un dominio finito validado. No es automáticamente compatible con Programs. |
| `compare_coverage(candidate_a, candidate_b, domain_id)` | C: comparación certificada relativa al dominio. |
| `partition(candidate_ids, observation_ids)` | C: exige carrier e identidades en el mismo orden; distingue generación y refinamiento. |
| `discriminants(candidate_ids, domain_id)` | C: casos realizables y distinciones verificadas; minimax sólo si se implementa y verifica. |
| `finish_episode(candidate_ids, explanation)` | Finaliza la exploración; el veredicto lo produce el controlador desde certificados y valoraciones. |

Las acciones nuevas son una propuesta de interfaz para las operaciones de
§3 del contrato, no endpoints implementados. La separación `begin/advance`
explicita persistencia, presupuesto y estados. Los análisis geométricos son
cálculos finitos del harness; no se hacen pasar por pasos del operador forward.

Ejemplo de función para enviar al proveedor, sujeto a la validación del modelo
elegido. El límite efectivo puede ser menor que 4096 según el manifiesto y el
presupuesto restante:

```json
{
  "type": "function",
  "name": "advance_session",
  "description": "Avanza una sesión existente bajo el contrato del episodio.",
  "strict": true,
  "parameters": {
    "type": "object",
    "properties": {
      "session_id": {"type": "string"},
      "max_steps": {"type": "integer", "minimum": 1, "maximum": 4096},
      "rationale": {"type": "string"},
      "expected_observation": {"type": ["string", "null"]}
    },
    "required": ["session_id", "max_steps", "rationale", "expected_observation"],
    "additionalProperties": false
  }
}
```

El modelo puede pedir, por ejemplo, un paso de `s17`; el controlador comprueba
que esa sesión pertenece al episodio, fue creada con el manifiesto congelado,
sigue disponible y tiene presupuesto. La existencia de la sesión y el tipo
de sus términos no quedan establecidos por el esquema JSON. La explicación
no altera los argumentos ejecutables ni concede permisos adicionales.

### 5.1 Adaptaciones pendientes que no deben ocultarse

El código local `experiments::proposal::process` recorría parseo,
elaboración, admisión y evaluación completa en una sola llamada. Esto quedó
resuelto en `411ad1b`: `Episode::prepare`, `admit`, `check_scope`,
`evaluate_admitted` y `EvaluationSession` son etapas separadas y `process`
su composición. No se permite copiar el algoritmo en Python ni ejecutar el
cálculo ocultándolo como mero parseo.

`experiments::exploration::evaluate` también ejecuta un `close_program`
completo. No es una implementación de avance por un paso. La sesión de
evaluación debe reutilizar `ForwardSession` para observación y la misma lectura
validada de `Decision` para su resultado. Cualquier extracción de esa lectura
será un helper compartido, no otra definición de `D`.

La certificación activa existente ejecuta cierres hipotéticos completos al
final de una búsqueda. La evaluación independiente puede ejecutar cierres de
referencia completos. Este trabajo es del verificador, está sujeto al runner
y se registra aunque el modelo sólo controle avances acotados. Un paso no
garantiza tiempo constante. Sin certificación completada no se emite éxito.

El certificado actual no cubre arbitrariamente cabezas existentes o Program
admission. Un rechazo `out_of_certificate_scope` no es rechazo ontológico ni
incompatibilidad normativa. El controlador no debe convertirlo en eliminación
del espacio de hipótesis por una valoración externa.

## 6. Máquina de estados y orden de ejecución

```text
CREATED → PREFLIGHT → READY
READY → MODEL_REQUESTED → MODEL_RECEIVED
MODEL_RECEIVED → ACTION_VALIDATED → TOOL_RUNNING
TOOL_RUNNING → RESULT_RECORDED → VIEW_RECORDED → READY
READY → WAITING_VALUATION → READY
READY → FINISHED | BUDGET_EXHAUSTED | ABORTED
```

`WAITING_VALUATION` representa una pausa cuando la acción siguiente depende de
una respuesta externa. Si el protocolo permite otras investigaciones mientras
se espera, se mantienen como acciones normales sin inventar esa valoración.

Orden obligatorio por turno:

1. Verificar que hashes, `D`, perfil y condición siguen siendo los congelados.
2. Reservar presupuesto y escribir `model_request_prepared` con el cuerpo que
   se enviará, sin credenciales. Persistir antes de iniciar la red.
3. Invocar `provider_openai.request_next_action` una sola vez y registrar
   respuesta, IDs, estado y uso reportado.
4. Comprobar respuesta completa, una llamada permitida, esquema, tamaños,
   referencias y presupuesto. Texto libre no se convierte por inferencia del
   controlador en una llamada de herramienta.
5. Registrar intención y argumentos validados antes de ejecutar el trabajador.
6. Obtener y persistir el resultado completo o el fallo observado por el runner.
7. Evaluar los claims comprobables y aplicar la proyección A/B/C. Registrar
   exactamente la respuesta proyectada y su hash.
8. Añadir los items de salida necesarios y el resultado de función, asociado
   al `call_id`, a la historia del modelo. Sólo entonces iniciar otro turno.

No hay ejecución paralela inicial. Proponer y evaluar candidatos no modifica
una sesión que ya estaba abierta sobre otra teoría. Para cambiar teoría,
bindings o expediente se abre otra sesión con identidad propia.

## 7. Errores, límites y estados jurídicos

El controlador separa fallos de transporte/protocolo de los resultados de
Residuum:

| Situación | Tratamiento |
|---|---|
| JSON inválido, función desconocida, referencia ajena o condición no autorizada | `protocol_error`; no se ejecuta una herramienta. El intento consume presupuesto y queda registrado. |
| HTTP fallido, timeout de API o respuesta incompleta | `model_error` o `model_incomplete`; no se ejecutan argumentos parciales. |
| Error de manifiesto, configuración de `D` o hashes cambiados | `harness_error`; episodio suspendido o abortado, nunca candidata incompatible. |
| `S_SYNTAX`, `S_ELABORATION`, rechazo de perfil | Conservar el código y etapa de la ruta de propuestas; no afirmar desacuerdo normativo. |
| Fuera de alcance del certificador | `out_of_certificate_scope`; candidata no evaluada normativamente. |
| Límite de pasos o búsqueda, parámetros pendientes, ciclo residual | Conservar el estado del contrato; no ausencia de derivabilidad. |
| Timeout/OOM o pérdida del trabajador | `runner_limit`/`worker_lost`; sin resultado jurídico completo. |
| Evaluación y certificados completos | Leer `D` y `kappa` fijos; la compatibilidad se informa con el objetivo y alcance declarados. |

Un resultado vacío no se interpreta uniformemente: el adaptador conserva la
distinción entre `I`, ausencia de explicación y condiciones pendientes de la
API correspondiente. Un fallo técnico no crea una experiencia recalcitrante.

En la primera versión no se ejecutan reintentos automáticos ocultos del SDK.
Un intento de red fallido queda en el historial; un eventual reintento es
explícito, consume presupuesto y no reutiliza ambiguamente un resultado.
Si el proveedor pudo haber computado pero no hay respuesta ni uso conocido,
se registra `usage_unknown`; no se asume coste cero ni se reintenta sin límite.

## 8. Presupuestos y proceso acotado de Residuum

El manifiesto requiere valores concretos antes de iniciar un episodio; no
hay valores infinitos ni aumentos automáticos. Debe distinguir:

- Máximo de requests de modelo, acciones, tokens de entrada/salida y duración
  de API. Cada turno consume también la historia reenviada.
- Tokens opacos/razonamiento y caché sólo según lo reportado por el proveedor;
  no estimarlos como razonamiento visible ni asumir determinismo por una semilla.
- Pasos por acción y por episodio, sesiones concurrentes, límites de búsqueda,
  tamaño de fuente, candidatos y solicitudes de valoración.
- Coste real de geometría, certificaciones y cierres internos. Donde el motor
  no exponga el número de transiciones se registra como desconocido, además
  de llamadas y tiempo; un cierre entero no se contabiliza como una sola ronda.

Antes de cada request se comprueba y reserva la capacidad restante. El tope
de salida se reduce al presupuesto disponible. Una restricción monetaria
requiere tarifas y política de estimación versionadas, pero no se presenta
como garantía exacta de facturación ante uso desconocido.

El trabajador Rust debe ejecutarse con los controles vigentes: 1 GiB total,
sin swap, un trabajo de Cargo y un hilo del harness durante compilación/tests.
No se lanzan trabajadores concurrentes que multipliquen ese presupuesto. El
diseño del proceso controlador y trabajador debe contabilizar su uso local
conjunto, no dar 1 GiB independiente a cada proceso. La inferencia remota tiene
presupuestos API separados; no es un proceso local bajo ese cgroup.

El `scripts/test_safe.py` inspeccionado no tenía transporte para un trabajador
persistente de herramientas. `--binary` es una ruta de binarios de tests, no
una autorización para ejecutar servidores arbitrarios. La CLI existente inicia
una sesión nueva por invocación y no implementa este protocolo IPC. Desde
`a843126` el runner selecciona el crate `residuum-boundary` para compilar y
probar; el modo acotado que lance `residuum-worker` con un manifiesto y un
archivo finito de peticiones se propone en un commit separado y no se usa
hasta su revisión.

Por tanto, integrar el trabajador y su transporte en un modo acotado del runner
es un requisito explícito pendiente. Debe revisarse sin cambiar límites ni
romper las rutas de tests/fixtures existentes. No se usará `subprocess` directo
sin controles, un fallback fuera del cgroup o un test como servidor encubierto
para sortear esa carencia. Hasta entonces sólo procede un simulador del
proveedor/transporte y pruebas individuales autorizadas.

## 9. Sesiones, identidades y transporte propuesto

El controlador posee el mapa de identificadores del episodio. El trabajador
posee los objetos Rust `ForwardSession`, `ResidualSession` y
`ActiveResidualSession`; no hay implementación Python de sus pasos.

Transporte inicial propuesto: mensajes JSON enmarcados por línea sobre stdin/
stdout de un trabajador local acotado, una petición en vuelo. stdout contiene
sólo protocolo; diagnósticos van a stderr y al registro privado. El formato
JSON no representa directamente todas las estructuras tipadas de Rust: el
adaptador debe definir DTOs explícitos y volver a validar mediante las APIs
existentes. `Debug` no es un protocolo de reanudación.

Cada solicitud interna contiene `protocol_version`, `episode_id`, `action_id`,
`manifest_hash`, operación y argumentos validados. Cada respuesta identifica
la acción y el estado, métricas disponibles y artefactos. El trabajador valida
otra vez sus referencias y el manifiesto; un `candidate_id` no es una ruta
que el modelo pueda elegir.

Los imports de propuestas proceden del manifiesto. Antes de resolver archivos,
la frontera debe rechazar declaraciones fuente fuera de la gramática admitida,
incluidos imports adicionales, usando el AST de parseo existente. Los errores
proyectados no pueden exponer archivos de reserva. El proyecto de serialización,
validación y transporte aún no está implementado; no se introduce un AST
jurídico paralelo.

La pareja `(episode_id, action_id)` identifica una ejecución. Repetir la misma
entrega con el mismo contenido devuelve el resultado ya registrado; otro
contenido con esa identidad es un error. Dos acciones nuevas con argumentos
iguales pueden ser dos avances deliberados y conservan identidades distintas.

Si se pierde el trabajador después de avanzar y antes de confirmar el resultado,
no se repite a ciegas ese avance. El episodio queda `recovery_required`. La
recuperación requiere reconstrucción explícita desde entradas y acciones
registradas y comparación con las observaciones previas; no se promete que
exista ya una serialización persistente de las sesiones.

## 10. Vistas A/B/C y control efectivo

La condición se fija una sola vez. Se aplican dos controles:

1. Catálogo de funciones: A no recibe herramientas exclusivas B/C, B no recibe
   las exclusivas C. El despachador rechaza también nombres inventados que no
   figuren en ese catálogo.
2. Proyección de resultados: cada respuesta, error, resumen y contexto se
   construye desde una lista permitida de campos. No basta con incluir una
   instrucción en el prompt o esconder visualmente un campo.

Los nombres de reglas y sus cuerpos en la teoría común no son secretos. Sí
se controla la entrega de provenance calculada, soportes certificados,
particiones, discriminantes y documentos privados. Los artefactos completos
del auditor no son rutas consultables por el explorador.

El backend verifica y registra la misma semántica en las tres condiciones.
B y C pueden consumir más trabajo y tokens al recibir feedback adicional;
se informa ese coste. No se añaden propuestas redactadas por un segundo LLM
exclusivamente a C. Las comparaciones se hacen entre episodios separados, sin
compartir historial ni candidatos descubiertos.

## 11. Registro persistente y reproducción

Directorio propuesto de resultados de un episodio, fuera del contexto visible:

```text
episode/
  manifest.json
  events.jsonl
  model-requests.jsonl
  model-responses.jsonl
  actions.jsonl
  evaluator-trace.jsonl
  visible-views.jsonl
  candidates.jsonl
  partitions.jsonl
  valuations.jsonl
  progress.jsonl
  artifacts/sha256/...
```

`events.jsonl` ordena los sucesos e identifica referencias en los registros
especializados; no hay dos historiales independientes que puedan discrepar.
Cada entrada tiene ID, secuencia, timestamp, evento previo o su hash, versión
de manifiesto y hashes de artefactos. Es append-only. Un cambio se añade como
nuevo evento; no reemplaza la decisión anterior. Los hashes detectan cambios
en lo conservado, no prueban por sí mismos la verdad o integridad externa del log.

Guardar en particular:

- Cuerpo exacto enviado al proveedor y respuesta serializada completa, sin
  secretos; IDs de request/response/call, estado, uso y parámetros efectivos.
- Catálogo y esquemas exactos de herramientas de ese turno, instrucciones y
  todos los resultados proyectados. Guardar lo que vio el LLM además del audit
  completo; el prompt no se reconstruye sólo con un resumen posterior.
- Fuente original propuesta y módulo privado, sus hashes, reglas elaboradas,
  etapas, orígenes y certificados. No borrar variantes de igual cobertura.
- Carrier y orden de hipótesis de cada partición, dominio de cobertura,
  fronteras y procedencia del cálculo.
- Valoraciones externas y su fundamento, sin convertirlas en hechos de Known.
- Progreso inferencial, geométrico y normativo acreditado por el verificador,
  separado de lo que el modelo esperaba obtener.
- Fallos, rechazos, presupuestos agotados y candidatos pendientes. El valor
  concreto de `derivedAt` se guarda, aunque al comparar ejecuciones se normalice
  únicamente ese sello según el contrato.

No se registra ni exige razonamiento interno oculto. Se conserva la explicación
declarada y cualquier contenido de continuación opaco que requiera el protocolo,
sin atribuirle significado jurídico.

Reproducción tiene dos alcances distintos: replay de acciones contra Residuum
para verificar observaciones y replay del registro sin red para auditar vistas.
Volver a llamar al LLM no garantiza las mismas decisiones. Una nueva llamada
es otra ejecución, aunque use el mismo prompt y configuración.

## 12. Valoración, actualización y fin del episodio

El controlador compara únicamente evaluaciones completadas y certificadas con
valoraciones externas resueltas bajo el `D` fijo. Mantiene separados:

- Candidatos propuestos y estructuralmente admitidos.
- Candidatos con evaluación completa y su compatibilidad por observación.
- Pendientes por evidencia, bindings, valoración, recursos o alcance de prueba.
- Candidatos rechazados estructuralmente y las razones de rechazo.

Una nueva valoración se aplica a todas las candidatas evaluables; una candidata
nueva se comprueba contra todo el historial resuelto. No se sustituye el dominio
de una partición sin identificar el nuevo carrier. El modelo puede solicitar
refinamientos, pero las operaciones matemáticas y sus límites los verifica el
harness. La geometría orienta la exploración; no produce por sí misma etiquetas
jurídicas para los casos discriminantes.

`finish_episode` contiene referencias a candidatos y una explicación breve.
El controlador termina con uno de estos resultados, indicando alcance:
`verified_compatible_candidate`, `no_verified_candidate`,
`awaiting_independent_valuation`, `budget_exhausted`, `technical_abort` o
`recovery_required`. Ninguno se traduce automáticamente como descubrimiento de
la justificación desconocida o aprobación de una regla.

Para Riggs, una compatibilidad con el impedimento propuesto se identifica como
tal. No se presenta como exclusión efectiva ni como propagación del remedio a
activos y titularidad. Si el objetivo futuro exige esa exclusión, requiere otro
`D` y otro episodio; no se cambia la etiqueta durante la búsqueda.

## 13. Orden de implementación y pruebas necesarias

No se implementa nada por efecto de este documento. La continuación técnica
debería tener estos incrementos revisables:

1. Esquemas, manifiesto, ledger y proveedor simulado: sin red ni costes de API.
2. Proyectores A/B/C y dispatcher; comprobar rechazos y ausencia de campos
   reservados en todas las salidas, incluidos errores.
3. Adaptaciones compartidas de propuesta/evaluación que hagan veraz la separación
   entre admisión, avance y certificación. Conservar las rutas y tests existentes.
4. Trabajador Rust y transporte integrado en el runner con los controles actuales.
5. Episodio completo contra proveedor simulado: errores, truncamiento, identidad
   de carriers, límites y pérdida de trabajador; comparación con certificados.
6. Adaptador OpenAI y prueba pequeña de conexión sólo con autorización explícita
   de inferencia pagada, modelo, datos y presupuesto. No depende de configurar
   Codex/Claude Code como explorador.
7. Episodios A/B/C medidos con manifiestos congelados y valoraciones independientes.

Todas las compilaciones y tests de Rust deben seguir `python3 scripts/test_safe.py`
con selecciones individuales. Pruebas de transporte y del controlador necesitan
un camino acotado revisado antes de ejecución; no se autoriza una suite completa
ni se cambian los límites para cerrar una prueba.

La primera aceptación técnica exige demostrar con un proveedor simulado que
una solicitud produce exactamente la acción autorizada, que el modelo no puede
alterar el contrato y que ninguna evaluación incompleta se convierte en un
veredicto normativo. La primera aceptación científica sigue siendo la de la
especificación experimental: candidatas admisibles, compatibilidad independiente
y comparación del efecto de la información adicional, no recuperación de `J*`.
