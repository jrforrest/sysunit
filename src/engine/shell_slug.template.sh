set -e -u

exec 2>&1
# Oh we should set $DEBUG to --debug and $VERBOSE to --verbose when appropriate
# so we can use them in unit scripts to set verbosity of commands

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

abort() {
  echo "$1"
  return 1
}

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
