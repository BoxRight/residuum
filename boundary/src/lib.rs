//! Communication boundary of the exploration worker (invocation protocol §9):
//! versioned wire DTOs and the line transport, as adapters outside the
//! semantic environment. This crate depends on `residuum`; `residuum` never
//! depends on it. Serialisation lives only here.

pub mod manifest;
pub mod startup;
pub mod transport;
pub mod wire;

#[cfg(test)]
mod process_script_tests;
#[cfg(test)]
mod serde_probe_tests;
#[cfg(test)]
mod startup_tests;
#[cfg(test)]
mod transport_tests;
#[cfg(test)]
mod wire_tests;
