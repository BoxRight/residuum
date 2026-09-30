module Legal

there is an entity called Thing.

there is a Conduct called deliver.

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

"do" means:
    a Person does a Conduct with an optional Thing for a Person,
    causing a Relation with debtor subject, thing indirectObject, conduct conduct, and creditor beneficiary to be added to the subject's patrimony obligations,
    and causing a Relation with debtor subject, thing indirectObject, conduct conduct, and creditor beneficiary to be added to the beneficiary's patrimony rights.

rule traditio(conduct, debtor, object, creditor, t):
    when a debtor gives the object to the creditor at time t,
    the debtor does the conduct with the object for the creditor after t.

rule delivery:
    use traditio with deliver.
