//! Spawns and manages executors that run scripts on targets

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::models::Target;

use anyhow::Result;

use super::Context as EngineContext;
use super::shell_executor::ShellExecutor;

pub type ExecutorArc = Arc<Mutex<ShellExecutor>>;
pub struct ExecutorPool {
    executors: HashMap<Target, ExecutorArc>,
}

impl ExecutorPool {
    pub fn new() -> ExecutorPool {
        ExecutorPool {
            executors: HashMap::new(),
        }
    }

    pub async fn get_executor(&mut self, target: &Target, ctx: EngineContext) -> Result<ExecutorArc> {
        if let Some(executor) = self.executors.get(target) {
            return Ok(executor.clone());
        };

        let executor = Arc::new(Mutex::new(ShellExecutor::init(target, ctx).await?));
        self.executors.insert(target.clone(), executor.clone());
        Ok(executor)
    }

    pub async fn finalize(&mut self) -> Result<()> {
        let executors = std::mem::take(&mut self.executors);
        
        for (_, executor) in executors.into_iter() {
            // Attempt to own the Arc if possible (no other strong references)
            match Arc::try_unwrap(executor) {
                Ok(mutex) => {
                    // Now get the inner executor
                    let executor = mutex.into_inner().unwrap();
                    executor.finalize().await?;
                }
                Err(_) => {
                    panic!("Executor can't be finalized: references still held");
                }
            }
        }

        Ok(())
    }
}
