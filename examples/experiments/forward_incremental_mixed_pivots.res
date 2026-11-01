module ForwardIncrementalMixedPivots

entity Person

const ana : Person
const beto : Person
const carla : Person
const tau : PropositionTime

seeded verb present(person : Person)
seeded verb motive(person : Person)
derived verb suspect(person : Person)
derived verb coSuspects(first : Person, second : Person)

rule suspicion(person, t) =
    present(person) @ t
    & motive(person) @ t
    <= suspect(person) @ t

rule pairing(first, second, t) =
    suspect(first) @ t
    & suspect(second) @ t
    <= coSuspects(first, second) @ t

experiment universe {
    seeds {
        present(ana) @ tau,
        present(beto) @ tau,
        present(carla) @ tau,
        motive(ana) @ tau,
        motive(beto) @ tau,
        motive(carla) @ tau,
    }
    query residual suspect(carla) @ tau
}
