// Transfer with comments before and between declarations.
module/* A comment also separates tokens. */Transfer

entity Thing # A line comment can follow a declaration.

/* Records retain their original structure.
   Comment text may contain Unicode: ≤ ⊸ ~, and // or #. */
entity Person {
    assets : Set/* collection */Thing
}

seeded verb delivers(
    subject   : Person, // giver
    object    : Thing,
    recipient : Person # receiver
)

effect verb acquires(
    subject : Person,
    object  : Thing
) =>
    subject./* field access */assets += object

rule deliveryAcquiresAsset(
    giver,
    object,
    receiver,
    t
) =
    delivers(giver, object, receiver) @ t
    ≤ /* Temporal consequence. */
    acquires(receiver, object) @ after t
// End of program.
