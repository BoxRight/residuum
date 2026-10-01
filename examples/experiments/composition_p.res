module SalesProgram

seeded verb sells(subject : Person, object : Thing, recipient : Person) = program sellsProgram

rule sellsProgram(seller, object, buyer, t) =
    sells(seller, object, buyer) @ t
    <= owns(buyer, object) @ after t
