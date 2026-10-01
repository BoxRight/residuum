module ProtectionObservation

rule observeProtection(subject, conduct, object, owner, t) =
    notDo(subject, conduct, object, owner) @ t
    <= protected(owner, object) @ t
