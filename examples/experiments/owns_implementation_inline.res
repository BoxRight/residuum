module OwnsImplementationInline

rule inlineProtection(owner, object, t) =
    owns(owner, object) @ t
    <= notDo(charlie, molest, object, owner) @ after t
