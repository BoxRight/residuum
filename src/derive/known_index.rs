//! Header-only index. Positions preserve the order and representation of Known.
use super::{Event, Polarity};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub(super) struct KnownIndex {
    buckets: BTreeMap<String, BTreeMap<(Polarity, usize), Vec<usize>>>,
}
impl KnownIndex {
    pub fn new(known: &[Event]) -> Self {
        let mut index = Self::default();
        for (position, event) in known.iter().enumerate() {
            index.insert(position, event);
        }
        index
    }
    pub fn insert(&mut self, position: usize, event: &Event) {
        self.buckets
            .entry(event.verb.clone())
            .or_default()
            .entry((event.polarity, event.args.len()))
            .or_default()
            .push(position);
    }
    fn positions(&self, pattern: &Event) -> &[usize] {
        self.buckets
            .get(&pattern.verb)
            .and_then(|bucket| bucket.get(&(pattern.polarity, pattern.args.len())))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

#[derive(Clone, Copy)]
pub(super) struct KnownView<'a> {
    pub events: &'a [Event],
    pub index: Option<&'a KnownIndex>,
}
impl<'a> KnownView<'a> {
    pub fn linear(events: &'a [Event]) -> Self {
        Self {
            events,
            index: None,
        }
    }
    pub fn candidates(self, pattern: &Event) -> Candidates<'a> {
        match self.index {
            Some(index) => Candidates::Indexed {
                events: self.events,
                positions: index.positions(pattern).iter(),
            },
            None => Candidates::Linear(self.events.iter()),
        }
    }
}
pub(super) enum Candidates<'a> {
    Linear(std::slice::Iter<'a, Event>),
    Indexed {
        events: &'a [Event],
        positions: std::slice::Iter<'a, usize>,
    },
}
impl<'a> Iterator for Candidates<'a> {
    type Item = &'a Event;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Linear(events) => events.next(),
            Self::Indexed { events, positions } => positions.next().map(|p| &events[*p]),
        }
    }
}
