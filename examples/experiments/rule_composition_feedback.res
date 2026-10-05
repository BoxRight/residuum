module RuleCompositionFeedback

entity Item
const item : Item
const tau : PropositionTime

seeded verb start(object : Item)
derived verb intermediate(object : Item)
derived verb returned(object : Item)
derived verb completed(object : Item)
derived verb completionObserved(object : Item)

// P contiene pEntry, pFeedback y observeCompletion; Q contiene qStep.
// Q puede habilitar una relación de P que no estaba habilitada al ejecutarlo.
// La fuente sólo escribe @ t y @ after t. El encadenamiento anida After.
rule pEntry(t, object) =
    start(object) @ t <= intermediate(object) @ after t
rule pFeedback(object, t) =
    returned(object) @ t <= completed(object) @ after t
rule qStep(object, t) =
    intermediate(object) @ t <= returned(object) @ after t

// Observación consultable con sintaxis existente, no una reescritura del tiempo
// de completed. El test inspecciona por separado su tiempo interno anidado.
rule observeCompletion(object, t) =
    completed(object) @ t <= completionObserved(object) @ tau

// β fija el tiempo de entrada; en las otras relaciones sólo fija el objeto
// y conserva t ligado, para que pueda recibir el tiempo del evento anterior.
rule pAtTau = pEntry(tau)
rule qForItem = qStep(item)
rule feedbackForItem = pFeedback(item)

experiment feedback {
    seeds { start(item) @ tau, }
    query residual completionObserved(item) @ tau
}
