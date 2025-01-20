//! Logic for setting up adatpers for unit execution
use std::collections::HashMap;
use anyhow::{Result, anyhow};

use super::subprocess::Command;
use crate::engine::Opts as EngineOpts;

use crate::models::Target;

/// Builds the command that will run a unit based on the adapter configured
/// for its target
pub fn build_command(target: &Target, opts: &EngineOpts) -> Result<Command> {
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
    } else if target.proto == "local" {
        if target.host != "localhost" {
            return Err(anyhow!("Local target must have host 'localhost'"))
        }
        return Ok(Command {
            cmd: "/bin/sh".into(),
            args: Vec::new(),
            env: HashMap::new(),
        })
    } else if target.proto == "podman" {
        let mut args = vec!["exec".into(), "-i".into()];
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
