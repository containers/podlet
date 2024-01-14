# Podlet

![Crates.io](https://img.shields.io/crates/v/podlet?style=flat-square)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/k9withabone/podlet/ci.yaml?event=push&label=ci&logo=github&style=flat-square)
![Crates.io License](https://img.shields.io/crates/l/podlet?style=flat-square)

Podlet generates [podman](https://podman.io/) [quadlet](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) files from a podman command or a compose file.

[![demo.gif](./demo.gif)](https://asciinema.org/a/629332)
Demo created with [autocast](https://github.com/k9withabone/autocast). You can also view the demo on [asciinema](https://asciinema.org/a/629332).

## Features

- Designed for podman v4.7.0 and newer
- Supports the following podman commands:
    - `podman run`
    - `podman kube play`
    - `podman network create`
    - `podman volume create`
- Convert a (docker) compose file to:
    - Multiple quadlet files
    - A pod with a quadlet kube file and Kubernetes YAML
- Generate from an existing container, network, or volume
- Write to stdout or to a file
- Options for including common systemd unit options
- Checks for existing systemd services to avoid conflict
    - Opt-out with `--skip-services-check`

## Install

Podlet can be acquired in several ways:

- Download a prebuilt binary from [releases](https://github.com/k9withabone/podlet/releases)
- As a container: `podman run quay.io/k9withabone/podlet`
    - Container images are available on [quay.io](https://quay.io/repository/k9withabone/podlet) and [docker hub](https://hub.docker.com/r/k9withabone/podlet)
- Use [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) to get a prebuilt binary: `cargo binstall podlet`
- Build and install with `cargo install podlet`

## Usage

```
$ podlet -h

Generate podman quadlet files from a podman command or a compose file

Usage: podlet [OPTIONS] <COMMAND>

Commands:
  podman    Generate a podman quadlet file from a podman command
  compose   Generate podman quadlet files from a compose file
  generate  Generate a podman quadlet file from an existing container, network, or volume
  help      Print this message or the help of the given subcommand(s)

Options:
  -f, --file [<FILE>]              Generate a file instead of printing to stdout
  -u, --unit-directory             Generate a file in the podman unit directory instead of printing to stdout [aliases: unit-dir]
  -n, --name <NAME>                Override the name of the generated file (without the extension)
      --overwrite                  Overwrite existing files when generating a file
      --skip-services-check        Skip the check for existing services of the same name
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

See `podlet --help` for more information.

### Podman Command

```
$ podlet podman -h

Generate a podman quadlet file from a podman command

Usage: podlet podman <COMMAND>

Commands:
  run      Generate a podman quadlet `.container` file
  kube     Generate a podman quadlet `.kube` file
  network  Generate a podman quadlet `.network` file
  volume   Generate a podman quadlet `.volume` file
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

To generate a quadlet file, just put `podlet` in front of your podman command!

```
$ podlet podman run quay.io/podman/hello

# hello.container
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
  -v ./Caddyfile:/etc/caddy/Caddyfile:Z \
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
Volume=./Caddyfile:/etc/caddy/Caddyfile:Z
Volume=caddy_data:/data

[Service]
Restart=always

[Install]
WantedBy=default.target
```

The name for the file was automatically pulled from the image name, but can be overridden with the `--name` option.

Podlet also supports creating kube, network, and volume quadlet files.

```
$ podlet podman kube play --network pasta --userns auto caddy.yaml

# caddy.kube
[Kube]
Yaml=caddy.yaml
Network=pasta
UserNS=auto
```

### Compose

```
$ podlet compose -h

Generate podman quadlet files from a compose file

Usage: podlet compose [OPTIONS] [COMPOSE_FILE]

Arguments:
  [COMPOSE_FILE]  The compose file to convert

Options:
      --pod <POD>  Create a Kubernetes YAML file for a pod instead of separate containers
  -h, --help       Print help (see more with '--help')
```

Let's return to the caddy example, say you have a compose file at `compose-example.yaml`:

```yaml
services:
  caddy:
    image: docker.io/library/caddy:latest
    ports:
      - 8000:80
      - 8443:443
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:Z
      - caddy-data:/data
volumes:
  caddy-data:
```

`podlet compose compose-example.yaml` will create a `caddy.container` like so:

```ini
# caddy.container
[Container]
Image=docker.io/library/caddy:latest
PublishPort=8000:80
PublishPort=8443:443
Volume=./Caddyfile:/etc/caddy/Caddyfile:Z
Volume=caddy-data:/data
```

If a compose file is not given, podlet will search for the following files in the current working directory, in order:

- `compose.yaml`
- `compose.yml`
- `docker-compose.yaml`
- `docker-compose.yml`

In addition, the `--pod` option will generate Kubernetes YAML which groups all compose services in a pod.

```
$ podlet compose --pod caddy compose-example.yaml

# caddy.kube
[Kube]
Yaml=caddy-kube.yaml

---

# caddy-kube.yaml
apiVersion: v1
kind: Pod
metadata:
  name: caddy
spec:
  containers:
  - image: docker.io/library/caddy:latest
    name: caddy
    ports:
    - containerPort: 80
      hostPort: 8000
    - containerPort: 443
      hostPort: 8443
    volumeMounts:
    - mountPath: /etc/caddy/Caddyfile:Z
      name: caddy-etc-caddy-Caddyfile
    - mountPath: /data
      name: caddy-data
  volumes:
  - hostPath:
      path: ./Caddyfile
    name: caddy-etc-caddy-Caddyfile
  - name: caddy-data
    persistentVolumeClaim:
      claimName: caddy-data
```

When converting compose files, not all options are supported by podman/quadlet. This is especially true when converting to a pod as some options must be applied to the pod as a whole. If podlet encounters an unsupported option an error will be returned. You will have to remove or comment out unsupported options to proceed.

See `podlet compose --help` for more information.

### Generate from Existing

```
$ podlet generate -h

Generate a podman quadlet file from an existing container, network, or volume

Usage: podlet generate <COMMAND>

Commands:
  container  Generate a quadlet file from an existing container
  network    Generate a quadlet file from an existing network
  volume     Generate a quadlet file from an existing volume
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help (see more with '--help')
```

If you have an existing container, network, or volume, you can use `podlet generate` to create a quadlet file from it.

```
$ podman container create --name hello quay.io/podman/hello:latest

$ podlet generate container hello

# hello.container
[Container]
ContainerName=hello
Image=quay.io/podman/hello:latest
```

These commands require that `podman` is installed and searchable from the [`PATH`](https://en.wikipedia.org/wiki/PATH_(variable)) environment variable.

See `podlet generate --help` for more information.

### In a Container

While podlet can be used as-is in a container, passing the command to it; if you want to utilize some of the write-to-file functionality, or create quadlet files from compose files, additional volumes may need to be attached.

An example of a generic podman command that runs the most up-to-date version of podlet with the current directory and user's quadlet directory attached to the container would be:

`podman run --rm --userns keep-id -e HOME -e XDG_CONFIG_HOME --user $(id -u) -v "$PWD":"$PWD" -v "$HOME/.config/containers/systemd/":"$HOME/.config/containers/systemd/" -w "$PWD" --security-opt label=disable --pull=newer quay.io/k9withabone/podlet`

Please note that `--security-opt label=disable` may be required for systems with SELinux. If your system does not use SELinux this may not be required.

Alternatively, if you just want podlet to read a specific compose file you can use:

`podman run --rm -v ./compose.yaml:/compose.yaml:Z quay.io/k9withabone/podlet compose /compose.yaml`

## Cautions

Podlet is not (yet) a validator for podman commands. Some podman options are incompatible with each other and most options require specific formatting and/or only accept certain values. However, a few options are fully parsed and validated in order to facilitate creating the quadlet file.

Podlet is meant to be used with podman v4.7.0 or newer. Some quadlet options are unavailable or behave differently with earlier versions of podman/quadlet.

## Contribution

Contributions, suggestions, and/or comments are appreciated! Feel free to create an [issue](https://github.com/k9withabone/podlet/issues), [discussion](https://github.com/k9withabone/podlet/discussions), or [pull request](https://github.com/k9withabone/podlet/pulls).

### Building

Podlet is a normal Rust project, so once [Rust is installed](https://www.rust-lang.org/tools/install), the source code can be cloned and built with:

```shell
git clone git@github.com:k9withabone/podlet.git
cd podlet
cargo build
```

Release builds are created with the `dist` profile:

```shell
cargo build --profile dist
```

### Local CI

If you are submitting code changes in a pull request and would like to run the CI jobs locally, you can run the following commands:

- format: `cargo fmt --check`
- clippy: `cargo clippy`
- test: `cargo test`
- build-container:
    - Ensure the container builds for both x86 and ARM platforms.
    - `podman build --platform linux/amd64 -t podlet .`
    - `podman build --platform linux/arm64/v8 -t podlet .`

## License

All source code for podlet is licensed under the [Mozilla Public License v2.0](https://www.mozilla.org/en-US/MPL/). View the [LICENSE](./LICENSE) file for more information.
