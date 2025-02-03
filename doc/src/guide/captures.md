# Captures

While parameters are sufficient for cascading data down from parent units,
sometimes parent units need to get data back from their children. Captures
accomplish this by allowing a unit to emit values, which can be captured by
their parents when desired. This allows us to do things like get 'facts'
about the system from shared units, or to respond to recoverable error
conditions.

```sh
#pkg.sh
meta() {
  desc "installs a package, will use sudo to escalate priveleges"
  params !name:string
}

deps() {
  dep sudo.sh
  dep os_info.sh -> !id:os_id:string
}

check() {
  case $os_id in
    debian)
      if dpkg -l | grep -q "^ii  $name"; then
        present
      fi
      ;;
    alpine)
      if apk info | grep -q "^$name"; then
        present
      fi
      ;;
    *)
      echo "This unit only works on debian and alpine"
      exit 1
      ;;
  esac
}

# Other operations skipped here, see cookbook for a more complete example of
# package handling.
```


Here, the line `dep "os_info.sh -> id:os_id:string"` tells sysunit that the
`os_info.sh` unit should be run, and that it is expected to emit a value named
`id` as a string which will then be aliased to the `os_info` parameter (the
alias is optional, we could have also specified `-> id:string`, but being able
to add context and avoid name collisions is nice.) This value is then available
in the `pkg.sh` unit via the `$os_id` variable.

### Emitting Values
Now let's look at the `os_info.sh` unit `pkg.sh` depends on:

```sh
#os_info.sh

check() {
  . /etc/os-release

  emit_value id "$ID"
  emit_value name "$NAME"
  emit_value version_id "$VERSION_ID"
  emit_value pretty_name "$PRETTY_NAME"
}
```

Here, the `emit_value` function is used to send key-value pairs to Sysunit, so
they can be provided to dependees. This can be called from `check`, `apply` or
`remove` hooks. We must ensure that all required values get emitted, but
optional values can be skipped. (For example, we may have values which are only
emitted if the unit is removed.)

### Parameter Availability

Capture params are available in the `check`, `apply`, and `remove` hooks, but
not in `deps` or `meta`, since dependencies will not have run until after the
`deps` hook has completed.
