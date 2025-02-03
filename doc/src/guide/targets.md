# Targets

So far, we've been running units against our local system, which is great for building projects
or setting up development environments. However, Sysunit is also capable of targeting other 
systems via adapters. This capability can be used to do things like:

- Set up remote servers
- Configure and provision SoC devices
- Build Docker or Podman containers
- Automate deployments

### Setting target via the CLI

When invoking a unit via the CLI, we can set the target explicitly with the `--target` flag.

```sh
sysunit apply foo_file.sh --target ssh://admin@my_server.net
```

The target is given as a URI, and the protocol portion is used to determine which adapter to use.
Out of the box, Sysunit provides the following adapters:

- `ssh`: Connects to a remote system via SSH
- `docker`: Runs the unit in a Docker container
- `podman`: Runs the unit in a Podman container

### Setting targets for dependencies

For more sophisticated workflows, it's desirable to have various units target different systems.
This can be accomplished by setting the target for dependencies.

```sh
# dev_env.sh

deps() {
    dep local://localhost:pkg.sh name=ssh
    dep ssh://admin@my_server.net:pkg.sh name=python3
    dep ssh://admin@my_server.net:pkg.sh name=tmux
}
```

When we run this with `sysunit apply dev_env.sh`, Sysunit will use the `deps`
hook to determine that we first need to install the SSH package on the local
system, and then install Python3 and tmux on our remote server.

Note that for adapters other than `local`, external binaries are invoked, so
ssh, Docker, or Podman may need to be installed on the host machine.

As with the local adapter, only one instance of an adapter per target is initialized
per Sysunit invocation. This means that if you have multiple units targeting the same
system, they will be run in the same session.

### Target Inheritance

When a unit is run with a target, that target is inherited by its dependencies, so units
don't need to explicitly set targets for their dependencies unless they need to target
another system.

### Dynamic Targets

As with everything else in dependencies, target strings are scriptable, so you can
do things like dynamic inventory using parameters.

```sh
install_nethack.sh

meta() {
    params !user:string, !host:string
}

deps() {
    dep ssh://$user@$host:pkg.sh name=nethack
}
```
