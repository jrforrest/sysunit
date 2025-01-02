use crate::events::{Event as E, OpEvent as OpE};
use super::Verbosity as V;
use colored::*;

mod root;
mod ex_section;
mod op;

pub use root::Ctx as RootContext;

/// Output handle that can be passed between contexts.  Makes
/// indentation for nested contexts a little easier.
#[derive(Clone)]
struct Out {
    idt: usize,
}

impl Out {
    fn new(idt: usize) -> Self {
        Self { idt }
    }

    fn dedent(&mut self) {
        if self.idt >= 2 {
            self.idt -= 2
        }
    }

    fn indent(&mut self) {
        self.idt += 2
    }

    fn ln(&self, s: &str) {
        println!("{:indent$}{}", "", s, indent = self.idt);
    }
}

enum Completion {
    Pending,
    Complete,
    Unhandled,
}
