# Orden temporal con `after`: expresividad, terminación y corrección

Estado: **CERTIFICADO EN DOMINIO FINITO; DEMOSTRADO BAJO LAS HIPÓTESIS INDICADAS**.

| | |
|---|---|
| Programa | `docs/legal-geometry/semantic-domain/temporal-order/temporal_order.res` |
| Tests | `src/bin/residuum/cli/temporal_order_tests.rs` (4 tests, ejecutados uno a uno con `test_safe.py --test-target cli`) |
| Resultados | `temporal-order/runs/certification-tests.txt` |

No se han modificado el motor ni LegalGeneral.

## 1. Representación

**Fechas:**
- Una fecha es un término `PropositionTime` de la forma $\mathrm{after}^n(r)$,
  donde $r$ es una constante raíz.
- En las fixtures, una aplicación de `after` equivale a un año.
- Las fechas aparecen **solo como argumentos**.

**Índice del expediente:** todos los átomos de cada regla comparten un único
índice `@ t`. El test `temporal_order_keeps_dates_as_arguments_and_one_dossier_index`
comprueba que ninguna variable de fecha se usa como índice, y que la
fixture `index_as_date.res`, que viola la separación, se detecta aunque
elabora.

**Verbos sembrados** (descripción del expediente):
- `datePoint(d)`
- `born(p, d)`
- `dossierDate(d)`

**Verbos derivados:**

| Verbo | Definición |
|---|---|
| `before(x, y)` | $y=\mathrm{after}^k(x)$ con $k\ge1$, dentro de los puntos de fecha |
| `notAfter(x, y)` | `before(x, y)`, o $x=y$ |
| `majority(p, m)` | $m=\mathrm{after}^{18}(b)$ si `born(p, b)` |
| `minorAt(p, d)` | `born(p, b)`, `notAfter(b, d)` y `before(d, m)` |
| `adultAt(p, d)` | `notAfter(m, d)` |
| `bornAfterDossier(p, d)` | `before(d, b)` |

Ninguna regla produce posiciones normativas, y el test lo comprueba.

## 2. Expresividad matemática de `after`

**DEMOSTRADO.** `after` es un constructor unario libre: es inyectivo, no tiene
ciclos y no tiene ecuaciones. Los términos $\mathrm{after}^n(r)$ se
corresponden biyectivamente con los pares $(r,n)\in R\times\mathbb N$.

**Qué expresa** con reglas Horn y emparejamiento de patrones:
- **sucesor**: construir `after(x)` en una cabeza;
- **predecesor**: emparejar `after(x)` en un cuerpo;
- **desplazamiento fijo**: $\mathrm{after}^{k}$ con $k$ escrito en la regla;
- **orden estricto** dentro de una raíz, como clausura de «subtérmino propio»
  sobre un conjunto finito de puntos;
- **igualdad**, por identidad de términos;
- aritmética unaria, que el repositorio ya certifica con las construcciones
  Peano (`peano_after.res`, `peano_multiplication.res`).

**Qué no expresa:**
- **orden entre raíces distintas**: $\mathrm{after}^n(r)$ y
  $\mathrm{after}^m(r')$ son incomparables, salvo con un dato explícito
  (`notEarlier`);
- **desigualdad**, ni **negación por fallo**: «no consta `before`» no es
  `notBefore`;
- **densidad** o **granularidad no unaria**: meses o días son representables
  en principio, pero en unario no son prácticos;
- **fechas como índice**: está prohibido por el contrato temporal certificado.

## 3. Terminación

**Proposición. DEMOSTRADO.**

*Enunciado.* Sea $D_0$ el conjunto de términos de fecha que aparecen en las
semillas, y sea $D=\mathrm{Sub}(D_0)\cup\mathrm{Sub}(\{\mathrm{after}^{18}(b)\})$,
con $b$ recorriendo las fechas de `born` y $\mathrm{Sub}$ el cierre por
subtérminos. Entonces todo término de fecha del cierre pertenece a $D$, el
cierre es finito y tiene como mucho $O(|D|^2)$ hechos `before`.

*Prueba.*
- `descend` solo produce subtérminos.
- `majorityDate` no es recursiva y produce $\mathrm{after}^{18}(b)$ una vez
  por cada nacimiento.
- `successor` y `extend` solo construyen `after(y)` si `datePoint(after(y))`
  ya existe.
- Las demás reglas no construyen términos nuevos.
- El universo de Herbrand efectivo es finito, y un cierre monótono sobre un
  universo finito alcanza su punto fijo.

∎

**COMPROBADO** en `before_is_the_strict_successor_order_on_each_spine`: con
puntos $\mathrm{after}^{30}(epoch)$ y $\mathrm{after}^{5}(other)$, el cierre
contiene exactamente $31+6$ puntos, ninguno fuera de $D$.

**Contraejemplo.** La regla ascendente `datePoint(x) <= datePoint(after(x))`
(`counterexamples/ascending.res`) rompe la cota de subtérminos.
- El cierre supera cualquier cota fija; el test usa 5.000 derivaciones.
- Esto es evidencia acotada. La no terminación está demostrada por la
  construcción: produce $\mathrm{after}^n$ para todo $n$.

**Limitación del runner.** `run_forward_experiments` (`--forward`) solo acepta
dependencias acíclicas, y este programa es recursivo. Por eso la
certificación usa `close_program` directamente desde los tests, igual que la
historia temporal.

## 4. Corrección

**`before`. DEMOSTRADO y COMPROBADO.**
- **Corrección:** por inducción sobre las derivaciones, `successor` da $k=1$
  y `extend` suma 1.
- **Completitud** sobre $D$: para $x=\mathrm{after}^i(r)$ y
  $y=\mathrm{after}^j(r)$ en $D$ con $i<j$, todos los puntos intermedios
  están en $D$ (por `descend`), y `extend` recorre la cadena.
- **COMPROBADO:** el conjunto derivado es exactamente
  $\{(i,j):i<j\}$ en cada raíz (465 + 15 pares), sin pares entre raíces.
  `notAfter` añade la diagonal.

**Condiciones derivadas. COMPROBADO**
(`majority_conditions_match_year_arithmetic_at_the_boundaries`). Son 22 casos
con nacimientos $b\in\{0,3,10\}$ y desplazamientos
$d-b\in\{-2,-1,0,1,17,18,19,25\}$, siempre que $d\ge0$. Se comparan con la
aritmética entera:

| Condición | Oráculo |
|---|---|
| `minorAt` | $b\le d<b+18$ |
| `adultAt` | $d\ge b+18$ |
| `bornAfterDossier` | $d<b$ |

En cada caso, con una sola raíz, se deriva exactamente una de las tres.

**Casos límite:**
- **$d=b$** (nace en la fecha del expediente): es `minorAt`.
- **$d=b+17$**: es `minorAt`.
- **$d=b+18$** (cumpleaños): es `adultAt`, porque `notAfter` incluye la
  igualdad.
- **$d<b$**: es solo `bornAfterDossier`. No se deriva `minorAt`, gracias a la
  condición `notAfter(b, d)`.

## 5. Contraejemplos y límites

Comprobados en `counterexamples_mark_the_limits_of_the_construction`:

| Caso | Resultado | Lectura |
|---|---|---|
| El nacimiento y la fecha del expediente tienen raíces distintas | no se deriva ninguna condición | **indeterminado**, no negativo |
| Dos `born` para la misma persona | se derivan `minorAt` y `adultAt` a la vez | el cierre es monótono: no repara datos contradictorios; detectarlo corresponde a la admisibilidad de K |
| Granularidad anual | dos nacimientos del mismo año son el mismo término | no distingue meses; con 17 años y 11 meses cuenta como 17 |
| Regla ascendente | supera la cota | no terminación (§3) |
| Fecha usada como índice | elabora; la detecta la comprobación estructural | la separación es un contrato verificado, no una restricción del elaborador |

## 6. Qué queda definido para el dominio

Con este programa, la **minoría de edad** y la **mayoría de edad** son
**definibles por composición**: `born` (extensión descriptiva primitiva: el
nacimiento es un hecho natural que la ontología no contiene), `dossierDate` y
el orden.

Lo que no queda definido:
- **«Edad avanzada»** necesita un umbral. Sería otra regla con
  $\mathrm{after}^{k}$, y la elección de $k$ es un parámetro descriptivo que
  hay que declarar; no se deduce de la ontología.
- **Conectar `before` con `notEarlier`**, que LegalGeneral usa en las reglas de
  anulación, haría derivado un dato hoy sembrado. **No se ha hecho.**
  Cambiaría las consecuencias de LegalGeneral y requiere una decisión
  aparte.
