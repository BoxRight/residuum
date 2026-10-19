# Piloto 004: los cuatro expedientes en forma legible (contrato v2)

Extraído de `frozen/dossiers.res` (SHA-256 `ecc58e13…`). No incluye ninguna
predicción de candidatas.

**Notación:**
- **+**: consta el hecho;
- **~**: consta evidencia explícita de que **no** se da;
- **?**: el expediente no dice nada (desconocido, que no es lo mismo que
  negativo).

**Elementos comunes a los cuatro expedientes:**
- Acto `Act(actN, ana, enterWith, obj, bea, private, dateN) @ tN`: ana entra
  llevando la cosa `obj`, con `bea` como beneficiaria de la conducta y bajo
  autoridad privada.
- `tN` es el momento del expediente. `dateN` es la fecha del acto, una
  constante simbólica sin calendario ni orden declarado.
- Posiciones que hay que valorar: `K(hypNorm, ana, enterWith, obj, bea, administrative, hypEnacted) @ tN`,
  con K ∈ {prohibition, permission, obligation}.

| | dossier1 | dossier2 | dossier3 | dossier4 |
|---|---|---|---|---|
| ana: allergic | + | + | ~ | ~ |
| ana: visuallyImpaired | ~ | ~ | + | + |
| bea: allergic | ~ | ~ | ~ | + |
| bea: visuallyImpaired | + | + | ~ | ~ |
| cal: allergic | ? | ? | ? | + |
| cal: visuallyImpaired | ? | ? | ? | ? |
| dan: allergic / visuallyImpaired | ? / ? | ? / ? | ? / ? | ? / ? |
| obj: animal | + | + | + | ~ |
| obj: food | ~ | ~ | + | ~ |
| copresent ana–bea | + | ~ | + | ~ |
| copresent ana–cal | ? | ? | + | + |
| copresent ana–dan | ? | ? | ~ | ~ |
| copresent bea–cal | ? | ? | ~ | + |
| copresent bea–dan | ? | ? | + | ~ |
| copresent cal–dan | ? | ? | ? | ? |

**Glosas congeladas** (`frozen/glosses.tsv`):

| Símbolo | Glosa |
|---|---|
| `allergic` | an allergy |
| `visuallyImpaired` | a visual impairment |
| `animal` | an animal |
| `food` | food or drink |
| `copresent` | the two persons are in the same place at the moment of the dossier |
| `enterWith` | entering while carrying the thing |

**Límites del contenido:**
- La alergia **no tiene objeto declarado**: el vocabulario del piloto no dice
  a qué es alérgica una persona.
- El **lugar no está representado**: no consta que `bea` sea titular del
  lugar en el que se entra.
- En `dossier4`, `obj` no es ni animal ni alimento, y no consta qué es.
- En `dossier1` y `dossier2`, `cal` y `dan` no tienen ningún hecho.
- Los expedientes salen de un diseño estructural (contrato §3). Algunas
  combinaciones pueden parecer artificiales.
