//! Top-level interface for running units

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};

use crate::models::{Operation, OpCompletion, Unit, UnitArc, Dependency, Meta, ValueSet, Target};
use super::unit_execution::UnitExecution;
use super::Context as EngineContext;
use super::shell_executor::{ShellExecutor, ExecutorArc};

use tracing::instrument;

use super::{
    loader::Loader,
    resolver::DependencyFetcher,
};

pub struct Runner {
    ctx: EngineContext,
    loader: Loader,
    unit_executions: HashMap<UnitArc, UnitExecution>,
    executors: HashMap<Target, ExecutorArc>,
}

impl Runner {
    pub fn new(loader: Loader, ctx: EngineContext) -> Runner {
        Runner {
            ctx,
            loader,
            unit_executions: HashMap::new(),
            executors: HashMap::new(),
        }
    }

    pub async fn get_meta(&mut self, unit: UnitArc) -> Result<Arc<Meta>> {
        let mut executor = self.get_executor(unit.clone()).await?;
        let result = executor.run_op(&unit.name, &ValueSet::new(), Operation::Meta).await?;
        match result {
            OpCompletion::Meta(status, meta) => {
                status.expect_ok()?;
                Ok(meta)
            },
            _ => panic!("Expected Meta operation to return Meta result"),
        }
    }

    pub async fn get_deps(&self, unit: UnitArc) -> Result<Result<Arc<Vec<Dependency>>>> {
        match self.run_op(unit, Operation::Deps).await? {
            OpCompletion::Deps(status, deps) => {
                status.expect_ok()?;
                execution.set_deps(deps);
                Ok(())
            },
            _ => panic!("Expected Deps operation to return Deps result"),
        }
    }

    pub async fn remove(&mut self, unit: UnitArc) -> Result<()> {
        let mut execution = self.get_unit_execution(unit.clone()).await?;
        let mut executor = self.get_executor(unit.clone()).await?;
        let result = executor.run_op(&execution.script, &execution.args, Operation::Remove).await?;
        match result {
            OpCompletion::Remove(status, emit_data) => {
                status.expect_ok()?;
                execution.add_emit_data(emit_data);
                Ok(())
            },
            _ => panic!("Expected Remove operation to return Remove result"),
        }
    }

    pub async fn apply(&mut self, unit: UnitArc) -> Result<()> {
        let mut execution = self.get_unit_execution(unit.clone()).await?;
        let mut executor = self.get_executor(unit.clone()).await?;
        let result = executor.run_op(&execution.script, &execution.args, Operation::Apply).await?;
        match result {
            OpCompletion::Apply(status, emit_data) => {
                status.expect_ok()?;
                execution.add_emit_data(emit_data);
                Ok(())
            },
            _ => panic!("Expected Apply operation to return Apply result"),
        }
    }

    pub async fn check(&mut self, unit: UnitArc) -> Result<bool> {
        match self.run_op(unit, Operation::Check).await? {
            OpCompletion::Check(status, check_presence, emit_data) => {
                status.expect_ok()?;
                execution.add_emit_data(emit_data);
                Ok(check_presence)
            },
            _ => panic!("Expected Check operation to return Check result"),
        }
    }

    pub async fn run_op(&mut self, unit: UnitArc, op: Operation) -> Result<OpCompletion> {
        let mut execution = self.get_unit_execution(unit.clone()).await?;
        let mut executor = self.get_executor(unit.clone()).await?;
        let result = executor.run_op(&execution.script, &execution.args, op).await?;
        result.expect_ok()?;
        Ok(result)
    }

    async fn get_unit_execution(&mut self, unit: UnitArc) -> &mut UnitExecution {
        match self.unit_executions.get_mut(&unit) {
            Some(execution) => Ok(execution),
            None => {
                panic!("Unit not initialized: {:?}", unit);
            }
        }
    }

    async fn get_executor(&mut self, unit: UnitArc) -> Result<ExecutorArc> {
        let default_target = Some(Target::default());
        let target = unit.target.as_ref().or(default_target.as_ref()).unwrap();
        match self.executors.get(&target).cloned() {
            Some(exarc) => Ok(exarc),
            None => {
                let exarc = Arc::new(Mutex::new(ShellExecutor::init(&target, self.ctx.clone()).await?));
                self.executors.insert(target.clone(), exarc.clone());
                Ok(exarc)
            }
        }
    }

    // A unit first needs to have arguments injected.  Then before running check/apply, it
    // will have emit values injected.
    //
    // Without these args being injected in phases, the unit would need for its dependencies
    // to have executed... before the deps operation.  This is recursive.
    async fn build_args_for(&self, unit: UnitArc, meta: Arc<Meta>) -> Result<ValueSet> {
        // Check that all required parameters have corresponding arguments

        for param in meta.params.iter() {
            if param.required && !unit.args.values.contains_key(param.name.as_str()) {
                return Err(anyhow!("Missing required parameter: {}", param.name.as_str()));
            }
        }

        for (key, value) in unit.args.values.iter() {
            let param = meta.params.iter().find(|param| param.name == *key);
            match param {
                Some(param) if value.get_type() != param.value_type => {
                    return Err(anyhow!("Argument {} is of type, {} not {} as expected", key, value.get_type(), param.value_type));
                },
                None => return Err(anyhow!("Parameter {} is provided, but not accepted", key)),
                _ => continue,
            };
        }
        Ok(unit.args.clone())
    }

    /// Assembles the arguments that should be passed to a unit which is to be executed.
    /// These include the arguments passed to the unit itself, as well as any captures
    /// from dependencies.
    #[instrument(skip(self))]
    pub async fn set_captures(&self, unit: UnitArc) -> Result<()> { 
        let execution_arc = self.get_unit_execution(unit.clone()).await?;
        let mut execution = execution_arc.lock().unwrap();
        let deps = execution.get_deps().await?;

        let mut captures = ValueSet::new();

        // Check that all captures are present and of the correct type and add them to the args
        for dep in deps.iter() {
            let dep_unit: UnitArc = Arc::new(Unit::new(dep.name.clone(), dep.args.clone(), dep.target.clone()));

            // Unit must already be running at this point.
            let emitted_values = self.get_emit_data(dep_unit.clone()).await?;
            for capture in dep.captures.iter() {
                let value = emitted_values
                    .get(&capture.name)
                    .ok_or_else(|| anyhow!("Capture could not be satisfied: {}:{}", dep_unit, capture.name))?;

                // Default the alias to the capture's name if not specified
                let alias = match &capture.alias {
                    Some(a) => a.clone(),
                    None => capture.name.clone(),
                };

                if value.get_type() != capture.value_type {
                    return Err(anyhow!(
                        "Capture {:?} from {:?} is of type {:?} not {:?} as expected",
                        capture.name,
                        dep_unit.name,
                        value.get_type(),
                        capture.value_type
                    ));
                }

                captures.add_value(&alias, value.clone());
            }
        }

        execution.set_args(captures.clone()).await?;
        Ok(())
    }
}

impl DependencyFetcher<UnitArc> for Runner {
    async fn get_node_dependencies(&self, unit: UnitArc) -> Result<Vec<UnitArc>> {
        let deps = self.get_deps(unit.clone()).await?;
        let units = deps
            .iter()
            .map(|dep| {
                let target = match dep.target {
                    Some(ref target) => Some(target.clone()),
                    None => unit.target.clone(),
                };
                let unit = Unit::new(dep.name.clone(), dep.args.clone(), target.clone());
                Arc::new(unit)
            })
            .collect();
        Ok(units)
    }
}
