# Sysunit

Sysunit is a minimal state management tool for POSIX systems. It's useful for
thing like wrangling your development environment, configuring servers,
building code, provisioning containers and IoT devices, and automating
deployments.

Instead of introducing a new language or requiring complex templating logic in
configuration files to define desired state, Sysunit uses composable shell
scripts to define units of change. This makes it easy for anyone familiar with
basic shell programming to pick up, and units can be run against spartan systems
such as devices running busybox or alpine containers.

## Qualities

- **Minimal**: Sysunit is a single binary with no dependencies.
- **Agentless**: Commands are streamed to target systems over anything that
  can intepret shell. This includes SSH, Docker, Podman, and even serial
  connections.
- **Aproachable**: There's only a couple concepts on top of basic shell
  scripting to learn here.
- **Fast**: Sysunit is written in Rust, and runs only one process per target
  system.
- **Idempotent**: Units are idempotent, so they can be run multiple times
  without changing the system state.
- **Composable**: Units can accept parameters, depend on other units, and emit
  values to their dependees, making this tool suitable for fairly complex
  workflows.

## Limitations

- **Sequential**: Sysunit currently only runs one unit at a time. This is a
  only a limitation of the CLI, and the architecture will allow for
  parallelization with alternative interfaces.
- **Stateful**: This isn't Nix, and only provides idempotency checks as a way of
  avoiding the pitfalls of stateful configuration management.
- **Small-scale**: This tool is not intended for managing very large fleets of
  systems. The CLI will become hard to understand with more than a few dozen,
  and the process-per-target model will fall over at some point beyond that.
- **Batteries Not Included**: Sysunit is oriented at folks comfortable with
  POSIX systems currently, so it doesn't have any built-in units. A contrib
  repo of oft-repeated units is likely to emerge with traction.

## Installation

Sysunit is oriented towards the Rust community until it reaches version 1.0, so
it is only available via Cargo:

```sh
cargo install sysunit
```

## Basic Operation
Given this basic unit:

```sh
# foo_file.sh

check() {
    if [ -f /tmp/foo ]; then
        present
    fi
}

apply() touch /tmp/foo;
remove() rm /tmp/foo;
```

`sysunit apply foo_file.sh` will ensure that `/tmp/foo` exists on the local system,
and `sysunit remove foo_file.sh` will remove it.

Read [the guide](guide.md) for approachable information on how to write and compose units, or
the [cookbook](cookbook.md) for a look at more sophisticated usage.
