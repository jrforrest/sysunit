exec 2>&1

meta() : ;
deps() : ;
check() : ;
apply() : ;
rollback() : ;
dep() _emit dep.unit $@;
file() _emit dep.file $@;
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
