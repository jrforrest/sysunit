//! Handles execution of operations on units
use anyhow::Result;
use crate::models::{
    ValueSet,
    Meta,
    Dependencies,
};
use crate::events::OpEventHandler;
use super::executor_pool::ExecutorArc;


/// Manages execution of the given unit. Caches operation results so operations are not run multiple times.
#[derive(Debug)]
pub struct UnitExecution {
    pub script: String,
    pub args: ValueSet,
    pub emit_data: ValueSet,
    pub deps: Option<Dependencies>,
    meta: Option<Meta>,
}

impl UnitExecution {
    pub async fn new(script: String) -> Result<UnitExecution> {
        Ok(UnitExecution {
            script,
            emit_data: ValueSet::new(),
            meta: None,
            deps: None,
            args: ValueSet::new(),
        })
    }

    /// Sets arguments on the unit that will be used for all proceeding operations
    pub async fn set_args(&mut self, args: &ValueSet) {
        self.args.merge(args);
    }

    /// Gets and caches meta data for the unit, running the meta operation on the given executor
    /// with events reported to op_ev_handler if it has not yet been fetched
    pub async fn get_meta(&mut self, executor: ExecutorArc, op_ev_handler: OpEventHandler) -> Result<&Meta> {
        match self.meta {
            Some(ref meta) => Ok(meta),
            None => {
                let mut executor = executor.lock().unwrap();
                let meta = executor.get_meta(op_ev_handler, &self.script, &self.args).await?;
                self.meta = Some(meta);
                Ok(self.meta.as_ref().unwrap())
            },
        }
    }

    /// Gets and caches dependencies for the unit, running the deps operation on the given executor
    /// with events reported to op_ev_handler if they have not yet been fetched
    pub async fn get_deps(&mut self, executor: ExecutorArc, op_ev_handler: OpEventHandler) -> Result<&Dependencies> {
        match self.deps {
            Some(ref deps) => Ok(deps),
            None => {
                let mut executor = executor.lock().unwrap();
                let deps = executor.get_deps(op_ev_handler, &self.script, &self.args).await?;
                self.deps = Some(deps);
                Ok(self.deps.as_ref().unwrap())
            },
        }
    }

    pub async fn remove(&mut self, executor: ExecutorArc, op_ev_handler: OpEventHandler) -> Result<()> {
        let mut executor = executor.lock().unwrap();
        let emit_data = executor.remove(op_ev_handler, &self.script, &self.args).await?;
        self.emit_data.merge(&emit_data);
        Ok(())
    }

    pub async fn apply(&mut self, executor: ExecutorArc, op_ev_handler: OpEventHandler) -> Result<()> {
        let mut executor = executor.lock().unwrap();
        let emit_data = executor.apply(op_ev_handler, &self.script, &self.args).await?;
        self.emit_data.merge(&emit_data);
        Ok(())
    }

    pub async fn check(&mut self, executor: ExecutorArc, op_ev_handler: OpEventHandler) -> Result<bool> {
        let mut executor = executor.lock().unwrap();
        let (status, emit_data) = executor.check(op_ev_handler, &self.script, &self.args).await?;
        self.emit_data.merge(&emit_data);
        Ok(status)
    }
}
