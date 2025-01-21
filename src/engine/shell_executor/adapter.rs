//! Logic for setting up adatpers for unit execution
use std::collections::HashMap;
use anyhow::{Result, anyhow};

use super::subprocess::Command;
use crate::engine::Opts as EngineOpts;

use crate::models::{Target, UnitArc};

const SHELL_OPTS: [&str; 3] = ["-e", "-x", "-u"];

/// Builds the command that will run a unit based on the adapter configured
/// for its target
pub fn build_command(unit: UnitArc, opts: &EngineOpts) -> Result<Command> {
    match unit.target {
        Some(ref target) => {
            get_adapter(opts, target)
        },
        None => Ok(Command {
            cmd: "/bin/sh".into(),
            args: SHELL_OPTS.iter().map(|s| s.to_string()).collect(),
            env: HashMap::new(),
        })
    }
}

fn get_adapter(opts: &EngineOpts, target: &Target) -> Result<Command> {
    if let Some(adapter_cmd) = opts.adapters.get(&target.proto) {
        Ok(Command {
            cmd: adapter_cmd.into(),
            args: vec![target.user_host_string()],
            env: HashMap::new(),
        })
    } else if target.proto == "ssh" {
        return Ok(Command {
            cmd: "ssh".into(),
            args: vec![target.user_host_string()],
            env: HashMap::new(),
        })
    } else if target.proto == "podman" {
        let mut args = vec!["exec".into(), "-it".into()];
        if let Some(user) = &target.user {
            args.push("--user".into());
            args.push(user.clone())
        }
        args.push(target.host.clone());
        args.push("/bin/sh".into());

        return Ok(Command {
            cmd: "podman".into(),
            args,
            env: HashMap::new()
        })
    } else {
        Err(anyhow!("No adapter found for target: {:?}", target))
    }
}
