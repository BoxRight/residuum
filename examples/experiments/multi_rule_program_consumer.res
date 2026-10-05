module MultiRuleProgramConsumer

import "./associated_multi_rule_program.res"

// No copia el cuerpo; mantiene las referencias y el ámbito léxico importado.
seeded verb consumerStarts(object : Thing) = program Chain
program ImportedGround = Partial(tau)

experiment imported {
    seeds { consumerStarts(item) @ tau }
    query residual completed(item) @ after tau
}
