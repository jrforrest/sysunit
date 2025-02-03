# File Dependencies

A unit may also depend on files. This is useful when you need to ensure that a
file exists on the system the unit is to be run on. Files are transported from
the local system to the target system via an appropriate subcommand.

```sh
#ssh_key.sh

meta() {
  desc "Sets an ssh key retrieved from the local system up on the remote system"
  params !name:string
}

deps() {
  dep dir.sh path=/home/jack/.ssh
  file src=/home/jack/.ssh/$name, dest=/home/jack/.ssh/$name
}

apply() {
  chmod 600 ~/.ssh/$name
}
```

Under the hood, commands like `podman cp` or `scp` are used to transport the file, so
entire directories can be given.

If a relative path is given for the source, the directory of the unit is used as the base path.

If a relative path is given for the destination, the home directory of the user the unit is run as
is used as the base path.

### Limitations

- Files can only be transported from the local system to the target, not from other systems.
- Tilde expansion is not supported.
