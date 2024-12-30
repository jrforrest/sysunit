/// Events are emitted to oservers so the stages of engine
/// and unit execution can be reported to the CLI, logging
/// and telemetry.

use crate::models::{UnitArc, Operation, OpCompletion, StdoutData};
use std::sync::{Arc, Mutex};
use anyhow::Result;

/// Events that can be emitted by the engine
#[derive(Clone, Debug)]
pub enum Event {
    Resolving,
    Resolved(Vec<UnitArc>),
    Op(UnitArc, Operation, OpEvent),
    Debug(String),
    EngineSuccess,
    Error(String),
}

/// Events emitted while an operation is being executed on a unit
#[derive(Clone, Debug)]
pub enum OpEvent {
    Started,
    Output(StdoutData),
    Complete(OpCompletion),
    Error(String),
}

/// Observers watch for events to occur so they can report
pub trait Observer: Send {
    fn handle(&mut self, event: Event) -> Result<()>;
}

pub type ObserverArc = Arc<Mutex<dyn Observer>>;

/// Provides an interface to dispatch events to multiple observers
#[derive(Clone)]
pub struct EventHandler {
    observers: Vec<ObserverArc>,
}

impl EventHandler {
    pub fn new(observers: Vec<ObserverArc>) -> Self {
        Self { observers }
    }

    pub fn handle(&self, event: Event) -> Result<()> {
        for observer in self.observers.iter() {
            observer.lock().unwrap().handle(event.clone())?;
        }

        Ok(())
    }

    pub fn get_op_handler(&self, unit: UnitArc, op: Operation) -> OpEventHandler {
        OpEventHandler::new(self.clone(), unit, op)
    }
}

/// Reports events pertaining to the execution of a specific unit
#[derive(Clone)]
pub struct OpEventHandler {
    ev_handler: EventHandler,
    unit: UnitArc,
    op: Operation,
}

impl OpEventHandler {
    pub fn new(ev_handler: EventHandler, unit: UnitArc, op: Operation) -> Self {
        Self { unit, ev_handler, op }
    }

    pub fn handle(&self, event: OpEvent) -> Result<()> {
        self.ev_handler.handle(Event::Op(self.unit.clone(), self.op, event))
    }
}
