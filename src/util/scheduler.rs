use std::{cmp::Reverse, collections::BinaryHeap, time::Instant};

struct Command<T> {
    when: Instant,
    func: Box<dyn FnOnce(&mut T)>,
}

impl<T> Command<T> {
    fn new<F: FnOnce(&mut T) + 'static>(when: Instant, func: F) -> Self {
        Self {
            when,
            func: Box::new(func),
        }
    }
}

impl<T> PartialEq for Command<T> {
    fn eq(&self, other: &Self) -> bool {
        self.when.eq(&other.when)
    }
}

impl<T> Eq for Command<T> {}

impl<T> PartialOrd for Command<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Command<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.when.cmp(&other.when)
    }
}

pub struct Scheduler<T> {
    heap: BinaryHeap<Reverse<Command<T>>>,
}

impl<T> Scheduler<T> {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn schedule<F: FnOnce(&mut T) + 'static>(&mut self, when: Instant, func: F) {
        let cmd = Command::new(when, func);
        self.heap.push(Reverse(cmd))
    }

    pub fn update(&mut self, now: Instant, this: &mut T) {
        while let Some(Reverse(c)) = self.heap.peek()
            && c.when > now
        {
            let cmd = unsafe { self.heap.pop().unwrap_unchecked() }.0;
            (cmd.func)(this);
        }
    }
}
