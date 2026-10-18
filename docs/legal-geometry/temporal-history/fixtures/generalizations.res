module Generalizations

import "vocabulary.res"

// The space of generalizations used by the formalizations. No rule mentions a
// dossier moment, an individual dossier or a learning epoch. Every rule has
// the proposition time as its FIRST parameter, so that native partial
// application can specialize it (alternative B); programs bind it by name
// (alternative C).

rule ordinaryBan(t, owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t & normEffective(dogBan, e) @ t
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, e) @ t

rule refinedBan(t, owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t & ~propertyA(dog) @ t & normEffective(dogBan, e) @ t
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, e) @ t

rule registrationDuty(t, owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t & propertyA(dog) @ t & normEffective(regDuty, e) @ t
    <= obligation(regDuty, owner, register, dog, venue, administrative, e) @ t

// Passive definitions. A rule used by a Program associated with a verb is a
// definition, not an ordinary rule (prepare_programs). templatesDefined is
// never derived nor seeded, so this Program never runs; its only effect is that
// no template is active as a general rule in any module importing this one.
program Templates(t : PropositionTime) {
    use ordinaryBan { t = t, }
    use refinedBan { t = t, }
    use registrationDuty { t = t, }
}

derived verb templatesDefined() = program Templates
