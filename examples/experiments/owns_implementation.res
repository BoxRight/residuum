module OwnsImplementation

rule ownsProgram(thirdParty, conduct, owner, object, t) =
    owns(owner, object) @ t
    <= notDo(thirdParty, conduct, object, owner) @ after t

rule implementedProtection = ownsProgram(charlie, molest)
