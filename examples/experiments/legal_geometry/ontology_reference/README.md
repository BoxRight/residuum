# Fuentes ontológicas reproducibles

Estas dos fuentes son copias literales de las bibliotecas usadas durante la
investigación, originalmente en
`/home/maitreya/12hectareas/vendor/residuum/modules/`.
Permiten ejecutar las fixtures sin disponer de ese repositorio hermano.

| Archivo | SHA-256 |
|---|---|
| `legal_general.res` | `639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc` |
| `legal_patrimonial.res` | `f7daf4d5eb937a7fde8534dba5618acc50d5aaa5b05ff86c9656892eb3a03fbf` |

Los módulos conservan `LegalGeneral` y `LegalPatrimonial`, sus declaraciones,
firmas, cuerpos de reglas, tiempos y efectos. No sustituyen la referencia
`examples/legal/modules/legal_monotonic.res`.

Las rutas de origen ahora apuntan a esta carpeta. Los certificados históricos
pueden conservar la ruta externa de la ejecución que los produjo; los hashes
de contenido permiten identificar la misma fuente. La revisión de integración
verifica los módulos de origen y la correspondencia de las declaraciones
elaboradas, sin afirmar igualdad literal entre las dos rutas de archivo.
