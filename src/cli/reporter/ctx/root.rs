use super::*;
use super::ex_section;
use crate::models::UnitArc;

pub struct Ctx {
    state: State,
    out: Out,
    v: V,
}

enum State {
    Root,
    Loading(ex_section::Ctx),
    ExecutionPlan,
    Running(ex_section::Ctx),
    Final
}

use std::fmt::{self, Display};
impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use State::*;
        write!(f, "{}", match self {
            Root => "Initial",
            Loading(_) => "Loading",
            ExecutionPlan => "ExecutionPlan",
            Running(_) => "Running",
            Final => "Final",
        })
    }
}

impl Ctx {
    pub fn new(v: V) -> Self {
        Self { state: State::Root, out: Out::new(0), v }
    }

    pub fn handle(&mut self, ev: E) {
        use State::*;
        match (&mut self.state, &ev) {
            (Root, E::Resolving) => self.enter_load(),
            (Loading(ctx), E::Op(..)) => ctx.handle(ev),
            (Loading(..), E::Resolved(units)) => {
                self.out.dedent();
                self.enter_state(ExecutionPlan);
                self.ex_plan(units);

            },
            (ExecutionPlan, E::Op(..)) => {
                self.out.dedent();
                self.enter_running();
                self.handle(ev)
            }
            (Running(ctx), E::Op(..)) => ctx.handle(ev),
            (Final, E::EngineSuccess) => {
                self.out.ln(&format!("{}", "Success".green().bold()));
            },
            (Final, E::Error(msg)) => {
                self.out.ln(&format!("{}", "Error".red().bold()));
                self.out.ln(&msg);
            },
            (_, E::Debug(msg)) => {
                if self.v >= V::Debug {
                    self.out.ln(msg);
                }
            }
            (_, E::EngineSuccess | E::Error(_)) => {
                self.out.dedent();
                self.enter_state(Final);
                self.handle(ev)
            },
            _ => unreachable!(),
        }
    }

    fn ex_plan(&self, units: &Vec<UnitArc>) {
        for unit in units.iter() {
            self.out.ln(&format!("{}", unit.name));
        }
    }

    fn enter_state(&mut self, state: State) {
        use State::*;
        match state {
            Root => (),
            _ => {
                self.out.ln(&format!("[ {} ]", state));
                self.out.indent();
            }
        }
        self.state = state;
    }

    fn enter_running(&mut self) {
        self.enter_state(State::Running(self.ex_context()));
    }

    fn enter_load(&mut self) {
        self.enter_state(State::Loading(self.ex_context()));
    }

    fn ex_context(&self) -> ex_section::Ctx {
        let mut out = self.out.clone();
        out.indent();
        ex_section::Ctx::new(self.v.clone(), out)
    }
}
