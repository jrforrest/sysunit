# Sysunit

![GitHub branch status](https://img.shields.io/github/checks-status/jrforrest/sysunit/master)
![Crates.io license](https://img.shields.io/crates/l/sysunit)
![Crates.io version](https://img.shields.io/crates/v/sysunit)
![Crates.io size](https://img.shields.io/crates/size/sysunit)


Minimal state management tool for POSIX systems.

[Guide](https://jackforrest.me/sysunit)

### Installation

Sysunit is currently oriented towards the Rust community and is only distributed via Cargo.

```sh
cargo install sysunit
```

### Basic Operation

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

## License

This project is licensed under the GNU Affero General Public License (AGPL) v3.0. See the [LICENSE](LICENSE.txt) file for details.
