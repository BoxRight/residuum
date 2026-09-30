module Legal

a Movable is a Thing.
a Immovable is a Thing.
a Fungible is a Thing.
a Money is a Fungible.

a Conduct has a Thing called conduct.
a Conduct has a optional Thing called indirectObject.

a Person has a Patrimony called patrimony.
a Patrimony has a set of Things called assets.
a Patrimony has a set of Relations called rights.
a Patrimony has a set of Relations called obligations.

a Relation has a Person called debtor.
a Relation has a optional Thing called thing.
a Relation has a optional Conduct called conduct.
a Relation has a Person called creditor.

"give" means:
    a Person gives a Thing to a Person,
    causing a Relation with debtor subject, thing object, conduct none, and creditor recipient to be added to the subject's patrimony obligations,
    and causing a Relation with debtor subject, thing object, conduct none, and creditor recipient to be added to the recipient's patrimony rights.

"agreesPrice" means:
    a Person agreesPrice a Thing to a Person.

"agreesObject" means:
    a Person agreesObject a Thing to a Person.

rule sales:
    when a debtor agreesPrice the object to the creditor at time t,
    and when a creditor agreesObject the object to the debtor at time t,
    the debtor gives the object to the creditor after t.
