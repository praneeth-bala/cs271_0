use std::collections::BinaryHeap;

#[derive(Clone, Eq, PartialEq)]
pub struct LamportEntry {
    pub lamport_clock: u64,
    pub client_id: u64,
}

impl Ord for LamportEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.lamport_clock, self.client_id).cmp(&(other.lamport_clock, other.client_id))
    }
}

impl PartialOrd for LamportEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct LamportQueue {
    queue: BinaryHeap<LamportEntry>,
    clock: u64,
}

impl LamportQueue {
    pub fn new() -> LamportQueue {
        Self { queue: BinaryHeap::<LamportEntry>::new(), clock: 0 }
    }

    pub fn insert(&mut self, lamport_clock: u64, client_id: u64){
        self.queue.push(LamportEntry{lamport_clock, client_id});
    }

    pub fn pop(&mut self){
        self.queue.pop();
    }

    pub fn peek(&self) -> Option<&LamportEntry> {
        self.queue.peek()
    }

    pub fn increment(&mut self) {
        self.clock += 1;
    }

    pub fn update(&mut self, other_clock: u64) {
        self.clock = self.clock.max(other_clock) + 1;
    }

    pub fn get_clock(&self) -> u64 {
        self.clock
    }
}
