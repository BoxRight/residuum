# Actualización incremental del cierre Horn

La [caracterización de la derivada discreta](discrete-derivative.md) obtiene este
delta como un menor punto fijo sobre eventos nuevos, demuestra sus leyes de
composición y separa el mantenimiento con R fijo de las adiciones de reglas.
El experimento posterior contrasta esa caracterización con la API existente;
no cambia las restricciones de la API descritas aquí.

Para reglas fijas R y una instancia que termina, definimos el delta lógico:

```text
Dclose_R(S, ΔS) = Known(close_R(S ∪ ΔS)) \ Known(close_R(S))
```

El contrato es igualdad de conjuntos de eventos:

```text
Known(close_R(S ∪ ΔS)) = Known(close_R(S)) ∪ Dclose_R(S, ΔS)
```

`IncrementalClosure::new(seeds, rules, clock)` utiliza el `close` existente para
establecer la base cerrada. `update(additions, clock)` mantiene esa base mediante
evaluación semi-naive: cada match tiene al menos un antecedente en la frontera
de eventos nuevos. Los demás antecedentes se unen contra Known, compartiendo
bindings. No vuelve a llamar a `close` para actualizar ni enumera los matches
compuestos exclusivamente de evidencia anterior. La copia privada de reglas
permanece fija y el caller sólo puede leer el cierre.

`ClosureDelta.known` contiene únicamente eventos nuevos, incluidas las seeds
añadidas. `ClosureDelta.derivations` contiene justificaciones nuevas, incluso si
su conclusión ya era conocida. Se conservan los testigos en orden de antecedente,
las sustituciones y todas las justificaciones distintas. Cada justificación nueva
se sella con el reloj externo; las anteriores conservan su sello. Vacío o duplicados
no generan trabajo ni sellos. Los axiomas `I <= C` ya fueron procesados al crear
la base; I no se convierte en evento o conocimiento vacío.

No se exige igualdad de orden de descubrimiento ni de DerivationTime con una
recomputación independiente. Sí se comparan los conjuntos lógicos y las parejas
(evento, DerivationRecord). Las estadísticas cuentan rondas, reglas examinadas,
pivotes compatibles en el delta y matches completos, sin prometer una mejora de
tiempo para todos los programas. El matcher existente se reutiliza sin cambios.

## Experimento desde un archivo Formal

[forward_incremental.res](../examples/experiments/forward_incremental.res) define
dos acuerdos Seeded, una venta Derived y su consecuencia Derived. El bloque
`base` contiene el acuerdo de precio; `delta`, el de objeto. Los dos datos juntos
derivan `saleEstablished` y después `deliverable`.

```sh
python3 scripts/test_safe.py --fixture examples/experiments/forward_incremental.res --incremental
```

El modo externo `--incremental` no amplía la gramática ni el TypedAST. Utiliza
exactamente dos bloques existentes llamados `base` y `delta`, ambos de seeds,
con el mismo goal. Como la gramática aún no tiene `query forward`, se conserva
el campo existente `query residual` sólo para expresar un goal bien tipado:
**este modo no ejecuta esa consulta ni realiza búsqueda residual**. Usa el goal
como observación del cierre forward.

La salida compara la actualización con `close` desde cero sobre la unión de
seeds. Hay un evento en la base, tres en el delta lógico y cuatro en ambos
cierres finales. El goal cambia de false a true, con dos justificaciones nuevas.

[forward_provenance.res](../examples/experiments/forward_provenance.res) verifica
el caso distinto: una nueva seed aporta otra justificación a una conclusión que
ya era conocida. Sólo esa seed entra en el delta lógico. El axioma ground de I
se conserva, sin volver a sellarlo.

## Alcance y verificación

El modo externo admite únicamente Horn sin defaults, cláusulas residuales ni
verbos Effect. Exige dependencias acíclicas antes de calcular cualquier cierre,
como condición suficiente de terminación. La biblioteca también se contrasta
con ciclos finitos del mismo tiempo; no hay garantía de terminación para cadenas
temporales infinitas como after(after(...)). Los controles del runner permanecen:
1 GiB total, sin swap, un job, un hilo y timeouts separados.

Los tests comparan todos los pares de subconjuntos de dos seeds y todas sus
segundas actualizaciones (64 secuencias), además de duplicados, vacío, provenance
alternativa, unidad, bindings incompatibles, polaridad explícita, ciclos finitos
y reloj externo. Los programas son la fuente de los datos; el oráculo independiente
es el `close` anterior. La comprobación finita no constituye una demostración
general del algoritmo.

Verificación de este incremento: ocho tests nuevos y cinco regresiones pasaron
individualmente; ambos archivos se ejecutaron con `--incremental`. La compilación
de tests alcanzó 565 MiB RSS, la del ejemplo 459 MiB, y cada ejecución unos 15 MiB.
No se ejecutó la suite completa. En el primer archivo se observaron tres rondas,
dos pivotes compatibles y dos matches completos; en el segundo, una ronda y un
match nuevo. Ambos reportaron igualdad lógica y de justificaciones.

Esto sólo mantiene adiciones bajo reglas fijas. No incluye eliminación de hechos,
cambios de reglas, derivada residual, defaults, negación por ausencia, ejecución
de efectos o ProgramVerb.
