use tracing::{Level, event, instrument};
use std::fmt;
use anyhow::{Result, Context};
use std::sync::{Arc, Mutex};
use crate::models::{
    UnitArc,
    ValueSet,
    Meta,
    Dependency,
    Operation,
    OpCompletion,
};
use crate::events::OpEvent;
use super::Context as EngineContext;
use super::shell_executor::{ShellExecutor, ExecutorArc};

use std::collections::HashMap;

pub type ExecutionArc = Arc<Mutex<UnitExecution>>;

/// Manages execution of the given unit on the given executor. Caches operation results so
/// operations are not run multiple times.
pub struct UnitExecution {
    unit: UnitArc,
    ctx: EngineContext,
    script: String,
    meta: Option<Arc<Meta>>,
    emit_data: Option<Arc<ValueSet>>,
    args: ValueSet,
}

impl fmt::Debug for UnitExecution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UnitExecution {{ unit: {:?} }}", self.unit.get_id())?;
        Ok(())
    }
}

impl UnitExecution {
    pub async fn new(unit: UnitArc, executor: ShellExecutorArc, ctx: EngineContext, script: String) -> Result<UnitExecution> {
        Ok(UnitExecution {
            unit,
            ctx,
            script,
            emit_data: None,
            meta: None,
            args: ValueSet::new().into().into(),
        })
    }

    pub async fn set_args(&mut self, args: &ValueSet) {
        self.args.merge(args);
    }

    pub async fn get_meta(&self) -> Arc<Meta> {
        match self.meta {
            Some(ref meta) => meta.clone(),
            None => panic!("Meta not set on unit: {}", &self.unit),
        }
    }

    pub async fn get_emit_data(&mut self) -> Result<Arc<ValueSet>> {
        match self.emit_data {
            Some(ref emit_data) => Ok(emit_data.clone()),
            None => panic!("Emit data not set on unit: {}", &self.unit),
        }
    }

    /// Caches emit data from an operation
    pub fn add_emit_data(&mut self, emit_data: Arc<ValueSet>) {
        let existing_emit_data = Arc::make_mut(&mut self.emit_data);
        existing_emit_data.merge(emit_data.as_ref());
    }
}
