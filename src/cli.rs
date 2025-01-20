//! Provides parsing and validation of command line arguments into
//! options required by the engine

use crate::{
    engine::Opts as EngineOpts,
    models::{Operation, Unit, Value, ValueSet},
    parser::parse_target,
};

use anyhow::{anyhow, Result};
use clap::{Arg, Command};

use async_std::path::PathBuf;

mod reporter;

pub use reporter::{EngineLogger, Verbosity};

pub struct CLI {
    matches: clap::ArgMatches,
        .arg(
            Arg::new("adapter")
                .help("Specify protocol and command in the form of <protocol>=<command>")
                .long("adapter")
                .value_name("PROTOCOL=COMMAND")
                .num_args(1),
        )
}

impl CLI {
    pub fn init() -> Result<CLI> {
        let matches = get_cli_definition().get_matches();
        let cli = CLI { matches };

        Ok(cli)
    }

    fn get_verbosity_level(&self) -> Result<Verbosity> {
        let debug = self.matches.get_flag("debug");
        let verbose = self.matches.get_flag("verbose");
        let quiet = self.matches.get_flag("quiet");

        if (debug as usize + verbose as usize + quiet as usize) > 1 {
            return Err(anyhow!(
                "Only one of --debug, --verbose, or --quiet can be used at a time"
            ));
        }

        if debug {
            Ok(Verbosity::Debug)
        } else if verbose {
            Ok(Verbosity::Verbose)
        } else if quiet {
            Ok(Verbosity::Quiet)
        } else {
            Ok(Verbosity::Default)
        }
    }

    pub fn get_engine_observer(&self) -> Result<EngineLogger> {
        Ok(EngineLogger::new(self.get_verbosity_level()?))
    }

    pub fn get_engine_options(&self) -> Result<EngineOpts> {
        let engine_opts = EngineOpts {
            remove_deps: self.matches.get_flag("remove_deps"),
            operation: self
                .matches
                .get_one::<String>("operation")
                .unwrap()
                .parse::<Operation>()
                .unwrap(),
            search_paths: self.get_search_paths()?,
            unit: self.get_unit()?.into(),
        };

        let operation = engine_opts.operation;
        let remove_deps = engine_opts.remove_deps;

        if !matches!(operation, Operation::Remove) && remove_deps {
            return Err(anyhow!(
                "--remove-deps can only be used with the 'remove' operation"
            ));
        }

        Ok(engine_opts)
    }

    fn get_unit(&self) -> Result<Unit> {
        let matches = &self.matches;
        let unit_name = matches.get_one::<String>("unit_name").unwrap();

        let mut arg_set = ValueSet::new();

        if let Some(occurences) = matches.get_occurrences::<String>("args") {
            for occurence in occurences {
                for value in occurence {
                    let parts: Vec<&str> = value.split('=').collect();
                    if parts.len() != 2 {
                        return Err(anyhow!("Arguments must be in the form of KEY=VALUE"));
                    } else {
                        let arg_value = Value::from_string(parts[1]);
                        arg_set.add_value(parts[0], arg_value);
                    }
                }
            }
        }

        let target = match matches.get_one::<String>("target") {
            Some(t) => Some(parse_target(t)?),
            None => None,
        };

        Ok(Unit::new(unit_name.to_string(), arg_set, target))
    }

    fn get_search_paths(&self) -> Result<Vec<PathBuf>> {
        let path_string = match self.matches.get_one::<String>("path") {
            Some(p) => p.clone(),
            None => match std::env::var("SYSU_PATH") {
                Ok(p) => p.to_string(),
                Err(_) => {
                    return Err(anyhow!(
                        "No path provided and no SYSU_PATH environment variable set"
                    ));
                }
            },
        };

        Ok(path_string.split(':').map(PathBuf::from).collect())
    }
}

fn get_cli_definition() -> Command {
    Command::new("sysu")
        .version("1.0")
        .about("Applies idempotent state-changing shell scripts")
        .arg(
            Arg::new("operation")
                .help("The operation to be applied")
                .required(true)
                .value_parser(["check", "apply", "remove", "meta"])
                .index(1),
        )
        .arg(
            Arg::new("unit_name")
                .help("The unit to be applied")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::new("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue)
                .short('v')
                .long("verbose"),
        )
        .arg(
            Arg::new("target")
                .help("The target to apply the operation to")
                .long("target")
                .short('t')
                .value_name("TARGET")
                .num_args(1),
        )
        .arg(
            Arg::new("quiet")
                .help("Suppress diagnostic output")
                .action(clap::ArgAction::SetTrue)
                .short('q')
                .long("quiet"),
        )
        .arg(
            Arg::new("debug")
                .help("Enable debug output")
                .action(clap::ArgAction::SetTrue)
                .short('d')
                .long("debug"),
        )
        .arg(
            Arg::new("remove_deps")
                .help("Include dependencies when removing a unit.")
                .action(clap::ArgAction::SetTrue)
                .short('r')
                .long("remove-deps"),
        )
        .arg(
            Arg::new("path")
                .help("Colon delimited search paths for units")
                .long("path")
                .short('p')
                .value_name("PATH")
                .num_args(1),
        )
        .arg(
            Arg::new("args")
                .help("Arguments to be passed to the unit")
                .long("arg")
                .short('a')
                .action(clap::ArgAction::Append)
                .value_name("KEY=VALUE")
                .num_args(1),
        )
}
