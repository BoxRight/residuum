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

"do" means:
    a Person does a Conduct with an optional Thing for a Person,
    causing a Relation with debtor subject, thing indirectObject, conduct conduct, and creditor beneficiary to be added to the subject's patrimony obligations,
    and causing a Relation with debtor subject, thing indirectObject, conduct conduct, and creditor beneficiary to be added to the beneficiary's patrimony rights.

"notDo" means:
    a Person notDo a Conduct with an optional Thing for a Person,
    causing a Relation with debtor subject, thing indirectObject, conduct conduct, and creditor beneficiary to be added to the subject's patrimony obligations,
    and causing a Relation with debtor subject, thing indirectObject, conduct conduct, and creditor beneficiary to be added to the beneficiary's patrimony rights.
