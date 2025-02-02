Sysunit is a minimal state management tool for POSIX systems. It's useful for
thing like manaing your development environment, configuring servers, building code,
provisioning containers, and automating deployments.

Instead of introducing a new language or requiring complex templating logic in
configuration files to define desired state, Sysunit uses composable shell
scripts to define units of change. This makes it easy for anyone familiar with
basic shell programming to pick up, and units can be run against systems as
minimal as busybox environments.

## Features

- **Minimal**: Sysunit is a single binary with no dependencies. It can execute
    units on any system that has a POSIX shell and can be reached over some protocol.
- **Idempotent**: Units are idempotent, so they can be run multiple times without
    changing the system state.
- **Dependencies**: Units can depend on other units, and Sysunit will resolve the
    dependency graph and avoid redundant executions.
- **Arguments**: Units can accept typed parameters, and Sysunit will validate
    their arguments before invocation.
- **Emit values**: Units can emit values, so their dependees can utilize them
    as parameters, and they are also type-checked. 
- **Targets**: Units can target POSIX systems via any executable that can run a shell
    script. This includes SSH and Docker, and I've used it to set up SoC devices
    over USB serial.

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
the [cookbook](cookbook.md) for look at how more sophisticated usage looks.
