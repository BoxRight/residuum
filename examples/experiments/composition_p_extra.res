module Registration

rule registerOwnership(owner, object, t) =
    owns(owner, object) @ t
    <= registered(owner, object) @ t
