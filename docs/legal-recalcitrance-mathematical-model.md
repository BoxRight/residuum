# Formalización del modelo de recalcitrancia y aprendizaje normativo

Informe de consolidación en `experiment-legal-order-geometry`. El punto de
partida es el contrato del proyecto: justificación subyacente desconocida →
generalización normativa → regla; las experiencias recalcitrantes permiten
revisar las generalizaciones sin presuponer conocida la justificación.

Se inspeccionaron código, fixtures, contratos y certificados existentes. No se
realizó búsqueda bibliográfica ni se adoptó otro modelo de aprendizaje. Este
documento no modifica la ontología, el motor ni los experimentos. Tampoco
implementa las operaciones matemáticas que describe.

La conclusión principal es precisa: **el harness construye y filtra
generalizaciones relacionales dentro de una gramática finita; Residuum verifica
las consecuencias de sus reglas mediante cierres independientes.** Eso permite
aproximaciones parciales contrastables. Todavía no demuestra aproximación a una
justificación verdadera identificada, ni convierte el impedimento de Riggs en
una exclusión efectiva.

## 1. Objetos y alcance de las afirmaciones

Se utilizan cuatro estados de verificación:

| Marca | Alcance |
|---|---|
| **DEMOSTRADO** | Proposición matemática con las hipótesis indicadas; no una verificación formal del código completo. |
| **FINITO** | Resultado de los universos y certificados identificados. |
| **IMPLEMENTADO** | Operación presente en el código, sin demostración general de equivalencia con el modelo matemático. |
| **ABIERTO** | Hipótesis conceptual o capacidad pendiente; no se presenta como operación ejecutable. |

### 1.1 Teoría, expediente y resultado

Una teoría de referencia \(T\) comprende declaraciones tipadas, reglas y, cuando
existan, programas y sus asociaciones. El módulo resuelto conserva los orígenes
de las declaraciones. En los experimentos de geometría jurídica y Riggs, las
teorías consideradas no requieren selección de defaults.

Un evento ground tiene identidad lógica:

\[
e=(\text{verbo},\text{argumentos},\text{tiempo proposicional},\text{polaridad}).
\]

Los argumentos conservan sujetos, objetos, actos, instrumentos, autoridades y
vigencias según sus firmas. `@ t` y el argumento `effective` no se identifican.
La transición de estado está fuera de esta identidad lógica. Los tipos se
interpretan respecto de las declaraciones elaboradas de \(T\).

Para un universo finito explícito \(E\), cerrado bajo las instancias de reglas
estudiadas, escribimos:

\[
K=\mathcal P(E),\qquad S\leq_K S'\iff S\subseteq S'.
\]

La evidencia externamente admisible pertenece a \(\mathcal A\subseteq K\).
En los catálogos Seeded exhaustivos, \(\mathcal A=\mathcal P(\Sigma)\), donde
\(\Sigma\) contiene eventos ground de verbos Seeded. Los estados geométricos de
\(K\) pueden incluir puntos Derived como premisas de teoría. Eso no los convierte
en entradas Seeded admitidas por Formal.

El cierre de eventos es \(C_T(S)\). La ejecución devuelve además justificaciones:

\[
\operatorname{Run}_T(S)=(C_T(S),\mathcal P_T(S)).
\]

Aquí \(\mathcal P_T\) significa provenance, no conjunto potencia. Conserva
reglas, antecedentes ordenados, sustituciones canónicas y orígenes; varias
justificaciones distintas de un evento se conservan. No es una función del
conjunto cerrado solamente: aportar una conclusión como premisa y derivarla
desde otros hechos pueden dar igual cierre y distintas pruebas.

Un resultado normativo exige un objetivo y un canal explícitos:

\[
N_T(S)=o_Q(C_T(S),\mathcal P_T(S)).
\]

Puede ser un conjunto de eventos jurídicos seleccionados; en una pregunta de
derivabilidad concreta puede ser la pertenencia de un evento ground. El booleano
de esa pregunta es una proyección, no una reducción de toda la ontología.
Un resultado de ejecución sobre `State` requiere otro canal y una ejecución
expresa de efectos; no se obtiene automáticamente del cierre.

### 1.2 Experiencia recalcitrante

La experiencia tiene la forma:

\[
\varepsilon=(S,Q,N_T(S),y^*(S),m),
\]

donde \(m\) registra fuente, fundamento, alcance, carácter histórico o hipotético
y estado de la valoración. \(y^*\) es la valoración alternativa fundamentada
respecto del mismo objetivo \(Q\). La discrepancia entre resultados comparables
constituye la recalcitrancia.

Una valoración puede exigir varias condiciones sobre conjuntos de consecuencias.
Entonces se declara un criterio \(\kappa_\varepsilon\) de satisfacción de esa
valoración, y la discrepancia es:

\[
\kappa_\varepsilon(N_T(S))=0.
\]

Esta forma no sustituye la valoración externa: precisa cómo se compara con el
resultado ejecutable. Si la valoración es un resultado exacto en el mismo
dominio, \(\kappa_\varepsilon(z)=[z=y^*(S)]\). Si sólo se ha formalizado una
condición necesaria de un fallo, satisfacerla no certifica el fallo completo.
Esto último es exactamente el límite del experimento actual de Riggs.

Para oposición evidencial, una observación de \(P\) puede registrar:

\[
(\mathbf1_{P\in C_T(S)},\mathbf1_{\sim P\in C_T(S)})\in\{0,1\}^2.
\]

Los cuatro pares distinguen falta de ambos soportes, soporte positivo, negativo
y simultáneo. No se introduce aquí una nueva semántica FOUR. El par \((1,1)\)
no determina qué resultado jurídico prevalece. Las reglas del dominio pueden
producir `inconsistent` o `disputedPerformance` sin eliminar evidencia.

La etiqueta externa no se inserta en \(S\), no recibe una identidad de evento
Seeded y no se obtiene de una contradicción interna. Su fundamentación es un
dato jurídico que debe documentarse. El harness puede comprobar su registro y
su utilización; no verifica por sí mismo la suficiencia jurídica del fundamento.

### 1.3 Sobreinclusión y subinclusión

Fijada una consecuencia \(q\):

\[
\mathcal R_{T,q}=\{S\in\mathcal A:q\in C_T(S)\}.
\]

Sobre el conjunto \(D\) de expedientes valorados, sea
\(Y_q^+=\{S\in D:\text{la valoración exige }q\}\). Cuando las valoraciones
determinan inclusión o exclusión respecto de \(q\), los desacuerdos son:

\[
\operatorname{Sobre}_q=(\mathcal R_{T,q}\cap D)\setminus Y_q^+,
\qquad
\operatorname{Sub}_q=Y_q^+\setminus\mathcal R_{T,q}.
\]

Las valoraciones pendientes no se incluyen como etiquetas negativas. Si hay
resultados múltiples, estas definiciones se aplican a cada objetivo declarado.
Riggs es sobreinclusión respecto de la atribución literal y subinclusión respecto
de un impedimento requerido que la referencia no produce. Cambiar de objetivo
cambia la descripción, no los hechos del expediente.

## 2. Justificación desconocida, generalización y regla

Distinguimos:

| Objeto | Papel en nuestro modelo |
|---|---|
| \(J^*\) | Justificación subyacente desconocida que motiva investigar las distinciones normativas. |
| \(G_\theta\) | Generalización candidata: condiciones relacionales tipadas para un tratamiento normativo. |
| \(r_\theta\) | Regla formal que materializa esas condiciones y su consecuencia. |
| \(T_\theta\) | Teoría hipotética obtenida mediante una operación de revisión declarada. |
| \(N_{T_\theta}(S)\) | Resultado producido por esa teoría en el expediente. |

Si se supone una función \(J^*:\mathcal A\to Y_Q\), sus valores no están
disponibles como un oráculo del experimento. No se genera una función verdadera
oculta para fabricar etiquetas y comprobar su recuperación. Tampoco se exige
que sea binaria, monotónica, computable o única. La formulación y las pruebas
actuales no dependen de resolver previamente esas cuestiones.

Para relacionar una valoración con \(J^*\) se necesita el supuesto de que esa
valoración ofrece información fundada acerca del tratamiento exigido por la
justificación. Este supuesto se declara; no se deduce del cierre ni de la
compatibilidad de una candidata. Sin él, el filtrado sigue siendo un procedimiento
correcto de ajuste a observaciones, pero no demuestra conocimiento de \(J^*\).

### 2.1 Generalización relacional

Para una cabeza ground \(h\), el patrón de Riggs permite definir:

\[
G_\theta(S,h)=1
\iff
\exists\sigma:\quad
\operatorname{head}(r_\theta)\sigma=h,
\quad \operatorname{body}(r_\theta)\sigma\subseteq C_T(S).
\]

La sustitución \(\sigma\) debe respetar las firmas y los bindings compartidos.
El mismo beneficiario, acto, instrumento, víctima y tiempo deben coincidir donde
la regla los comparte. No basta con marcar que «hay un homicidio» en algún lugar
del expediente. Variables locales aportan testigos concretos al match.

La generalización selecciona una región relacional de casos. Su implementación
es una cláusula parametrizada con esos antecedentes y una consecuencia. En Riggs
la cabeza nueva no alimenta ninguna regla: basta evaluar el cuerpo sobre
\(C_T(S)\). Una revisión con retroalimentación requiere el cierre completo de
\(T_\theta\); la fórmula anterior no describe automáticamente toda revisión.

El generador pertenece al harness. El parser/elaborador comprueba las reglas
generadas y el motor ejecuta sus instancias. β-reduction especializa parámetros;
el matching obtiene sustituciones a partir de evidencia. Ninguna de esas dos
operaciones decide por sí misma qué generalización debe aprenderse.

### 2.2 Sentido preciso de «aproximación»

El sentido actualmente comprobable es **compatibilidad parcial revisable con
valoraciones documentadas y consecuencias verificadas**:

\[
\forall i\leq n:\quad
\kappa_i(N_{T_\theta}(S_i))=1.
\]

Esto no mide una distancia numérica a \(J^*\). Tampoco afirma coincidencia fuera
de los expedientes evaluados. Si varias candidatas cumplen esas condiciones,
se conservan todas: una sola experiencia no identifica una regla única.

**DEMOSTRADO.** Si dos posibles funciones subyacentes coinciden en los expedientes
observados y difieren fuera de ellos, ninguna operación que utilice únicamente
esas observaciones puede distinguirlas. La entrada del procedimiento es idéntica
en ambos casos. Por tanto, compatibilidad finita no implica identificación global.

Con una gramática finita fija y valoraciones determinadas, el espacio de versiones
se estabiliza tras un número finito de eliminaciones estrictas. Esa estabilización
no demuestra convergencia a \(J^*\). Identificación dentro de la gramática exige,
además, que exista una candidata adecuada y que los casos disponibles separen las
candidatas de comportamiento distinto. Incluso entonces se identifica una regla
o clase observable dentro de esa gramática, no la razón que la justifica.

Los documentos anteriores denominan \(J_\theta\) a los clasificadores de soportes.
Para este informe, esos objetos se interpretan como **hipótesis clasificatorias
de generalización**, distintas de \(J^*\). No se cambia el código ni se renombra
el catálogo; se precisa el papel de su notación.

## 3. Cierre y geometría de la ontología

### 3.1 Propiedades de clausura

Para un conjunto fijo de implicaciones ground positivas sobre literales explícitos:

\[
C_T(S)=\mu F.\left(S\cup F\cup
\{h:(B\Rightarrow h)\in T,\ B\subseteq F\}\right).
\]

`~Act` es un literal explícito y no una condición por ausencia. Sobre un universo
finito cerrado, la saturación termina y satisface:

\[
S\subseteq C_T(S),\quad
S\subseteq S'\Rightarrow C_T(S)\subseteq C_T(S'),\quad
C_T(C_T(S))=C_T(S).
\]

**DEMOSTRADO.** Se conservan los hechos de entrada; toda prueba habilitada desde
\(S\) sigue habilitada desde \(S'\); saturar un conjunto ya cerrado no añade
eventos. Las verificaciones del motor en las ventanas registradas contrastan
estas propiedades. No constituyen una demostración general del evaluador Rust.

`close_program` incorpora programas asociados mediante activaciones positivas
Seeded/Derived y reglas especializadas. Una activación nueva se inicializa contra
todo Known disponible. La incrementalidad distingue nuevos eventos, derivaciones
y activaciones. Riggs no necesita esas activaciones para su catálogo de reglas.
En el lenguaje general, `after` puede generar cadenas temporales no acotadas:
la finitez y la terminación de estos experimentos no se trasladan automáticamente
a cualquier programa.

La idempotencia se refiere al conjunto de eventos, no a igualdad literal de
objetos `Closure` tras resembrar todos sus puntos. Las justificaciones y el reloj
de derivación deben compararse por separado. `derivedAt` es un sello operacional;
las auditorías entre ejecuciones lo excluyen conservando la identidad de prueba.

### 3.2 Fronteras mínimas y extremos

En \(\mathcal P(\Sigma)\), la región de un objetivo ground \(q\) es ascendente.
Su frontera es:

\[
\mathcal B_{T,q}=\min_{\subseteq}\{S\subseteq\Sigma:q\in C_T(S)\}.
\]

**DEMOSTRADO, para este dominio finito.**

\[
\mathcal R_{T,q}=\bigcup_{B\in\mathcal B_{T,q}}\uparrow B.
\]

Cada caso positivo contiene un soporte mínimo, obtenido por eliminación finita;
la monotonía da la suficiencia de todos sus supersets. La frontera es una
anticadena. Los bitsets de los verificadores codifican eventos elaborados
completos; no sustituyen sus relaciones por indicadores inventados.

La frontera certificada de `compliance` en el catálogo inicial es
\(\{\{obligationFiled,Act\},\{right,Act\}\}\), con todos sus bindings.
Las fronteras permiten comparar qué evidencia exige cada generalización. No
producen las valoraciones externas ni descubren por sí mismas una justificación.

Para un estado cerrado completo \(F\):

\[
ex_T(F)=\{x\in F:x\notin C_T(F\setminus\{x\})\}.
\]

Bajo finitez, vacío cerrado y dependencias ground acíclicas, se tiene
antiintercambio: para \(x\neq y\), ambos fuera de \(C_T(S)\),

\[
x\in C_T(S\cup\{y\})\Rightarrow
y\notin C_T(S\cup\{x\}).
\]

**DEMOSTRADO bajo esas hipótesis.** Una derivación nueva de \(x\) necesita usar
\(y\), induciendo un camino \(y\to x\); la situación inversa exigiría un ciclo.
Todo generador de \(F\) contiene sus extremos, pues sin un extremo no puede
recuperarlo. Una inducción topológica reconstruye cada punto no extremo desde
los de rango inferior. Por tanto \(ex_T(F)\) genera \(F\) y es su única base
mínima. En consecuencia:

\[
C_T(S)=F\iff ex_T(F)\subseteq S\subseteq F.
\]

Es una propiedad de esa clausura, no una unicidad de explicación del objetivo
\(q\). Un estado cerrado puede tener base única y un objetivo varias fronteras
mínimas. Si no hay productores Seeded, para entradas Seeded la base extrema de
su cierre es precisamente la entrada: esos hechos no se pueden reconstruir.

**FINITO.** Los certificados de la ontología comparan General y Patrimonial
en un universo común de 73 eventos, y verifican bases en ventanas controladas.
Riggs examina 13 puntos, incluidos un impedimento pasivo y puntos Derived, no
todo el lenguaje ni todos los tiempos posibles.

### 3.3 Conservación y pérdida geométrica

La separación de `legal_monotonic.res` en LegalGeneral —27 reglas— y
LegalPatrimonial —`complianceAsset`, `deliver`, `molest`— conserva los cuerpos
elaborados de la referencia de 28 reglas. La correspondencia de provenance
requiere trasladar explícitamente los módulos de origen; no se deduce de la
igualdad de declaraciones.

**FINITO.** En Riggs las candidatas sólo añaden una cabeza terminal. Sus grafos
ground siguen acíclicos en el universo certificado; se conserva antiintercambio.
Esto no implica que cualquier generalización aprendida preserve esa geometría.

Contraejemplo existente: una revisión añade `Act & disputedPerformance → compliance`.
La regla original `compliance & violation → disputedPerformance` da, sobre
\(F=C(\{Act,violation\})\), cerrado y sin las dos conclusiones:

\[
disputedPerformance\in C'(F\cup\{compliance\}),\qquad
compliance\in C'(F\cup\{disputedPerformance\}).
\]

Se viola antiintercambio. Los sujetos, objetos y tiempos son los del testigo
tipado certificado, no variables proposicionales libres. Cambiar el orden de
antecedentes no elimina el contraejemplo si conserva el mismo operador.
Antiintercambio informa sobre la transformación inferencial; no decide su validez
jurídica ni se usa como filtro automático del aprendizaje.

## 4. Aprendizaje sucesivo y granularidad

### 4.1 Espacio de versiones

Fijamos una gramática tipada \(\mathcal L\), una operación hipotética
\(\operatorname{op}\) y un conjunto de candidatas \(\mathcal H_0\).
Para cada \(\theta\):

\[
r_\theta=\operatorname{formalizar}(G_\theta),\qquad
T_\theta=\operatorname{op}(T,r_\theta).
\]

Cada nueva experiencia determinada actualiza:

\[
\mathcal H_{n+1}=\{\theta\in\mathcal H_n:
\kappa_{n+1}(N_{T_\theta}(S_{n+1}))=1\}.
\]

La valoración pendiente conserva \(\mathcal H_n\). Si queda vacío se informa
incompatibilidad entre observaciones, dominio y gramática; no se selecciona una
candidata arbitraria ni se genera automáticamente una regla nueva.

**IMPLEMENTADO en el harness.** Los experimentos de funciones de soportes generan
168 funciones monotónicas sobre cuatro eventos Seeded completos; seis respuestas
hipotéticas conservan 11 y la séptima positiva conserva 5. Son controles del
procedimiento, con valoraciones estipuladas. No constituyen deliberaciones
jurídicas fundamentadas que identifiquen una justificación desconocida.

En Riggs, la premisa de un argumento puede ser `Compatible`, `Incompatible` o
`Indeterminate`. Ese estado, la valoración externa y la permanencia de la regla
son tres registros distintos. No obtener prueba del impedimento significa falta
de prueba en ese cierre, no evidencia negativa del impedimento ni refutación por
ausencia de una premisa. El filtro compara la cobertura exigida expresamente
por la valoración; no transforma ausencia en `EvidentialNot`.

### 4.2 Particiones de estados y particiones de hipótesis

Una generalización puede agrupar estados por la misma respuesta:

\[
S\equiv_{G_\theta}S'\iff G_\theta(S)=G_\theta(S').
\]

Para patrones parametrizados, hay que fijar qué cabezas o perfil de respuestas
se observan y cómo se comparan identidades de sujetos. No se borran identidades
mediante una equivalencia tácita. Esta partición expresa la granularidad de ese
canal; no es necesariamente la partición por clausura:

\[
S\equiv_T S'\iff C_T(S)=C_T(S').
\]

Un protocolo sobre expedientes \(D\) induce otra partición, ahora de hipótesis:

\[
\theta\equiv_D\eta
\iff\forall S\in D:\ o_Q(\operatorname{Run}_{T_\theta}(S))
=o_Q(\operatorname{Run}_{T_\eta}(S)).
\]

Las firmas pueden observar consecuencias o justificaciones; no son el mismo
canal. La etiqueta recibida selecciona las hipótesis compatibles, mientras la
predicción de cada hipótesis determina el bloque en que se encuentra.

En el orden «más fina ≤ más gruesa», las equivalencias de particiones satisfacen:

\[
E_{\pi\wedge\rho}=E_\pi\cap E_\rho,
\qquad
E_{\pi\vee\rho}=\operatorname{Eq}(E_\pi\cup E_\rho).
\]

**DEMOSTRADO.** Combinar firmas de observación equivale a intersectar sus kernels:
\(\ker(o_1,o_2)=\ker(o_1)\wedge\ker(o_2)\). La intersección conserva
exactamente las igualdades compartidas. La menor equivalencia que contiene ambas
relaciones da su join; no añade capacidad discriminante.

**IMPLEMENTADO en el harness.** `kernel`, `meet`, `finer` y el verificador de
particiones calculan estas operaciones. El experimento de nueve hipótesis
enumera 21.147 particiones; no implementa una retícula nueva en el lenguaje.

### 4.3 Cuándo hay refinamiento y cuándo no

Si una experiencia separa dos expedientes antes indistinguibles y se quiere
conservar la distinción anterior, añadir un canal \(d\) produce:

\[
\pi_{\mathrm{nueva}}=\pi_{\mathrm{anterior}}\wedge\ker d.
\]

Esta es una formalización defendible de refinar la granularidad. Si una valoración
exige respuestas distintas dentro de un mismo bloque, ningún clasificador que
factorice por esa partición puede satisfacerlas: sería constante en el bloque.
Hace falta distinguirlo o reconsiderar las valoraciones o su alcance.

Pero sustituir una generalización por otra no conserva necesariamente la firma
anterior. Contraejemplo: en tres expedientes tipados con obligación sola,
derecho solo y ambos, una condición «hay obligación» agrupa
\(\{O,OR\}\mid\{R\}\); «hay derecho» agrupa
\(\{O\}\mid\{R,OR\}\). Son particiones incomparables. Es un ejemplo
matemático de condiciones, no una nueva regla del experimento ni una valoración
jurídica de esos expedientes.

Filtrar hipótesis tampoco añade por sí solo un canal sobre expedientes. Puede
eliminar precisamente la hipótesis que distinguía dos casos. El refinamiento de
particiones de hipótesis por nuevas preguntas y la revisión de una partición de
expedientes son operaciones diferentes.

### 4.4 Adición de reglas y clases de clausura

**DEMOSTRADO.** Sean \(C\) y \(C'\) operadores de clausura sobre el mismo
\(\mathcal P(E)\), con \(C(S)\subseteq C'(S)\) para todo \(S\), como ocurre
al añadir implicaciones Horn. Entonces:

\[
C'(C(S))=C'(S).
\]

Prueba: \(S\subseteq C(S)\subseteq C'(S)\); aplicar monotonía e idempotencia
de \(C'\) da igualdad entre los extremos. Por tanto:

\[
C(S)=C(U)\Rightarrow C'(S)=C'(U).
\]

La partición por clausura se hace más gruesa o permanece igual; una adición pura
no refina esa partición sobre el mismo carrier. Si cambia el universo primero
hay que declarar un carrier común. La comparación de Riggs lo hace mediante la
firma pasiva del impedimento en la referencia.

**FINITO, contraejemplo certificado a un refinamiento universal.**
\(F=C_T(S_{Riggs})\) y \(F\cup\{Impediment\}\) tienen cierres distintos en
la referencia pasiva y el mismo cierre bajo cada una de las 16 supervivientes.
La nueva regla distingue expedientes según sus condiciones, pero fusiona estas
dos clases de clausura. No hay contradicción: son particiones de objetos y canales
distintos.

### 4.5 Partición, cociente y congruencia

Toda equivalencia permite un conjunto de clases. Para que ese cociente preserve
la retícula de estados cerrados hace falta una congruencia:

\[
F\sim F',\ H\sim H'
\Rightarrow
F\wedge H\sim F'\wedge H',\quad
F\vee H\sim F'\vee H'.
\]

En \(\operatorname{Fix}(C_T)\),
\(F\wedge H=F\cap H\) y \(F\vee H=C_T(F\cup H)\).
La condición garantiza que las operaciones no dependen del representante.
Sin ella sólo se tiene una partición observable, no ese cociente de retículas.

Contraejemplo usando la frontera real de `compliance`: sean
\(F_O=C_T(\{obligationFiled\})\), \(F_A=C_T(\{Act\})\) y
\(F_0=C_T(\varnothing)=\varnothing\), en el caso tipado coincidente de la
ventana certificada. El canal «deriva compliance» identifica \(F_O\) con \(F_0\),
pues ninguno cumple. Pero:

\[
compliance\in F_O\vee F_A,
\qquad compliance\notin F_0\vee F_A.
\]

Su equivalencia no respeta join y no es congruencia. El canal observacional
puede ser monotónico y seguir sin ser un homomorfismo de retículas.

Tampoco toda equivalencia por clausura respeta meet en \(\mathcal P(E)\).
Para la implicación abstracta \(a\land b\Rightarrow c\), las entradas
\(\{a,b\}\) y \(\{a,b,c\}\) tienen el mismo cierre. Al intersectarlas con
\(\{c\}\) se obtiene respectivamente \(\varnothing\) y \(\{c\}\), de
cierres distintos. Es un contraejemplo matemático mínimo a una identificación
automática, sin introducir reglas nuevas en la ontología.

Los cocientes no son necesarios para el filtrado actual: bastan kernels,
particiones y conjuntos de versiones. El código inspeccionado no construye un
cociente jurídico con operaciones inducidas de meet/join. La retícula de
particiones implementada en el harness no equivale a construir ese cociente.

## 5. Reconstrucción operacional de Riggs

### 5.1 Teoría y experiencia histórica

La [referencia sucesoria](../examples/experiments/legal_geometry/riggs_succession_reference.res)
importa LegalGeneral y añade cuatro reglas, sin alterar las 27 originales:

1. Disposición operativa + muerte del mismo testador + supervivencia del
   beneficiario → atribución testamentaria.
2. Atribución → activo patrimonial, que participa en las reglas generales de
   titularidad y capacidad.
3. Atribución + muerte + homicidio ilícito del beneficiario contra el testador
   + relación causal con esa muerte → adquisición ligada al ilícito.
4. Enlace anterior + finalidad adquisitiva sobre el mismo acto, instrumento y
   beneficiario → adquisición con finalidad.

`operativeDisposition` incorpora como supuesto positivo la operatividad admitida
del instrumento. `intentionalKilling` conserva el contrato de homicidio intencional
ilícito del experimento. No se infiere operatividad por ausencia de revocación.

El [expediente histórico](../examples/experiments/legal_geometry/riggs_cases.res)
contiene seis seeds tipados: disposición, muerte, supervivencia, homicidio,
causalidad y finalidad. Identifica a Francis y Elmer, el instrumento, el interés
residual, el acto y la muerte; conserva autoridad, `will1880`, `death1882` y
`@ dossierTime`. Su cierre tiene 12 eventos y seis justificaciones.

La [observación histórica](../examples/experiments/legal_geometry/riggs_rule_observations.tsv)
registra la decisión de impedir esa adquisición con su fuente y alcance. Su
traducción operacional exige derivar `hypotheticalSuccessionImpediment` para el
mismo beneficiario, instrumento e interés. La valoración documentada no se
genera desde una candidata ni se introduce en Known.

Hay dos niveles: el fallo requiere exclusión; el harness comprueba una condición
experimental necesaria de ajuste, la derivación del impedimento propuesto.
Mantenerlos separados evita llamar exclusión a la mera adición de un evento.

### 5.2 Generación de las 64 generalizaciones

El generador fija la atribución \(E\) en todo antecedente y elige cualquier
subconjunto de seis patrones:

| Patrón | Relación tipada |
|---|---|
| \(K\) | Homicidio del beneficiario contra ese testador, con acto, autoridad y vigencia. |
| \(L\) | Adquisición vinculada al ilícito, con instrumento, acto, muerte, beneficiario y objeto. |
| \(F\) | Finalidad adquisitiva sobre el mismo acto, instrumento y beneficiario. |
| \(B\) | Activo atribuido por la referencia al beneficiario, objeto e instrumento. |
| \(\sim K\) | Oposición evidencial explícita al patrón de homicidio. |
| \(\sim F\) | Oposición evidencial explícita al patrón de finalidad. |

Para cada \(M\subseteq\{K,L,F,B,\sim K,\sim F\}\):

\[
r_M:\quad E\ \&\ \bigwedge_{a\in M}a\ \le\ Impediment\ @t.
\]

Son \(2^6=64\) cláusulas parametrizadas. `generated_source` obtiene los
parámetros usados, genera Formal y el test compara el texto con el
[catálogo](../examples/experiments/legal_geometry/riggs_candidate_rules.res).
Los tipos y bindings se elaboran con el lenguaje existente. No se ejecutan las
64 reglas conjuntamente: cada teoría privada incorpora exactamente una.

La gramática, la cabeza común, la obligatoriedad de \(E\) y la selección de
patrones son supuestos del harness, no distinciones aprendidas desde cero.
\(B\) describe atribución bajo \(T\), no cobro efectivo. La firma original de
\(L\) no contiene testador: el enlace pasa por el instrumento y su productor;
los certificados Seeded se limitan al dominio de un testador por instrumento.
No se afirma que la ontología imponga universalmente esa dependencia funcional.

### 5.3 Teorías hipotéticas y resultado del filtrado

La operación utilizada es exclusivamente:

\[
T_M=T\oplus r_M
=T\cup\{\text{declaración Derived de Impediment},r_M\}.
\]

`Study::theory` conserva declaraciones y orígenes de \(T\). `Study::run` calcula
el cierre hipotético; `Study::certify` proyecta fuera la cabeza nueva y exige
igualdad de eventos completos y justificaciones con un cierre original
independiente, ignorando sólo `derivedAt`.

**FINITO.** Las 384 comparaciones —64 candidatas × seis expedientes— conservan
la referencia. La observación histórica conserva las candidatas 00–15: todas
las combinaciones de \(K,L,F,B\). Las otras 48 requieren \(\sim K\) o
\(\sim F\), que ese expediente no soporta. No se deriva su falsedad por ausencia;
simplemente no producen la consecuencia exigida en ese cierre.

Una experiencia positiva no prueba necesidad de todos los antecedentes presentes.
Por eso sobreviven tanto \(E\) solo como condiciones más estrechas. También
sobrevive el control amplio que impediría cualquier atribución. El filtrado
histórico no puede excluirlo sin nuevas observaciones o restricciones declaradas.

En los seis expedientes, las 16 supervivientes tienen cuatro perfiles:

| Familia | Candidatas | Sin homicidio / tercero | Sin finalidad | Causalidad pendiente |
|---|---|---|---|---|
| \(E\), \(B\) opcional | 0, 8 | Deriva | Deriva | Deriva |
| \(E\land K\), \(B\) opcional | 1, 9 | Sin prueba | Deriva | Deriva |
| \(E\land L\), \(K,B\) opcionales | 2, 3, 10, 11 | Sin prueba | Deriva | Sin prueba |
| Exige \(F\), demás opcionales | 4–7, 12–15 | Sin prueba | Sin prueba | Sin prueba |

Todas derivan en el histórico y ninguna en supervivencia no establecida.
Sobre los 64 subconjuntos de los seis seeds, aparecen seis fronteras distintas:
ruta literal; ruta + homicidio; ruta + homicidio + causalidad; ruta + finalidad;
ruta + homicidio + finalidad; ruta + homicidio + causalidad + finalidad.
\(B\) es redundante porque \(E\) ya lo deriva; \(K\) junto a \(L\) lo es en
esas entradas Seeded porque producir \(L\) requiere el homicidio. La redundancia
no se extiende sin más a entradas que aportan directamente \(L\) como Derived.

### 5.4 Pregunta discriminante y actualización

La pregunta sobre homicidio causalmente ligado a la adquisición, con la finalidad
adquisitiva explícitamente negada, divide las supervivientes en 8/8. Es minimax
única entre los cinco contrafactuales del catálogo:

| Caso | Derivan / sin prueba | Mayor rama |
|---|---:|---:|
| Sin homicidio del beneficiario | 2 / 14 | 14 |
| Homicidio sin finalidad adquisitiva | 8 / 8 | 8 |
| Homicidio de tercero | 2 / 14 | 14 |
| Causalidad no establecida | 4 / 12 | 12 |
| Supervivencia no establecida | 0 / 16 | 16 |

La finalidad negada es evidencia \(\sim F\), no una inferencia desde falta de
\(F\). El [guion de respuestas](../examples/experiments/legal_geometry/riggs_rule_followup_valuations.tsv)
prueba tres ramas independientes: exigir impedimento conserva ocho sin requisito
\(F\); no exigirlo conserva las otras ocho; suspender conserva las dieciséis.
Son valoraciones hipotéticas declaradas, no sentencias de contrafactuales.

Se aprende qué condiciones conservan compatibilidad con la respuesta. Los
fundamentos del guion permiten comprobar la mecánica conversacional; no sustituyen
una valoración jurídica independiente efectivamente realizada. La partición por
predicciones se combina por meet, y la respuesta selecciona un bloque.

### 5.5 Geometría, provenance y límite de exclusión

El universo geométrico común contiene los 12 eventos históricos más el impedimento:
13 puntos, 8.192 estados. La referencia añade sólo la firma pasiva de este último
para permitir comparar el mismo carrier.

**FINITO.** Referencia más 64 teorías producen 532.480 estados. Los certificados
comprueban extensividad, monotonicidad, idempotencia, vacío cerrado, antiintercambio,
generación por extremos y aciclicidad. Hay 2.094 contrastes del modelo de saturación
con el evaluador real. La referencia tiene 1.984 estados cerrados; las supervivientes,
entre 1.776 y 1.968. Las 48 rechazadas son inertes en este universo positivo, que no
contiene sus negaciones: esa comprobación no cubre todos los estados con oposición.

Las pruebas nuevas identifican regla candidata y módulo `RiggsCandidateRules`;
las originales conservan `LegalGeneral` y `RiggsSuccessionReference`. Se verifica
renombramiento tipado, dos expedientes concurrentes y dos actos con la misma
cabeza y justificaciones distintas. Esto certifica transporte de bindings y
generalización estructural, no decisiones jurídicas independientes en casos nuevos.

El impedimento no es \(\sim testamentaryEntitlement\). Es una conclusión positiva
terminal. Como ninguna regla original la consume, no altera atribución, activo,
titularidad o capacidad. Una adición Horn conserva esas conclusiones.

**DEMOSTRADO, límite operacional.** Si \(q\in C_T(S)\) y sólo se añaden reglas,
\(q\in C_{T\cup\{r\}}(S)\). Por tanto, si el contrato exigiera además que la
atribución literal dejase de derivarse, ninguna de las 64 adiciones lo satisfaría.
Las 16 satisfacen el criterio de impedimento propuesto, no ese contrato más fuerte.
Resolverlo requiere una operación explícita distinta de adición; no se implementa
ni se elige una política en este informe.

## 6. Correspondencia concreta y operaciones disponibles

Los nombres de archivo de la tabla son relativos a la raíz del repositorio.
Las marcas se refieren al alcance indicado, no a una certificación universal.

| Objeto matemático | Significado jurídico | Implementación concreta | Estado de verificación |
|---|---|---|---|
| \(T\), firmas y orígenes | Teoría inmutable de referencia | `src/typed.rs`: `Module`, `DeclarationOrigin`; `src/modules.rs`; `legal_monotonic.res`, vendor `legal_general.res` y `legal_patrimonial.res` | IMPLEMENTADO; correspondencia FINITA en `docs/legal-ontology-order-geometry.md` y su inventario. |
| \(S\in\mathcal A\) | Expediente tipado admitido | `Experiment.input`, categorías de entrada; `src/elab.rs`: `lower_ground_event` y elaboración de Seeds; `riggs_cases.res` | IMPLEMENTADO. El dominio finito y su completitud son supuestos del harness. |
| \(e\), polaridad y tiempo | Hecho o conclusión con identidad | `Event::logical_key`, `Polarity`, `TimeExpr`, `PropositionKind` en `src/typed.rs` | IMPLEMENTADO; controles de tipos/tiempos/provenance en los tests de Riggs. |
| \(C_T\) | Consecuencias de una teoría | `src/derive.rs`: `close_program`, `close_program_reference`; `src/derive/program_incremental.rs`: `IncrementalProgramClosure` | IMPLEMENTADO; leyes DEMOSTRADAS para saturación ground finita, correspondencia FINITA con el motor. |
| Activación y especialización | Aplicar comportamiento asociado a instancia conocida | `prepare_programs`, `activation_bindings`; `src/programs.rs`: `specialize_program`, `compile_program`; `src/elab.rs`: `lower_rule`, `substitute_rule_body` | IMPLEMENTADO; Riggs usa cláusulas ordinarias, no una nueva capacidad de aprendizaje de Program. |
| \(\mathcal P_T\) | Pruebas operacionales, no razones normativas | `DerivationRecord`, `Substitution`; `ProgramRuleOrigin`, `program_rule_origin`; tests Riggs: `audit_proofs`, `proof_identity`, `audit_reference` | IMPLEMENTADO; auditorías FINITAS conservan multiplicidad e ignoran sólo el sello concreto. |
| \(N_T\), \(Q\) | Resultado jurídico observado | Objetivos ground de experimentos, `Study::goal`, consulta de pertenencia en `predictions` | Proyección IMPLEMENTADA en el harness; no hay función universal de decisión jurídica. |
| \(y^*,m,\kappa\) | Valoración fundada y traducción al criterio experimental | `riggs_rule_observations.tsv`, `riggs_rule_followup_valuations.tsv`; filtrado en `learned_rule_partitions_and_counterfactual_questions_preserve_indetermination` | IMPLEMENTADO en el harness. Registro de fuente comprobado; suficiencia jurídica del fundamento, ABIERTO. |
| \(J^*\) | Justificación desconocida | No existe objeto ni oráculo de justificación verdadera en el motor/harness | ABIERTO; hipótesis que orienta el experimento. |
| \(G_\theta\), \(r_\theta\) | Distinción normativa y cláusula que la expresa | `legal_geometry_riggs_rule_learning_tests.rs`: `OPTIONS`, `generated_source`; `riggs_candidate_rules.res` | Generación IMPLEMENTADA en harness; cláusulas tipadas ejecutadas por Residuum. |
| \(T\oplus r_\theta\) | Adición hipotética de impedimento | `Study::theory`, `run`, `certify` | IMPLEMENTADO en harness; 384 equivalencias originales FINITAS. |
| \(\mathcal H_n\) | Generalizaciones aún compatibles | `predictions`, filtro de respuestas; funciones antiguas: `functions`, `predict`, `submit` en tests de recalcitrancia/conversación | IMPLEMENTADO en harness; 64→16 y 168→11→5, FINITOS y de contratos diferentes. |
| \(\mathcal B_{T,q}\) | Evidencia mínima suficiente para un objetivo | `legal_geometry_frontier_tests.rs`; `minima`, `grounding`, `saturation` en aprendizaje Riggs | Representación DEMOSTRADA en dominio finito; certificados FINITOS en `goal-frontier-results.json`, `minimal-seed-supports.tsv`. |
| \(ex_T(F)\), antiintercambio | Independencia inferencial de un estado completo | `legal_geometry_extreme_tests.rs`: `extreme_points`, `verify_basis`; test Riggs `candidate_theories_check_geometry_extreme_bases_and_seed_frontiers` | DEMOSTRADO bajo hipótesis DAG/finitez/vacío; FINITO en `extreme-results.json`, `geometry.tsv`, `extreme-bases.tsv`. |
| \(\Pi(\mathcal H)\), meet/join | Distinciones entre generalizaciones | `legal_geometry_partition_tests.rs`: `Partition::of/le/meet/join`, `all_partitions`; conversación: `kernel`, `meet`, `finer` | Leyes DEMOSTRADAS; operaciones IMPLEMENTADAS sólo en harness; 21.147 particiones FINITAS del catálogo de nueve hipótesis. |
| Casos discriminantes | Preguntas que separan candidatas | `scores`, `best` en conversación; scores del test Riggs; `pair-minimal-discriminants.tsv`, `questions.tsv` | IMPLEMENTADO en harness; minimax 8/8 y soportes, FINITOS. |
| Congruencia/cociente jurídico | Operaciones independientes de representantes | No implementación en estos experimentos; contraejemplo de compliance en §4.5 | Condición DEMOSTRADA; equivalencia automática refutada. |
| Exclusión efectiva e incorporación | Desplazar la atribución o aprobar una revisión | Ninguna operación del experimento Riggs realiza ese paso | ABIERTO; no se confunde con el impedimento Derived. |

Las operaciones de teoría deben permanecer diferenciadas:

| Operación | Situación actual |
|---|---|
| Adición monotónica de consecuencias | Soportada por reglas y usada en las teorías privadas de Riggs. |
| Excepción entre defaults | `src/defaults.rs::evaluate_defaults` permite prioridad explícita entre candidatos; no es una excepción automática al título sucesorio ni se usa aquí. |
| Restricción de aplicabilidad | Puede expresarse mediante otro cuerpo/condición de regla o `when`/`blocked when` de defaults; revisar una regla fija requiere construir otra teoría explícita. |
| Prioridad sobre consecuencias jurídicas ya derivadas | No se obtiene de la prioridad de defaults ni de `P/~P`; no implementada en Riggs. |
| Sustitución controlada de reglas | Usada en copias privadas por el harness antiguo de revisiones de compliance. No es una operación de aprendizaje del motor ni la operación elegida para Riggs. |
| Retractación de Known | No soportada por el cierre incremental de adiciones. `execute_effect` puede modificar State; eso no retracta automáticamente eventos jurídicos. |

El selector conserva `ConsistencyStatus::NotChecked` en `src/defaults.rs`.
Ninguna inspección justifica atribuirle una política de exclusión sucesoria.

## 7. Formulación mínima y preguntas abiertas

El modelo ejecutable mínimo consiste en:

\[
(T,\mathcal A,o_Q,\mathcal L,\operatorname{op},\varepsilon_1,\ldots,\varepsilon_n).
\]

La justificación \(J^*\) permanece desconocida y fuera del estado computado.
El procedimiento construye candidatas en \(\mathcal L\), las materializa como
reglas, evalúa \(T_\theta\) por separado, aplica las valoraciones documentadas y
mantiene todas las compatibles. Las fronteras describen sus requisitos mínimos;
las particiones organizan qué observaciones las distinguen; las propiedades de
clausura se vuelven a comprobar para cada teoría hipotética.

No hace falta un cociente de retículas para que este procedimiento esté bien
definido. Sí hace falta una congruencia si se quiere operar con estados cerrados
sin conservar representantes. No se introducen conexiones de Galois,
bisimulaciones ni nuevos mecanismos como condiciones previas del modelo.

Lo demostrado y lo abierto quedan ordenados por dependencia:

1. **Resultado objetivo y traducción.** Precisar si cada valoración exige una
   conclusión adicional o un resultado efectivo que excluye otra consecuencia.
   En Riggs existe el primer criterio, pero el fallo requiere el segundo.
2. **Fundamentación externa.** Obtener valoraciones jurídicas independientes de
   los contrafactuales. Los guiones actuales prueban respuestas posibles; no
   permiten elegir la respuesta por conveniencia del ajuste.
3. **Operación de revisión.** Si se exige exclusión efectiva, declarar qué
   transformación de teoría se contrasta. La adición sola no puede conseguirla.
4. **Cobertura de la gramática.** Determinar si los patrones existentes bastan
   para expresar las distinciones reveladas; un espacio vacío no autoriza ampliar
   silenciosamente la gramática.
5. **Identificabilidad.** Separar las candidatas de comportamiento distinto dentro
   del dominio admisible y el canal elegido. Los casos 8/8 avanzan en esa tarea,
   sin identificar una razón normativa verdadera.
6. **Aproximación de \(J^*\).** Formular, si se dispone de fundamento, los supuestos
   que permiten interpretar compatibilidad sucesiva como información sobre la
   justificación. Los resultados finitos actuales no prueban convergencia global.
7. **Conservación geométrica.** Certificar antiintercambio, bases y clases para
   cada nueva transformación y universo. No imponer su preservación como criterio
   jurídico ni extrapolar desde la cabeza terminal de Riggs.

El aprendizaje ya observado es real como construcción y contrastación de reglas
relacionales: produce consecuencias nuevas mediante el evaluador y reduce un
espacio explícito con observaciones. Su alcance jurídico depende de la valoración
y de la operación de revisión. Su relación con una justificación desconocida es
una hipótesis de aproximación parcial, no una identidad entre regla y razón.

## 8. Trazabilidad de esta consolidación

Se reutilizaron los contratos y resultados de:

- [Geometría de la ontología](legal-ontology-order-geometry.md),
  [extremos](legal-ontology-extreme-points.md) y
  [fronteras](legal-ontology-minimal-frontiers.md).
- [Retícula de particiones](legal-identifiability-partition-lattice.md) y
  [dependencias de revisiones](legal-revision-dependencies-and-justification-boundary.md).
- [Funciones candidatas](legal-recalcitrance-justificatory-functions.md) y
  [actualización conversacional](legal-conversational-justification-learning.md).
- [Riggs con referencia fija](legal-riggs-fixed-reference-experiment.md),
  [aprendizaje argumentativo](legal-riggs-conversational-argument-learning.md) y
  [aprendizaje de reglas](legal-riggs-normative-rule-learning.md).

Los certificados Riggs de
[summary.json](legal-geometry/riggs-rule-learning/summary.json),
[geometry.tsv](legal-geometry/riggs-rule-learning/geometry.tsv),
[closure-class-mergers.tsv](legal-geometry/riggs-rule-learning/closure-class-mergers.tsv),
[questions.tsv](legal-geometry/riggs-rule-learning/questions.tsv) y
[verification.md](legal-geometry/riggs-rule-learning/verification.md) documentan
los resultados finitos y seis tests individuales pasados anteriormente mediante
`scripts/test_safe.py`. Este informe no declara una nueva ejecución de esos tests.

Para esta consolidación se inspeccionaron fuentes y certificados; las nuevas
proposiciones y contraejemplos se argumentan en el texto. No se recompiló ni se
ejecutó la suite, no se recalcularon cierres, no se editaron fuentes previas y no
hubo commit o merge. La revisión documental comprueba enlaces locales y hashes
de los archivos preexistentes, junto con la conservación de `main` y `v1`.
