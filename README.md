# Podlet

![Crates.io](https://img.shields.io/crates/v/podlet?style=flat-square)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/k9withabone/podlet/format-clippy-test.yaml?event=push&label=ci&logo=github&style=flat-square)
![Crates.io License](https://img.shields.io/crates/l/podlet?style=flat-square)

Podlet generates [podman](https://podman.io/) [quadlet](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) (systemd-like) files from a podman command.

![Made with VHS](https://vhs.charm.sh/vhs-4x04CoFBi5Hj1EZ0zKlSWE.gif)

## Features

- Write to stdout or to a file.
- Supports the following podman commands:
    - `podman run`
    - `podman kube play`
    - `podman network create`
    - `podman volume create`
- Options for including common systemd unit options.

## Install

Podlet can be acquired in several ways:

- Download a prebuilt binary from [releases](https://github.com/k9withabone/podlet/releases)

- Use [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) to get a prebuilt binary: `cargo binstall podlet`

- Build and install with `cargo install podlet`

## Usage

```
$ podlet -h

Podlet generates podman quadlet (systemd-like) files from a podman command.

Usage: podlet [OPTIONS] <COMMAND>

Commands:
  podman  Generate a podman quadlet file from a podman command
  help    Print this message or the help of the given subcommand(s)

Options:
  -f, --file [<FILE>]              Generate a file instead of printing to stdout
  -n, --name <NAME>                Override the name of the generated file (without the extension)
  -d, --description <DESCRIPTION>  Add a description to the unit
      --wants <WANTS>              Add (weak) requirement dependencies to the unit
      --requires <REQUIRES>        Similar to --wants, but adds stronger requirement dependencies
      --before <BEFORE>            Configure ordering dependency between units
      --after <AFTER>              Configure ordering dependency between units
  -i, --install                    Add an [Install] section to the unit
      --wanted-by <WANTED_BY>      Add (weak) parent dependencies to the unit
      --required-by <REQUIRED_BY>  Similar to --wanted-by, but adds stronger parent dependencies
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version
```

To generate a quadlet file, just put `podlet` in front of your podman command!

```
$ podlet podman run quay.io/podman/hello

[Container]
Image=quay.io/podman/hello
```

This is useful for more complicated commands you are copying. For example, let's create a quadlet file for running caddy. We'll also use a few options for additional sections in the file.

```
$ podlet --file . --install --description Caddy \
  podman run \
  --restart always \
  -p 8000:80 \
  -p 8443:443 \
  -v ./Caddyfile:/etc/caddy/Caddyfile \
  -v caddy_data:/data \
  docker.io/library/caddy:latest

Wrote to file: ./caddy.container

$ cat caddy.container

[Unit]
Description=Caddy

[Container]
Image=docker.io/library/caddy:latest
PublishPort=8000:80
PublishPort=8443:443
Volume=./Caddyfile:/etc/caddy/Caddyfile
Volume=caddy_data:/data

[Service]
Restart=always

[Install]
WantedBy=default.target
```

The name for the file was automatically pulled from the image name, but can be overridden with the `--name` option.

Podlet also supports creating kube, network, and volume quadlet files. However, not all options for their corresponding podman commands are supported by quadlet. Accordingly, those options are also not supported by podlet.

```
$ podlet podman kube play --network pasta --userns auto kube.yaml

[Kube]
Yaml=kube.yaml
Network=pasta
RemapUsers=auto
```

## Contribution

This is my (@k9withabone) first real rust project and is mostly meant as a learning project for myself. That said, contributions, suggestions, and/or comments are appreciated!

## License

All source code for podlet is licensed under the [Mozilla Public License v2.0](https://www.mozilla.org/en-US/MPL/). View the [LICENSE](./LICENSE) file for more information.
