module AssociatedGivingProgram

import "./legal_monotonic.res"

// Lambda del experimento de asociación anterior; fuera de la ontología congelada.
rule givingProgram(subject, object, recipient, t) =
    I <= give(subject, object, recipient) @ after t
