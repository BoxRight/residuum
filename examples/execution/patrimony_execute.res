module PatrimonyExecute

import "../legal/give_formal.res"

const juan : Person
const pedro : Person
const juanPatrimony : Patrimony
const pedroPatrimony : Patrimony
const car : Movable
const tau : PropositionTime

seeded verb sells(subject : Person, object : Thing, recipient : Person)

rule saleGives(seller, object, buyer, t) =
    sells(seller, object, buyer) @ t
    <= give(seller, object, buyer) @ after t

// The legal `give` reaches the sets through `subject.patrimony.assets`, so the
// state declares each person's patrimony reference and the patrimony's sets.
experiment sale {
    seeds { sells(juan, car, pedro) @ tau, }
    state @ tau {
        juan.patrimony = juanPatrimony,
        pedro.patrimony = pedroPatrimony,
        juanPatrimony.assets = {car},
        pedroPatrimony.assets = {},
    }
    query residual give(juan, car, pedro) @ after tau
}
