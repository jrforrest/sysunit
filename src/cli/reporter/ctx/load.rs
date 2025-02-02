//! Shows progress of loading stage
use super::*;
use crate::events::{Event, OpEvent};
use crate::models::{UnitArc, Operation};

pub struct Ctx {
    out: Out,
    state: State,
    v: V,
    diag_buf: Vec<E>,
}

enum State {
    Root,
    Op(op::Ctx),
}

impl Ctx {
    pub fn new(v: V, out: Out) -> Self {
        Self {
            v,
            out,
            state: State::Root,
            diag_buf: Vec::new(),
        }
    }

    pub fn handle(&mut self, e: Event) {
        use State::*;
        match (&mut self.state, &e) {
            // If we are in the top-level loading context and we encounter an error, that
            // means we are buffering unit operation events, so we should replay them
            // so the operations fully render and the error can be displayed with context
            (Root, E::Op(_, _, OpEvent::Error(_))) => {
                let replay_evs = std::mem::take(&mut self.diag_buf);
                for diag_e in replay_evs {
                    self.handle(diag_e.clone());
                }
                self.v = V::Verbose;
                self.handle(e);
            },
            // If we're not in verbse mode, collect unit operations into the diag buffer
            // so they can be replayed if we encounter an error
            (Root, E::Op(_, _, _)) if self.v < V::Verbose => {
                self.diag_buf.push(e);
            },
            (Root, E::Op(unit, op, OpE::Started)) => {
                self.enter_op(unit.clone(), *op)
            },
            (Op(ref mut op_ctx), E::Op(unit, op, _)) => {
                if op_ctx.matches(unit, op)  {
                    op_ctx.handle(e);
                } else {
                    self.enter_state(Root);
                    self.handle(e)
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn report_ok(&self) {
        self.out.ln(&format!("{}", "OK".green().bold()));
    }

    fn enter_op(&mut self, unit: UnitArc, op: Operation) {
        let mut out = self.out.clone();
        out.indent();
        let op_ctx = op::Ctx::new(unit, op, self.v.clone(), out);
        self.enter_state(State::Op(op_ctx));
    }

    fn enter_state(&mut self, state: State) {
        use State::*;
        match &state {
            Root => self.out.dedent(),
            Op(ctx) => {
                self.out.ln(&ctx.get_header());
                self.out.indent();
            },
        }
        self.state = state;
    }
}
