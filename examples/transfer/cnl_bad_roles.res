module Transfer

a Person has a set of Things called assets.

"delivers" means:
    a Person delivers a Thing to a Person.

"acquires" means:
    a Person acquires a Thing,
    causing that Thing to be added to that Person's assets.

rule badDelivery:

    when a thing delivers a person to a person at time t,
    the person acquires the thing after t.
