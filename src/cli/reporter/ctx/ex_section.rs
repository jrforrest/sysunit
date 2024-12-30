//! Renders the loading and run sections in Sysu output
use super::*;
use crate::events::Event;
use crate::models::{UnitArc, Operation};

pub struct Ctx {
    out: Out,
    state: State,
    v: V,
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
        }
    }

    pub fn handle(&mut self, e: Event) {
        use State::*;
        match (&mut self.state, &e) {
            (Root, E::Op(unit, op, OpE::Started)) => {
                self.enter_op(unit.clone(), op.clone())
            },
            (Op(ref mut op_ctx), E::Op(unit, op, _)) => {
                if op_ctx.matches(&unit, op)  {
                    op_ctx.handle(e);
                } else {
                    self.enter_state(Root);
                    self.handle(e)
                }
            },
            _ => unreachable!(),
        }
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
