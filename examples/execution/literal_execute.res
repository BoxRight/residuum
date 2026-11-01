module LiteralExecute

entity Marker
const Fixed : Marker

entity Bucket {
    markers : Set Marker
}

const bucket : Bucket
const tau : PropositionTime

seeded verb clear(subject : Bucket)

effect verb removeMarker(subject : Bucket) =>
    subject.markers -= Fixed

rule clearing(subject, t) =
    clear(subject) @ t
    <= removeMarker(subject) @ after t

// The literal constant in the effect body removes the declared element.
experiment cleared {
    seeds { clear(bucket) @ tau, }
    state @ tau { bucket.markers = {Fixed} }
    query residual removeMarker(bucket) @ after tau
}
