use super::*;
use crate::models::{UnitArc, Operation, StdoutData, OpCompletion};
use std::fmt::{self, Display};

pub struct Ctx {
    unit: UnitArc,
    op: Operation,
    state: State,
    out: Out,
    v: V,
    diag_buf: Vec<OpE>,
}

#[derive(Debug)]
enum State {
    Root,
    Output,
    Error,
    EmitData
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use State::*;
        write!(f, "{}", match self {
            Root => "Initial",
            Output => "Output",
            Error => "Error",
            EmitData => "EmitData",
        })
    }
}

impl Ctx {
    pub fn new(unit: UnitArc, op: Operation, v: V, out: Out) -> Self {
        Self {
            unit,
            op,
            state: State::Root,
            v,
            out,
            diag_buf: Vec::new(),
        }
    }

    pub fn get_header(&self) -> String {
        let opstr = match self.op {
            Operation::Meta => "Meta",
            Operation::Deps => "Dependencies",
            Operation::Check => "Check",
            Operation::Apply => "Apply",
            Operation::Remove => "Remove"
        };

        format!("[ {} | {} ]", opstr, self.unit.tag())
    }

    pub fn matches(&self, unit: &UnitArc, op: &Operation) -> bool {
        self.unit == *unit && self.op == *op
    }

    pub fn handle(&mut self, e: E) {
        match e {
            // We handle any events meant for our current unit and op
            E::Op(u, o, op_e) if self.unit == u && self.op == o => self.handle_op_ev(op_e),
            // And notify the caller if we can't handle it
            _ => unreachable!(),
        }
    }

    fn handle_op_ev(&mut self, op_e: OpE) {
        use State::*;
        use V::*;
        match (&self.state, &op_e) {
            (Root, OpE::Started) => (),
            (Root, OpE::Output(StdoutData::TextLine(_))) => {
                if self.v >= Verbose {
                    self.enter_state(Output);
                    self.handle_op_ev(op_e)
                } else {
                    // Save the output for future diagnostics if we error out
                    self.diag_buf.push(op_e);
                }
            },
            (Root, OpE::Output(StdoutData::Message(_))) => {
                if self.v >= Verbose {
                    self.enter_state(EmitData);
                    self.handle_op_ev(op_e)
                } else {
                    // Save the output for future diagnostics if we error out
                    self.diag_buf.push(op_e);
                }
            },
            (Output, OpE::Output(StdoutData::TextLine(o))) => {
                self.out.ln(o);
            },
            (EmitData, OpE::Output(StdoutData::Message(m))) => {
                let header = match &m.header.field {
                    Some(s) => &format!("{}.{}", m.header.name, s),
                    None => &m.header.name,
                };
                self.out.ln(&format!("{} | {}", header, m.text));
            },
            (Output | EmitData, _) => {
                self.enter_state(Root);
                self.handle_op_ev(op_e);
            },
            (Root, OpE::Complete(op_completion)) => {
                match op_completion {
                    OpCompletion::Check(_, present, _) => {
                        if *present {
                            self.out.ln(&format!("{}", "Present".green().bold()));
                        } else {
                            self.out.ln(&format!("{}", "Pending".yellow().bold()));
                        }
                    },
                    _ => {
                        self.out.ln(&format!("{}", "OK".green().bold()));
                    }
                }
            }
            (_, OpE::Error(msg)) => {
                if ! matches!(self.state, State::Root) {
                    self.enter_state(State::Root)
                }

                let replay_evs = std::mem::take(&mut self.diag_buf);
                self.v = V::Verbose;
                for op_ev in replay_evs {
                    self.handle_op_ev(op_ev.clone())
                }

                if ! matches!(self.state, State::Root) {
                    self.enter_state(State::Root)
                }

                self.enter_state(Error);
                self.out.ln(&format!("{}", msg.red()));
            }
            // The error state can only handle error events
            (Error, _) => unreachable!()
        }
    }

    fn enter_state(&mut self, state: State) {
        use State::*;
        match &state {
            Root => self.out.dedent(),
            Error => {
                self.out.ln(&format!("[ {} ]", "Error".red().bold()));
                self.out.indent();
            }
            state => {
                self.out.ln(&format!("[ {} ]", state));
                self.out.indent();
            }
        }
        self.state = state;
    }
}
