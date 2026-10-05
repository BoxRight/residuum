module EffectBodyConstants

entity Marker
const Fixed : Marker

entity Entry {
    marker : Marker
}
entity Bucket {
    entries : Set Entry
    markers : Set Marker
}

effect verb capture(subject : Bucket) =>
    subject.entries += Entry {
        marker = Fixed,
    }

// Un parámetro local mantiene precedencia sobre una constante del módulo.
effect verb captureArgument(subject : Bucket, Fixed : Marker) =>
    subject.entries += Entry {
        marker = Fixed,
    }

effect verb addMarker(subject : Bucket) =>
    subject.markers += Fixed

effect verb removeMarker(subject : Bucket) =>
    subject.markers -= Fixed
