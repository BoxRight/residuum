module InlineComposition

rule sellsProgram(seller, object, buyer, t) =
    sells(seller, object, buyer) @ t
    <= owns(buyer, object) @ after t

rule protectionFromCharlie(owner, object, t) =
    owns(owner, object) @ t
    <= notDo(charlie, molest, object, owner) @ after t
