use tracing::{Level, event, instrument};
use std::fmt;
use anyhow::{Result, Context};
use crate::models::{
    UnitArc,
    ValueSet,
    Meta,
    Dependency,
    Operation,
    OpCompletion,
};
use crate::events::{EventHandler, OpEvent};
use super::shell_executor::ShellExecutor;

use std::sync::Arc;

use std::collections::HashMap;

/// Contains state and cached data for a unit execution
///
/// When a unit's execution has completed, its executor is allowed to finalize.  However,
/// things like emitted values, params and deps may still be needed by the execution of other
/// units further up the dependency stack.  We retain this state here behind the same interface
/// as the executor for convenience.
///
/// More context for error messages coming from the executor are also added here to remove
/// repetitive work from implementing more executors.
///
/// This struct relies on careful ordering of operations, and will panic if methods are called
/// in the wrong order.
///
/// TODO: This should probably be refactored into a state machine
pub struct UnitExecution {
    unit: UnitArc,
    executor_state: ExecutorState,
    ev_handler: EventHandler,
    emit_data: Arc<ValueSet>,
    executed_operations: HashMap<Operation, OpCompletion>,
}

impl fmt::Debug for UnitExecution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let executor_state = match self.executor_state {
            ExecutorState::Running(_) => "Running",
            ExecutorState::Finished => "Finished",
        };

        write!(f, "UnitExecution {{ unit: {:?}, state: {:?}",
            self.unit.get_id(),
            executor_state,
        )?;
        Ok(())
    }
}

impl UnitExecution {
    pub async fn init(unit: UnitArc, ev_handler: EventHandler, script: &str) -> Result<UnitExecution> {
        let executor = ShellExecutor::init(unit.clone(), script, ev_handler.clone()).await?;

        Ok(UnitExecution {
            unit,
            ev_handler,
            executed_operations: HashMap::new(),
            emit_data: ValueSet::new().into(),
            executor_state: ExecutorState::Running(executor),
        })
    }

    /// Runs an operation on the unit, providing a cached result if it has already been run.
    /// Panics if the operation has not yet been run but the executor has already exited.
    #[instrument]
    async fn run_op(&mut self, op: Operation) -> Result<OpCompletion> {
        use std::collections::hash_map::Entry;

        match self.executed_operations.entry(op) {
            Entry::Occupied(entry) => {
                Ok(entry.get().clone())
            },
            Entry::Vacant(entry) => {
                event!(Level::INFO, "Running operation {} on unit {}", op, self.unit);

                let op_ev_handler = self.ev_handler.get_op_handler(self.unit.clone(), op);

                let operation_result = match self.executor_state {
                    ExecutorState::Finished => {
                        panic!("Can't run operation {}:{}, executor has already exited!", self.unit, op)
                    },


                    ExecutorState::Running(ref mut executor) => {
                        op_ev_handler.handle(OpEvent::Started)?;
                        executor.run_op(op, op_ev_handler.clone()).await
                            .context(format!("Operation {} on unit {} failed!", op, self.unit))
                    }
                };

                match operation_result {
                    Ok(op_completion) => {
                        entry.insert(op_completion.clone());
                        op_ev_handler.handle(OpEvent::Complete(op_completion.clone()))?;
                        Ok(op_completion)
                    },
                    Err(e) => {
                        op_ev_handler.handle(OpEvent::Error(e.to_string()))?;
                        self.finalize().await?;
                        return Err(e)
                    }
                }
            }
        }
    }

    #[instrument]
    pub async fn get_meta(&mut self) -> Result<Arc<Meta>> {
        match self.run_op(Operation::Meta).await? {
            OpCompletion::Meta(status, meta) => {
                status.expect_ok().context(format!("{}:{} operation failed", self.unit, Operation::Meta))?;
                Ok(meta)
            },
            _ => panic!("Expected Meta operation to return Meta result"),
        }
    }

    #[instrument]
    pub async fn get_deps(&mut self) -> Result<Arc<Vec<Dependency>>> {
        match self.run_op(Operation::Deps).await? {
            OpCompletion::Deps(status, deps) => {
                status.expect_ok().context(format!("{}:{} operation failed", self.unit, Operation::Deps))?;
                Ok(deps)
            },
            _ => panic!("Expected Deps operation to return Deps result"),
        }
    }

    #[instrument]
    pub async fn check(&mut self) -> Result<bool> {
        match self.run_op(Operation::Check).await? {
            OpCompletion::Check(status, check_presence, emit_data) => {
                status.expect_ok().context(format!("{}:{} operation failed", self.unit, Operation::Check))?;
                self.add_emit_data(emit_data);
                return Ok(check_presence);
            },
            _ => panic!("Expected Check operation to return Check result"),
        }
    }

    pub async fn apply(&mut self) -> Result<()> {
        match self.run_op(Operation::Apply).await? {
            OpCompletion::Apply(status, emit_data) => {
                status.expect_ok().context(format!("{}:{} operation failed", self.unit, Operation::Apply))?;
                self.add_emit_data(emit_data);
                Ok(())
            },
            _ => panic!("Expected Apply operation to return Apply result"),
        }
    }

    pub async fn remove(&mut self) -> Result<()> {
        match self.run_op(Operation::Remove).await? {
            OpCompletion::Remove(status, emit_data) => {
                status.expect_ok().context(format!("{}:{} operation failed", self.unit, Operation::Remove))?;
                self.add_emit_data(emit_data);
                Ok(())
            },
            _ => panic!("Expected Remove operation to return Remove result"),
        }
    }

    pub async fn set_args(&mut self, args: ValueSet) -> Result<()> {
        match self.executor_state {
            ExecutorState::Running(ref mut executor) => {
                executor.set_args(args).await?;
                Ok(())
            },
            ExecutorState::Finished => {
                panic!("Can't set args on unit {}, executor has already exited!", self.unit);
            }
        }
    }

    pub async fn get_emit_data(&mut self) -> Result<Arc<ValueSet>> {
        Ok(self.emit_data.clone())
    }

    /// Finalizes the executor, retaining any state that may be needed by other units
    pub async fn finalize(&mut self) -> Result<()> {
        // Take the executor out and finalize it outside the match statement
        match std::mem::replace(&mut self.executor_state, ExecutorState::Finished) {
            ExecutorState::Running(executor) => executor.finalize().await?,
            ExecutorState::Finished =>  (),
        };
        Ok(())
    }

    fn add_emit_data(&mut self, emit_data: Arc<ValueSet>) {
        let existing_emit_data = Arc::make_mut(&mut self.emit_data);
        existing_emit_data.merge(emit_data.as_ref());
    }
}

#[derive(Debug)]
enum ExecutorState {
    Running(ShellExecutor),
    Finished,
}
