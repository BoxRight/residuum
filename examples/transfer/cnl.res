module Transfer

a Person has a set of Assets called assets.

"transfers" means:
    a Person transfers an Asset to a Person,
    causing the object to be removed from the subject's assets, and added to the recipient's assets.

"delivers" means:
    a Person delivers an Asset to a Person.

rule transfer:

    when a giver delivers an asset to a receiver at time t,
    the giver transfers the asset to the receiver after t.

default transferNormally: normally use transfer.
