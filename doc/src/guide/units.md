# Units

A Unit is a shell script which invokes various hooks that Sysunit will call
throughout a run. These are, in order of their execution:

- **meta**: Provides metadata about the unit, such as its name, description, and
    parameters that it accepts.
- **deps**: Lists the units that this unit depends on.
- **check**: Determines if the unit needs to be applied or removed
- **apply**: Applies the unit to the system
- **remove**: Removes the unit from the system

Let's start with a basic example and go from there. Here's a simple unit that
puts the current time into a file.

```sh
# foo_file.sh

apply() {
    echo "Writing the current date to /tmp/foo"
    date > /tmp/foo;
}
```

Running `sysunit apply foo_file.sh` will run the `apply` hook, which will write
the current date to `/tmp/foo`. 

Note that the output from your unit is captured when doing this, and is only
displayed if the unit fails to help you diagnose issues.

You can increase the verbosity of Sysunit with the `--verbose` flag to always
include this output, which can be helpful for monitoring the progress of
units that will take a long time to execute. Just like in a shell script,
output from commands executed in your unit will also be included, so if
you invoke a package manager for example, you can see what it's doing
while your unit executes with this method.

##### Note

Sysunit is only intended to handle printable characters! Outputting arbitrary
binary can cause issues with its handling of output, and may cause your unit to
fail.

See [How It Works](how_it_works.md) for more information on why this is the case.

## Idempotency

The unit above is not idempotent, so Sysunit will run it every time you call
`sysunit apply foo_file.sh`. To make it idempotent, you can add a `check` hook.
Let's also provide a hook so sysunit can remove the file.

```sh
# foo_file.sh

check() {
    if [ -f /tmp/foo ]; then
        echo "Found /tmp/foo"
        present
    else
        echo "Did not find /tmp/foo"
    fi
}

apply() {
    echo "Writing the current date to /tmp/foo"
    date > /tmp/foo;
}

remove() {
    echo "Removing /tmp/foo"
    rm /tmp/foo;
}
```

Now when you run `sysunit apply foo_file.sh`, Sysunit will check if `/tmp/foo`
exists, and if it does, it will not run the `apply` hook. If you run `sysunit
remove foo_file.sh`, it will remove `/tmp/foo`, only if it already exists.

## Handling Errors

Sysunit runs unit hooks in a shell with `set -e -u`, meaning that any non-zero
exit status from your commands will cause the unit to fail, and any output to
be printed to aid with diagnosis of the issue. Let's look at a unit that will
fail:

```sh
# fail.sh

apply() {
    echo "I have chosen not to comply"
    false
}
```

On running this, you should see output to the terminal telling you which unit
failed, and include its output.

If the output is not enough to go off of, you can invoke sysunit with
`--debug` which will include trace output from your script (this is just `set
-x` in the subshell that your unit runs in.)
