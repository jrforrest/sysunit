# Parameters

Units are more useful if when they can be re-used. For example, you might have
a unit that installs some package on the target system:

```sh
# pkg.sh

check() {
    if dpkg -l | grep -q vim-nox; then
        present
    fi
}

apply() {
    apt-get install -y vim-nox
}
```

It would be really troublesome to have to copy this unit for each package we want.
Let's parameterize it so it can install any package we want:

```sh
# pkg.sh

meta() {
    params !name:string
}

check() {
    if dpkg -l | grep -q $name; then
        present
    fi
}

apply() {
    apt-get install -y $name
}
```

Here we've introduced a new hook, `meta`, which is used to tell sysunit about your
unit. The `meta` hook supports the `params` function, which takes a list of
parameter definitions. In this case, we've defined a single parameter, `name`,
which is a string, and is required as denoted by the `!` prefix.

When you run this unit, you'll need to provide a value for `name`, which can be done
via the sysunit CLI since this is the only unit we are invoking for now:
 `sysunit apply pkg.sh --arg name=tmux`.

If it is invoked with in invalid type, sysunit will exit with an error message.

 The types of parameter currently allowed are:
 
- **string**: Any string value
- **int**: A numerical value in base 10 format with no decimal
- **float**: Any numerical value in base 10 format, must include a decimal
- **bool**: Either `true` or `false`

More complex types such as arrays are not currently supported, but a JSON type
to allow structured data to validated by sysunit and handled in scripts that
can make use of them through jq or similar. For the time being, using strings
is recommended.

### Optional Parameters

In the example above, the `$name` parameter is required, so Sysunit guarantees that
the shell parameter will be set when our hooks are run. If we make the parameter
optional, we need to take care in our shell script to handle the case where it
is empty, or it will fail due to the `set -u` environment it runs in.

```sh
# name_file.sh

meta() {
    params name:string
}

apply() {
    # Default to Jack if no other name is provided
    echo ${name:-Jack} > /tmp/my_name
}
```

### Other Metadata

The `meta` hook also allows specifying some additional Metadata which will help
document your units.

```sh
# foo_file.sh

meta() {
    author "Jack Forrest"
    desc "Writes some stuff to /tmp/foo"
    version "0.5.0"
}
```

None of this is required, or used much within sysunit currently, but it's readily
legible and will likely be presented in a more useful form by the Sysunit CLI soon.

### Dynamic Metadata

It is possible to script the meta hook, so you could dynamically define parameters
based on the environment. All hooks for a unit are executed on the target system,
so facts are available here.

However, **this is heavily discouraged**. It will be confusing to yourself and other
users of your units if these things change between invocations.

