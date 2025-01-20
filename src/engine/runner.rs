//! Top-level interface for running units

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};

use crate::models::{Unit, UnitArc, Dependency, Meta, ValueSet};
use super::unit_execution::UnitExecution;
use super::Context as EngineContext;

use tracing::instrument;

use super::{
    loader::Loader,
    resolver::DependencyFetcher,
};

type ExecutionArc = Arc<Mutex<UnitExecution>>;
type ExecutionMap = HashMap<UnitArc, ExecutionArc>;

/// Provides an atomic reference counted, interior mutable interface for managing the
/// execution of units.  It contains state of individual unit executions behind a mutex
/// internally, so it can be called from async tasks safely.  Individual unit unit_executions
/// are behind their own mutexes as well, so running an operation on one unit should not block
/// another.
pub struct Runner {
    unit_executions: Arc<Mutex<ExecutionMap>>,
    loader: Loader,
    ctx: EngineContext,
}

/// Handles repetitve mutex boilerplate for the many functions that
/// need to lock unit executions below
macro_rules! run_ex {
    ($self:ident, $unit:ident, $op:ident) => {
        {
            let ex_arc = $self.get_unit_execution($unit.clone())
                .await
                .map_err(|e| e.context(format!("Failed to run unit {}", $unit.name)))
                ?;
            let mut unit_execution = ex_arc.lock().unwrap();
            unit_execution.$op().await
        }
    }
}

impl Runner {
    pub fn new(loader: Loader, ctx: EngineContext) -> Runner {
        Runner {
            ctx,
            loader,
            unit_executions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_deps(&self, unit: UnitArc) -> Result<Arc<Vec<Dependency>>> {
        run_ex!(self, unit, get_deps)
    }

    pub async fn check(&self, unit: UnitArc) -> Result<bool> {
        run_ex!(self, unit, check)
    }

    pub async fn apply(&self, unit: UnitArc) -> Result<()> {
        run_ex!(self, unit, apply)
    }

    pub async fn remove(&self, unit: UnitArc) -> Result<()> {
        run_ex!(self, unit, remove)
    }

    pub async fn finalize(&self, unit: UnitArc) -> Result<()> {
        run_ex!(self, unit, finalize)
    }

    async fn get_emit_data(&self, unit: UnitArc) -> Result<Arc<ValueSet>> {
        run_ex!(self, unit, get_emit_data)
    }

    async fn get_unit_execution(&self, unit: UnitArc) -> Result<ExecutionArc> {
        // Return the existing unit execution if it's already present in the unit executions map
        {
            let existing_units = self.unit_executions.lock().unwrap();
            if let Some(executor) = existing_units.get(&unit) {
                return Ok(executor.clone());
            }
        }

        // If it's not present yet, create a new unit execution and add it to the map
        let script = self.loader.load(&unit.name).await?;
        let mut execution = UnitExecution::init(unit.clone(), self.ctx.clone(), &script).await?;
        let meta = execution.get_meta().await?;
        let args = self.build_args_for(unit.clone(), meta).await?;
        execution.set_args(args).await?;

        let arc = Arc::new(Mutex::new(execution));

        let mut unit_executions = self.unit_executions.lock().unwrap();
        unit_executions.insert(unit.clone(), arc.clone());

        Ok(arc)
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
