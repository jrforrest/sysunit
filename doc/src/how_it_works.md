Some of the things Sysunit does will the shell units can seem a bit magical,
but there's only a couple of tricks going on here to get them to work.

## Shell Execution

Sysunit simply pipes your scripts into a shell interpreter, potentially over
some intermediary like SSH or Docker, runs its hooks, and captures the output.

Execution of hooks from units is interleaved, since, for example, dependencies
of a unit need to be known and executed before it is. These hooks are all run
in subshells, so they can't affect the environment of other units.

## Hooks

The shell environment hooks run in is set up with someting like this:

```sh
exec 2>&1

meta() : ;
deps() : ;
check() : ;
apply() : ;
rollback() : ;
dep() _emit dep $@;
author() _emit meta.author $@;
desc() _emit meta.desc $@;
params() _emit meta.params $@;
emits() _ emit meta.emits $@;
present() _emit present true;

emit_value() {
  local key="${1:?key must be provided to emit_value}"
  shift
  _emit "value.${key}" "$@"
}

_emit() {
  local key=${1:?key must be provided to _emit}
  shift
  local val="$@";
  printf "\n\001${key}\002${val}\003"
}
```

Beyond this, the subshells have `set -e -u` (and maybe -x if you specify --debug) to
help catch errors, but there's really not much else going on for the execution of a single
hook. 

## Emits

Sysunit extracts various values from your units with emit messages, which are just
bits of stdout output delimited with some non-printable characters so sysunit
can differentiate it from diagnostic output that should be shown to the user.

These emit messages follow a format like this:

```
\001
  <key>.<field>
\002
  <message content>
\003
```

The whitespace is insignificant here. The key and field are used to indicate to
Sysunit what kind of message is being emitted, and then the message content is
handled appropriately to do things like set up dependencies or emit values to
dependees.
