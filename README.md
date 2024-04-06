# Podlet

![Crates.io](https://img.shields.io/crates/v/podlet?style=flat-square)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/containers/podlet/ci.yaml?event=push&label=ci&logo=github&style=flat-square)
![Crates.io License](https://img.shields.io/crates/l/podlet?style=flat-square)

Podlet generates [podman](https://podman.io/) [quadlet](https://docs.podman.io/en/stable/markdown/podman-systemd.unit.5.html) files from a podman command, compose file, or existing object.

[![demo.gif](./demo.gif)](https://asciinema.org/a/633918)
Demo created with [autocast](https://github.com/k9withabone/autocast). You can also view the demo on [asciinema](https://asciinema.org/a/633918).

## Features

- Supports the following podman commands:
    - `podman run`
    - `podman kube play`
    - `podman network create`
    - `podman volume create`
    - `podman image pull`
- Convert a (docker) compose file to:
    - Multiple quadlet files.
    - A pod with a quadlet kube file and Kubernetes YAML.
- Generate from existing:
    - Containers
    - Networks
    - Volumes
    - Images
- Write to stdout or to a file.
- Options for including common systemd unit options.
- Checks for existing systemd services to avoid conflict.
    - Opt-out with `--skip-services-check`.
- Set podman version compatibility with `--podman-version`.
- Resolve relative host paths with `--absolute-host-paths`.

## Install

Podlet can be acquired in several ways:

- Download a prebuilt binary from [releases](https://github.com/containers/podlet/releases).
- As a container: `podman run quay.io/k9withabone/podlet`.
    - Container images are available on [quay.io](https://quay.io/repository/k9withabone/podlet) and [docker hub](https://hub.docker.com/r/k9withabone/podlet).
- Use [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) to get a prebuilt binary: `cargo binstall podlet`.
- Build and install with `cargo install podlet`.

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
  -f, --file [<FILE>]                        Generate a file instead of printing to stdout
  -u, --unit-directory                       Generate a file in the podman unit directory instead of printing to stdout [aliases: unit-dir]
  -n, --name <NAME>                          Override the name of the generated file (without the extension)
      --overwrite                            Overwrite existing files when generating a file
      --skip-services-check                  Skip the check for existing services of the same name
  -p, --podman-version <PODMAN_VERSION>      Podman version generated quadlet files should conform to [default: 4.8] [aliases: compatibility, compat] [possible values: 4.4, 4.5, 4.6, 4.7, 4.8]
  -a, --absolute-host-paths [<RESOLVE_DIR>]  Convert relative host paths to absolute paths
  -d, --description <DESCRIPTION>            Add a description to the unit
      --wants <WANTS>                        Add (weak) requirement dependencies to the unit
      --requires <REQUIRES>                  Similar to --wants, but adds stronger requirement dependencies
      --before <BEFORE>                      Configure ordering dependency between units
      --after <AFTER>                        Configure ordering dependency between units
  -i, --install                              Add an [Install] section to the unit
      --wanted-by <WANTED_BY>                Add (weak) parent dependencies to the unit
      --required-by <REQUIRED_BY>            Similar to --wanted-by, but adds stronger parent dependencies
  -h, --help                                 Print help (see more with '--help')
  -V, --version                              Print version
```

See `podlet --help` for more information.

### Podman Command

```
$ podlet podman -h

Generate a podman quadlet file from a podman command

Usage: podlet podman [OPTIONS] <COMMAND>

Commands:
  run      Generate a podman quadlet `.container` file
  kube     Generate a podman quadlet `.kube` file
  network  Generate a podman quadlet `.network` file
  volume   Generate a podman quadlet `.volume` file
  image    Generate a podman quadlet `.image` file
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help (see more with '--help')

Podman Global Options:
      --cgroup-manager <MANAGER>             Cgroup manager to use [possible values: cgroupfs, systemd]
      --conmon <PATH>                        Path of the conmon binary
      --connection <CONNECTION_URI>          Connection to use for remote Podman service
      --events-backend <TYPE>                Backend to use for storing events [possible values: file, journald, none]
      --hooks-dir <PATH>                     Set the OCI hooks directory path
      --identity <PATH>                      Path to ssh identity file
      --imagestore <PATH>                    Path to the 'image store'
      --log-level <LEVEL>                    Log messages at and above specified level [default: warn] [possible values: debug, info, warn, error, fatal, panic]
      --module <PATH>                        Load the specified `containers.conf(5)` module
      --network-cmd-path <PATH>              Path to the `slirp4netns(1)` command binary
      --network-config-dir <DIRECTORY>       Path of the configuration directory for networks
      --out <PATH>                           Redirect the output of podman to a file without affecting the container output or its logs
  -r, --remote[=<REMOTE>]                    Access remote Podman service [possible values: true, false]
      --root <VALUE>                         Path to the graph root directory where images, containers, etc. are stored
      --runroot <VALUE>                      Storage state directory where all state information is stored
      --runtime <VALUE>                      Path to the OCI-compatible binary used to run containers
      --runtime-flag <FLAG>                  Add global flags for the container runtime
      --ssh <VALUE>                          Define the ssh mode [possible values: golang, native]
      --storage-driver <VALUE>               Select which storage driver is used to manage storage of images and containers
      --storage-opt <VALUE>                  Specify a storage driver option
      --syslog                               Output logging information to syslog as well as the console
      --tmpdir <PATH>                        Path to the tmp directory for libpod state content
      --transient-store[=<TRANSIENT_STORE>]  Enable transient container storage [possible values: true, false]
      --url <VALUE>                          URL to access Podman service
      --volumepath <VALUE>                   Volume directory where builtin volume information is stored
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

Podlet also supports creating kube, network, volume, and image quadlet files.

```
$ podlet podman kube play --network pasta --userns auto caddy.yaml

# caddy.kube
[Kube]
Yaml=caddy.yaml
Network=pasta
UserNS=auto
```

Global podman options are added to the `GlobalArgs=` quadlet option.

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

`podlet compose compose-example.yaml` will create a `caddy.container` file like so:

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

Also note that podlet does not yet support [compose interpolation](https://github.com/compose-spec/compose-spec/blob/master/spec.md#interpolation).

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
  image      Generate a quadlet file from an image in local storage
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help (see more with '--help')
```

If you have an existing container, network, volume, or image, you can use `podlet generate` to create a quadlet file from it.

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

Please note that `--security-opt label=disable` may be required for systems with SELinux. If your system does not use SELinux, the option is not needed. Podman recommends disabling SELinux separation when mounting system files and directories to containers. See the note at the end of the "Labeling Volume Mounts" section in the `podman run --volume` [documentation](https://docs.podman.io/en/stable/markdown/podman-run.1.html#volume-v-source-volume-host-dir-container-dir-options).

Alternatively, if you just want podlet to read a specific compose file you can use:

`podman run --rm -v ./compose.yaml:/compose.yaml:Z quay.io/k9withabone/podlet compose /compose.yaml`

## Cautions

Podlet is primarily a tool for helping to get started with podman systemd units, aka quadlet files. It is not meant to be an end-all solution for creating and maintaining quadlet files. Files created with podlet should always be reviewed before starting the unit.

Podlet is not (yet) a validator for podman commands. Some podman options are incompatible with each other and most options require specific formatting and/or only accept certain values. However, a few options are fully parsed and validated in order to facilitate creating the quadlet file.

## Contribution

Contributions, suggestions, and/or comments are appreciated! Feel free to create an [issue](https://github.com/containers/podlet/issues), [discussion](https://github.com/containers/podlet/discussions), or [pull request](https://github.com/containers/podlet/pulls).

### Building

Podlet is a normal Rust project, so once [Rust is installed](https://www.rust-lang.org/tools/install), the source code can be cloned and built with:

```shell
git clone git@github.com:containers/podlet.git
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
