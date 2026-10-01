# Composición y orden operacional: experimento finito

El experimento usa exclusivamente declaraciones y reglas Formal existentes.
No añade operaciones al lenguaje, ni modifica `close`, `close_program`, el
incremental, el residual, defaults, FOUR o el runtime.

Las definiciones son archivos fuente:

| Archivo | Papel |
|---|---|
| [composition_domain.res](../examples/experiments/composition_domain.res) | Firmas, constantes y universos de evidencia |
| [composition_identity.res](../examples/experiments/composition_identity.res) | Programa sin reglas: identidad sobre Known |
| [composition_p.res](../examples/experiments/composition_p.res) | Asociación `sells → sellsProgram → owns` |
| [composition_p_extra.res](../examples/experiments/composition_p_extra.res) | Consecuencia adicional `registered` |
| [composition_q.res](../examples/experiments/composition_q.res) | Asociación `owns → ownsProgram(charlie, molest) → notDo` |
| [composition_q_extra.res](../examples/experiments/composition_q_extra.res) | Consecuencia adicional `protected` |
| [composition_inline.res](../examples/experiments/composition_inline.res) | Las dos reglas base escritas inline, sin asociaciones |

El [harness](../src/experiments/composition_structure_tests.rs) combina el catálogo
con fragmentos fuente antes de la elaboración ordinaria. Un fragmento sólo puede
anotar una firma idéntica con su asociación, o aportar reglas. No puede reemplazar
una asociación ni cambiar tipos. Esto no introduce imports ni sintaxis de módulos
compuestos. Los bloques `premises` de `knowledgeDomain` son observaciones hipotéticas
para el experimento, no seeds reales; sus queries FOUR no se ejecutan aquí.

## Composición y deltas

Se comparan dos familias: los programas básicos P/Q y esos mismos programas con
sus consecuencias adicionales. Para cada familia:

1. Ejecutar P mediante `close_program`.
2. Pasar su Known como entrada a Q mediante `close_program`.
3. Comparar con ambos programas asociados en un único cierre.
4. Comparar con las reglas inline.
5. Comparar actualizaciones incrementales P seguido de Q, el incremental inline
   y la recomputación del programa asociado.

La igualdad incluye identidad de eventos, categoría proposicional, tiempos,
metadata de transición, nombres de reglas, sustituciones y witnesses de provenance.
Los stamps de DerivationTime siguen siendo externos: se exige presencia y unicidad,
no igualdad de etiquetas entre schedules diferentes.

El universo incremental tiene cuatro evidencias firmadas para dos ventas:
`sells(juan, car, pedro) @ tau`, `sells(pedro, car, juan) @ sigma` y sus negativos
explícitos. Sus 16 subconjuntos se usan tanto como bases como adiciones:

\[
2\text{ familias}\times16\text{ bases}\times16\text{ adiciones}=512.
\]

Las 512 actualizaciones coinciden en eventos y justificaciones. Repetir una
adición produce deltas vacíos. También coinciden con las diferencias de eventos
y registros entre los cierres completos anteriores y posteriores.

El incremental de producción sigue admitiendo sólo reglas fijas. El harness
verifica que cada asociación tiene como cuerpo su propia instancia positiva,
con todos los parámetros y tiempo en el mismo orden. En ese fragmento usa las
reglas ya elaboradas como vista Horn y compara cada resultado con la semántica
activa. Esto no prueba incrementalidad general de programas con cuerpos `I`,
guards adicionales, feedback o activación dinámica no autoanclada.

## Orden entre programas

En el harness:

\[
P\preceq_\Omega Q
\iff
\forall K\in\Omega,\quad Known_P(K)\subseteq Known_Q(K).
\]

El input es el mismo a ambos lados y se conserva; por tanto esta comparación
equivale a inclusión de nuevas consecuencias observables. Provenance se verifica
por separado y no decide este orden. No es una definición de \(\le_L\).

Para P, \(\Omega\) contiene 256 estados: todos los subconjuntos de ocho evidencias
firmadas, para las dos ventas y dos instancias de owns indicadas por las fixtures.
Para Q se incluyen además todas las entradas intermedias producidas por P: 384
estados en total. Así Q no se compara vacuamente sólo sobre seeds de sells.

Ambas familias producen la misma tabla:

| \(\preceq\) | Identidad | Básico | Extra |
|---|---|---|---|
| Identidad | Sí | Sí | Sí |
| Básico | No | Sí | Sí |
| Extra | No | No | Sí |

Se verifica además monotonía respecto de añadir evidencia en Known. La
composición \(Q\circ P\) es monótona en ambos argumentos en estos dominios:
4.608 comparaciones del primero y 4.608 del segundo, sin contraejemplos.
Las variantes `extra` incluyen reglas auxiliares del programa; no son cambios
en la categoría o en el cuerpo de una nueva construcción ProgramVerb.

## Adjunción: qué aparece y qué falla

Aquí los carriers son explícitos y diferentes del orden de programas anterior:

- X: 64 estados de evidencia firmada de **tres instancias ground de sells**.
  Se prueban tanto inclusión de conocimiento como el orden puntual de verdad
  del modelo FOUR existente.
- Y: los 256 subconjuntos de ocho consecuencias Derived observables del programa
  asociado completo, ordenados por inclusión.
- F: ejecutar ese programa sobre X y observar sólo sus consecuencias Derived.

Para cada Y, se enumeran todas las X con \(F(X)\subseteq Y\), y se busca una
**mayor** entre ellas en el orden de entrada elegido. No se presupone que exista.
En ambas comparaciones existe G, y se verifica exhaustivamente:

\[
F(X)\subseteq Y\iff X\le G(Y).
\]

Así aparece \(F\dashv G\) en estos dos modelos finitos, con 16.384 pares comprobados
en cada uno. G da la mayor evidencia permitida para mantener los observables
dentro de Y. **No es una explicación abductiva**. Por ejemplo, con orden de
conocimiento, G del conjunto observable vacío incluye evidencia exclusivamente
negativa: esa evidencia no activa ningún programa.

También se busca una **menor** evidencia suficiente, sin presuponerla:

\[
B(Y)\le X\iff Y\subseteq F(X).
\]

Esa adjunción \(B\dashv F\) falla para 240 de los 256 conjuntos objetivo en ambos
órdenes. Un contraejemplo con objetivo singleton es:

```text
goal: owns(pedro, car) @ after(tau)

alternativa 1: sells(juan, car, pedro) @ tau
alternativa 2: sells(charlie, car, pedro) @ tau
```

Cada alternativa por sí sola es suficiente. Son incomparables por inclusión, y
su cota inferior común no deriva el goal. En el orden FOUR de verdad tampoco hay
una menor evidencia suficiente: su meet pierde la evidencia positiva necesaria.
En particular, F no preserva meets de conocimiento: la intersección de los dos
resultados contiene owns, mientras el resultado de la intersección de las
evidencias no lo contiene. Conservar alternativas es indispensable.

Estos resultados describen **evidencia de instancias → observables de su programa**.
No establecen una adjunción entre una declaración Verb y una Rule-lambda como
objetos de M2, ni identifican \(\preceq_\Omega\) con \(\le_L\). El uso del orden de
verdad FOUR en X tampoco reemplaza la semántica Horn por entailment algebraico.
No se implementa ningún adjunto en el motor ni se modifica la búsqueda residual.

## Verificación

Pasaron tres tests individuales bajo los controles existentes. Compilación:
532 MiB RSS; ejecución máxima: 26 MiB RSS. Se mantuvieron 1 GiB total, sin swap,
un job, un hilo y timeouts. No se ejecutó la suite completa.

Reporte: `target/test-diagnostics/20261001T063207593845Z-18451/`.
Los resultados detallados están en `04.output.txt`, `05.output.txt` y `06.output.txt`.

```sh
python3 scripts/test_safe.py --test experiments::composition_structure_tests::composition_staged_associated_inline_and_incremental_deltas_agree_exhaustively
python3 scripts/test_safe.py --test experiments::composition_structure_tests::finite_program_order_and_composition_are_monotone_in_each_argument
python3 scripts/test_safe.py --test experiments::composition_structure_tests::finite_activation_observation_adjunction_is_scoped_and_requirements_have_alternatives
```
