# Dependencies

Single units can be useful, but the goal of Sysunit is to allow them to be
easily composed to express complex desired system states. This is mostly
accomplished by defining dependencies that your unit requires.

Let's use our package installation unit from the previous section, and invoke
it with various arguments to set up a development environment for writing
Python:

```sh
# dev_env.sh

deps() {
    dep pkg.sh name=git
    dep pkg.sh name=python3
    dep pkg.sh name=tmux
}
```

When we run this with `sysunit apply dev_env.sh`, sysunit will use the `deps`
hook to determine that before `dev_env.sh` can be considered applied, three
instances of the `pkg.sh` unit must be applied first, each with the provided
`name` argument. Each of these dependencies will have their `check` hook run
before applying, and `apply` will only be run if it does not indicate that the
unit is already applied.

Note that this unit does not provide an `apply` hook, so Sysunit will do
nothing by default after the dependencies are satisfied, units like this
are only used as a sort of manifest to define a set of units that should be
applied together.

### Unit Path

By default, Sysunit will look for units in the current working directory, but
you can specify colon-separated paths in which units can be found either with
the `--path` or `p` flag, or by setting `SYSUNIT_PATH` in your environment.

### Dependency Resolution

Unit instances are resolved into a Directed Acyclic Graph (DAG) before
execution. A unit instance is differentiated by a combination of its target
(explained in another section) and its arguments, and an instance is executed
only once per sysunit invocation. Let's see how this works in practice:


```sh
#foo_file.sh

deps() { dep scratch_dir.sh };

check() {
    if [ -f /tmp/scratch/foo ]; then
        present
    fi
}

apply() echo "helloooo" > /tmp/scratch/foo;
```

```sh
#cat_pic.sh

check() {
    if [ -f /tmp/scratch/cat.jpg ]; then
        present
    fi
}

apply() curl -o /tmp/scratch/cat.jpg https://placekitten.com/200/300
```

```sh
#scratch_dir.sh

check() {
    if [ -d /tmp/scratch ]; then
        present
    fi
}

apply() mkdir -p /tmp/scratch
```

```sh
#project_files.sh

deps() {
    dep foo_file.sh
    dep cat_pic.sh
}
```

When we run `sysunit apply project_files.sh`, Sysunit will run the units in this order:

- `scratch_dir.sh`
- `foo_file.sh`
- `cat_pic.sh`
- `project_files.sh`

Note that scratch\_dir is only run once, even though two units include it in their deps.

## Dynamic Dependencies

When we define parameters that our unit can accept, they are injected prior to running the
`deps` hook. This allows us to define dependencies that are based on parameters of our unit:

```sh
#cat_pic.sh

meta() {
    params !url:string
}

deps() {
    if $url | grep -q '^ssh'; then
       dep pkg.sh name=scp 
    else
       dep pkg.sh name=curl
    fi
}

apply() {
    if $url | grep -q '^ssh'; then
        scp $url /tmp/scratch/cat.jpg
    else
        curl -o /tmp/scratch/cat.jpg $url
    fi
}
```

As with all other hooks, the `deps` hook can run any arbitrary shell code, and is executed
on the target system

Unlike Metadata, deps are intended to be dynamic, and that capability is very powerful.
However, some users may not desire the lack of determinism in knowing which dependencies
will be run when they invoke a unit. The Execution Plan output intends to provide clarity
on what dynamic units have been selected, but use of this feature can be avoided for those
who don't desire it.

## Dynamic Parameters

We can also pass our dynamic parameters directly down to our dependencies:

```sh
#foo_file.sh

meta() {
    params !directory:string
}

deps() {
    dep dir.sh directory="$directory"
}

apply() {
    touch $directory/foo
}
```

```sh
#directory.sh

meta() {
    params !directory:string
}

apply() mkdir -p $directory;
```

Running this unit with `sysunit apply foo_file.sh --arg directory=/tmp/` will first
create the `/tmp/` directory, and then create the foo file in it.
