# Sysunit

Minimal state management tool for POSIX systems

### Rationale

Heavyweight state management tools like Ansible are great for sophisticated environments, but are unwieldy
for simple use-cases.

Shell scripts are great for applying small changes to POSIX systems, but can lack idempotency,
composability, and robustness.

Sysunit provides an intermediate solution, with a minimal learning curve for those accustomed to shell
scripting.  I've found it effective for automating development environments, configuring servers and
IoT devices, setting up more complex containers than Dockerfiles are suitable for,
and occasionally as a build tool.

##### Why Shell?

It's imperative, everyone already knows it, and it's installed everywhere. Sysunit mitigates its most
prominent weaknesses.

### Features

- Units of change are idempotent
- Dependency resolution and cycle detection
- Typed parameters and return values for units  
- Built with async Rust. Fast, robust and suitable for resource-constrained environments.

### Usage

```sh
export SYSUNIT_PATH=~/units:/etc/units #default: ./units

# Apply a unit of change and its dependencies to the system
sysunit apply dev_environment.sh

# Rollback a unit of change and its dependencies from the system
sysunit remove dev_environment.sh --recursive
```

### Installation

Sysunit is currently oriented towards the Rust community and is only distributed via Cargo.

```sh
cargo install sysunit
```

### Units

A unit is a single idempotent state change to a UNIX system, expressed as a shell script which
provides hooks for the various stages of execution.

```sh
# foo_file.sh

check() [ -f /etc/foo.conf ] && emit_ok;
apply() touch /etc/foo.conf;
remove() rm /etc/foo.conf;
```

```sh
sysunit apply foo_file.sh
```

They can expose metadata, have dependencies, receive arguments (optionally required),
provide arguments to their dependents, receive emitted values from their dependencies,
and emit values to their dependents.

Here's an example of a unit using these features:

```sh
#cat.sh

meta() {
    version 1.5.2
    author "Jack Forrest"
    params "!filename:string, x_dim:int, y_dim:int"
}

deps() {
    dep 'pkg.sh name="python"'
    dep 'dir.sh path="./tmp/"
    # curl.sh emits the path to the installed curl binary, which we require
    # and capture here.
    dep 'curl.sh -> !binary_path:curl_binary_path:string'
}

check() [ -f ./tmp/$filename] && emit_ok;

apply() {
  $curl_binary_path "placekitten.com/${x_dim:?200}/${y_dim:?200}" > ./tmp/$filename
  if python ./analyze_kitty.py ./tmp/$filename; then
    emit is_cute true
  else
    # kitty so uggo it crashed our python script :(
    echo "analyze_kitty.py failed!"; # provide error message on unit failure
    exit 1; # fail the unit
  fi
}

remove() rm ./tmp/$filename;
```

```sh
sysunit apply cat.sh -a filename=kitty.jpg -a x_dim=200 -a y_dim=200
```

### Dependencies

Units are identified uniquely by their name, and arguments.  When a unit has multiple dependants,
Sysunit's resolver ensures that the dependency is run only once.  Values it emits are stored and
supplied to each dependent.  Circular dependencies are not permitted.

### Unit Execution

## License

This project is licensed under the GNU Affero General Public License (AGPL) v3.0. See the [LICENSE](LICENSE) file for details.
