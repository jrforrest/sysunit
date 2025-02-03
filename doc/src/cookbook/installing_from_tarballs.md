# Installing From Tarballs

Node.js is a dependency that I usually need to install from tarballs due to it being out of date in
most package managers. Here's the unit I use to do that:

```sh
#node_install.sysu

# [ install.sh ]
meta() params !version:string;
deps() {
    dep ./download_source.sh version=$version -> !dir:node_dir:string;
    dep pkg.sh name=make
    dep pkg.sh name=build-essential
}

check() {
    if which node >/dev/null; then
        present
    fi
}

apply {
    cd $node_dir
    ./configure
    make
    sudo make install
}

remove {
    cd $node_dir
    sudo make uninstall
}

# [ download_source.sh ]
meta() params !verson:string;

deps() {
    dep pkg.sh name=curl
}

check() {
    if [ -d /tmp/node_build/node-$version ]; then
        present
    fi
}

apply() {
    mkdir -p /tmp/node_build
    cd /tmp/node_build
    url="https://github.com/nodejs/node/archive/refs/tags/${version}.tar.gz"
    curl -L $url | tar -xz
    cd node-$version
    emit_value dir `pwd`
}

remove() {
    rm -rf /tmp/node_build
}
```

This unitfile splits the job into two units so the node source isn't unnecessarily
downloaded if there are errors.

It can be run with `sysunit apply ./node_install.sysu/install.sh version=14.17.0`
