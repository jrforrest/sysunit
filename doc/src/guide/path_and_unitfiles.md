# Path and Unitfiles

Sysunit will look for units in the current working directory by default, but provides
more sophisticated lookup options enabling a great deal of flexibility in how you lay
out your configuration.

#### Note

Units are always loaded from the host system, even when using adapters. They are
streamed to target systems, but never copied.

## Search Paths

Directories of units can be added to the search path with the `--path` or `-p` flags,
or via the `SYSUNIT_PATH` environment variable. These units can then be referenced
from other units with no path prefix.

For example, if you have `/etc/units/foo.sh` and `/etc/units/bar.sh`, foo may
reference bar with `dep bar.sh`

## Unit Files

Individual files can be a pain when you have many small units, so Sysunit has the
capability of parsing multiple units from a single file using a simple comment
header format to delimit them.

```sh

# The header below indicates to sysunit that we're starting a new unit

# [ foo_file.sh ]
deps() {
    dep ./dir.sh path=/tmp
}

apply() touch /tmp/foo;

# [ dir.sh ]

# We are now inside of the dir.sh unit

check() {
    if [ -d $path ]; then
        present
    fi
}

apply() mkdir -p $path;
```

## Relative Paths

Relative paths in units are resovled relative to the directory the unit resides in. It's
probably best-practice to mostly use relative paths, so you can move your units around
and disambiguate if you directories containing unrelated units to your search path.

When using relative paths, Unitfiles act just like a directory. If `build_units.sysu` is in
your search path and contains a unit named `foo.sh`, youc an reference it from another unit
with `build_units.sysu/foo.sh`. The `.sh` extension is optional, but recommended for the sake
of clarity.

## Layout Recommendations

I personally maintain common units in a mixture of `/etc/units` for system-wide
config, `~/.local/etc/units` for my user-specific stuff, `~/src/build` for
units I use for a mixture of projects, and then `./units` or `./Unitfile.sysu`
for those specific to a certain project.

I have `export SYSUNIT_PATH="/etc/units:~/.local/etc/units:~/src/build"` in my
`.bashrc` to it easy to invoke any of the common ones from anywhere. 

When I'm working in a project, I often invoke sysunit with `-p ./units` to include
those specific to my project.
