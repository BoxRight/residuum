module OwnershipProgram

derived verb owns(owner : Person, object : Thing) = program protectionFromCharlie

rule ownsProgram(thirdParty, conduct, owner, object, t) =
    owns(owner, object) @ t
    <= notDo(thirdParty, conduct, object, owner) @ after t

rule protectionFromCharlie = ownsProgram(charlie, molest)
