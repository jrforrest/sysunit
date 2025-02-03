//! Contains logic for running operations on units and managing their execution state
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow, Context};

use crate::models::{Operation, Unit, UnitArc, Dependencies, Meta, ValueSet};
use crate::events::{OpEvent, OpEventHandler};
use super::unit_execution::UnitExecution;
use super::Context as EngineContext;
use super::executor_pool::ExecutorPool;
use super::transport::transport_file;

use super::{
    loader::Loader,
    resolver::DependencyFetcher,
};

pub struct Runner {
    ctx: EngineContext,
    loader: Loader,
    executor_pool: ExecutorPool,
    unit_executions: HashMap<UnitArc, UnitExecution>,
}

impl Runner {
    pub fn new(loader: Loader, ctx: EngineContext) -> Runner {
        Runner {
            ctx,
            loader,
            executor_pool: ExecutorPool::new(),
            unit_executions: HashMap::new(),
        }
    }
    
    pub async fn get_deps(&mut self, unit: UnitArc) -> Result<&Dependencies> {
        let op_ev_handler = self.ctx.ev_handler.get_op_handler(unit.clone(), Operation::Meta);
        let executor_arc = self.executor_pool.get_executor(&unit.target, self.ctx.clone()).await?;
        let execution = self.get_unit_execution(unit.clone()).await;
        execution.get_deps(executor_arc, op_ev_handler.clone()).await
            .map_err(|e| {
                op_ev_handler.handle(OpEvent::Error(e.to_string())).unwrap();
                anyhow!("Failed to get deps for unit {} on target {}", &unit.name, &unit.target)
            })
    }
    
    pub async fn remove(&mut self, unit: UnitArc) -> Result<()> {
        let op_ev_handler = self.ctx.ev_handler.get_op_handler(unit.clone(), Operation::Remove);
        let executor_arc = self.executor_pool.get_executor(&unit.target, self.ctx.clone()).await?;
        let execution = self.get_unit_execution(unit.clone()).await;
        execution.remove(executor_arc, op_ev_handler.clone()).await
            .map_err(|e| {
                op_ev_handler.handle(OpEvent::Error(e.to_string())).unwrap();
                anyhow!("Failed to remove unit {} on target {}", &unit.name, &unit.target)
            })
    }
    
    pub async fn apply(&mut self, unit: UnitArc) -> Result<()> {
        let op_ev_handler = self.ctx.ev_handler.get_op_handler(unit.clone(), Operation::Apply);
        let executor_arc = self.executor_pool.get_executor(&unit.target, self.ctx.clone()).await?;
        let execution = self.get_unit_execution(unit.clone()).await;
        execution.apply(executor_arc, op_ev_handler.clone()).await
            .map_err(|e| {
                op_ev_handler.handle(OpEvent::Error(e.to_string())).unwrap();
                anyhow!("Failed to apply unit {} on target {}", &unit.name, &unit.target)
            })
    }

    pub async fn check(&mut self, unit: UnitArc) -> Result<bool> {
        let op_ev_handler = self.ctx.ev_handler.get_op_handler(unit.clone(), Operation::Check);
        let deps = {
            let deps = self.get_deps(unit.clone()).await?;
            deps.clone()
        };
        let captures = self.get_captures(unit.clone(), &deps).await?;
        op_ev_handler.handle(OpEvent::Started).unwrap();
        self.transport_files(&unit, &deps, op_ev_handler.clone()).await?;
        let executor_arc = self.executor_pool.get_executor(&unit.target, self.ctx.clone()).await?;
        let execution = self.get_unit_execution(unit.clone()).await;
        execution.set_args(&captures).await;
        execution.check(executor_arc, op_ev_handler.clone()).await
            .map_err(|e| {
                op_ev_handler.handle(OpEvent::Error(e.to_string())).unwrap();
                anyhow!("Failed to check unit {} on target {}", &unit.name, &unit.target)
            })
    }

    async fn get_unit_execution(&mut self, unit: UnitArc) -> &mut UnitExecution {
        match self.unit_executions.get_mut(&unit) {
            Some(execution) => execution,
            None => {
                panic!("Unit not initialized: {:?}", unit);
            }
        }
    }

    async fn transport_files(&mut self, unit: &UnitArc, deps: &Dependencies, op_ev_handler: OpEventHandler) -> Result<()> {
        for file in deps.files.iter() {
            op_ev_handler.handle(OpEvent::TransportingFile(file.clone())).unwrap();
            transport_file(&file, &unit.target).await?;
            op_ev_handler.handle(OpEvent::FileTransported(file.clone())).unwrap();
        }

        Ok(())
    }

    // A unit first needs to have arguments injected.  Then before running check/apply, it
    // will have emit values injected.
    //
    // Without these args being injected in phases, the unit would need for its dependencies
    // to have executed... before the deps operation.  This is recursive.
    async fn build_args_for(&self, unit: UnitArc, meta: &Meta) -> Result<ValueSet> {
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

    pub async fn finalize(&mut self) -> Result<()> {
        self.executor_pool.finalize().await
    }

    /// Assembles the arguments that should be passed to a unit which is to be executed.
    /// These include the arguments passed to the unit itself, as well as any captures
    /// from dependencies.
    pub async fn get_captures(&self, unit: UnitArc, deps: &Dependencies) -> Result<ValueSet> { 
        let mut captures = ValueSet::new();

        // Check that all captures are present and of the correct type and add them to the args
        for dep in deps.units.iter() {
            let target = match dep.target {
                Some(ref target) => target.clone(),
                None => unit.target.clone(),
            };
            let dep_unit: UnitArc = Arc::new(Unit::new(dep.name.clone(), dep.args.clone(), target));

            // Assume the unit has already been run and has emit values
            let emitted_values = match self.unit_executions.get(&dep_unit).map(|execution| &execution.emit_data) {
                Some(data) => data,
                None => panic!("Can't get emit values, unit not initialized: {:?}", unit),
            };

            for capture in dep.captures.iter() {
                let value = emitted_values
                    .get(&capture.name)
                    .ok_or_else(|| {
                        anyhow!("Unit {} expected its dependency {} to emit a value for capture: {}", unit.name, dep_unit.name, capture.name)
                    })?;

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

        Ok(captures)
    }

    /// Initializes a unit, running its meta and deps operations
    async fn load_unit(&mut self, unit: UnitArc) -> Result<&UnitExecution> {
        let script = self.loader.load(&unit.name).await?;
        let mut execution = UnitExecution::new(script).await?;
        let executor_arc = self.executor_pool.get_executor(&unit.target, self.ctx.clone()).await?;
        
        // Run the units meta operation to get its metadata
        let meta = {
            let op_ev_handler = self.ctx.ev_handler.get_op_handler(unit.clone(), Operation::Meta);
            execution.get_meta(executor_arc.clone(), op_ev_handler.clone()).await
                .map_err(|e| {
                    op_ev_handler.handle(OpEvent::Error(e.to_string())).unwrap();
                    anyhow!("Failed to get meta for unit {} on target {}", &unit.name, &unit.target)
                })?
        };

        // Sets the arguments given for the unit on its execution so they can be used for
        // following operations
        let args = self.build_args_for(unit.clone(), meta).await?;
        execution.set_args(&args).await;

        // Run the units deps operation to get its dependencies
        let op_ev_handler = self.ctx.ev_handler.get_op_handler(unit.clone(), Operation::Deps);
        execution.get_deps(executor_arc, op_ev_handler).await?;

        self.unit_executions.insert(unit.clone(), execution);
        Ok(self.unit_executions.get(&unit).unwrap())
    }
}

impl DependencyFetcher<UnitArc> for Runner {
    async fn get_node_dependencies(&mut self, unit: UnitArc) -> Result<Vec<UnitArc>> {
        let execution = if let Some(execution) = self.unit_executions.get(&unit) {
            execution
        } else {
            self.load_unit(unit.clone()).await
                .context(format!("Failed to load unit {}", unit.name))?
        };

        let units = execution.deps.as_ref().unwrap().units
            .iter()
            .map(|dep| {
                let target = match dep.target {
                    Some(ref target) => target.clone(),
                    None => unit.target.clone(),
                };
                let unit = Unit::new(dep.name.clone(), dep.args.clone(), target);
                Arc::new(unit)
            })
            .collect();
        Ok(units)
    }
}
