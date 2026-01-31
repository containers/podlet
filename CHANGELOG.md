# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-02-02

After an extended hiatus, Podlet is back! I, [@k9withabone], apologize for my prolonged absence and silence. Life is overwhelming at times.
The good news is [@TheRealBecks] has stepped up and joined me as a maintainer! We are also looking another maintainer to join us! If you are interested please comment on [this discussion (#171)](https://github.com/containers/podlet/discussions/171).

This is a fairly large release which includes a number of bug fixes and new features. In the future, the goal is to have smaller releases more frequently. Here are some highlights of this release:

- Added support for Quadlet options introduced in Podman v5.1.0 ([#85](https://github.com/containers/podlet/pull/85)) and v5.2.0 ([#123](https://github.com/containers/podlet/pull/123)). This includes support for creating `.build` Quadlet files with `podlet podman build` and from the `build` section of a service in a Compose file.

- Added the `-s, --split-options` option ([#168](https://github.com/containers/podlet/pull/168)) which allows the user to specify which Quadlet options should be split onto multiple lines.
By default, Podlet combines all Quadlet options that can be into a space separated list.
For example, `podlet podman run -e ONE=one -e TWO=two image` results in:

  ```ini
  # image.container
  [Container]
  Environment=ONE=one TWO=two
  Image=image
  ```

  While `podlet -s Environment podman run -e ONE=one -e TWO=two image` results in:

  ```ini
  # image.container
  [Container]
  Environment=ONE=one
  Environment=TWO=two
  Image=image
  ```

  See `podlet --help` for more information and all Quadlet options that can be split.

- Fixed a number of bugs related to Compose file handling ([#122](https://github.com/containers/podlet/pull/122)).

- Added a Minimum Supported Rust Version (MSRV) policy ([#169](https://github.com/containers/podlet/pull/169)) which can be seen in the [contribution guide](https://github.com/containers/podlet/blob/2e9898d001d965bf15376aabd31c7c0ba2b1c937/CONTRIBUTING.md#minimum-supported-rust-version-msrv).

### Features

- *(container)* Add `GroupAdd=` Quadlet option.
- *(container)* Add `subpath` image mount option.
- *(generate)* Add options to ignore unsupported pod options. ([#86](https://github.com/containers/podlet/issues/86))
  - Added `--ignore-infra-conmon-pidfile` and `--ignore-pod-id-file` options to `podlet generate pod`.
  - The `--infra-conmon-pidfile` and `--pod-id-file` options are deliberately unimplemented by `podlet podman pod create` as they are set by Quadlet when generating the systemd service unit file, and cannot be set multiple times. Because `podlet generate pod` uses `podlet podman pod create` internally, this caused an error when attempting to generate a `.pod` Quadlet file from a pod created using either of those options, such as those created with the deprecated `podman generate systemd` command.
- *(compose)* Merge `<<` keys. ([#58](https://github.com/containers/podlet/issues/58))
- *(compose)* Perform additional validation.
  - Ensures that networks, volumes (when used across multiple services), configs, and secrets used in each service are defined in the appropriate top-level field.
- *(container)* Add `LogOpt=` Quadlet option.
- *(container)* Add `StopSignal=` Quadlet option.
- *(container)* Add `NetworkAlias=` Quadlet option.
- *(pod)* Add `NetworkAlias=` Quadlet option.
- *(build)* Generate `.build` Quadlet file from command.
  - Added the `podlet podman build` subcommand.
- *(compose)* `.build` Quadlet files from Compose. ([#100](https://github.com/containers/podlet/issues/100))
  - Added support to `podlet compose` for converting the `build` section of a Compose service to a `.build` Quadlet file.
- *(compose)* Search for `podman-compose.yaml` when using `podlet compose` by [@TheRealBecks] in [#167](https://github.com/containers/podlet/pull/167).
- Add `--split-options` option. ([#118](https://github.com/containers/podlet/issues/118))
  - By default, when generating a Quadlet file, Podlet will combine all Quadlet options that can be into a space separated list (e.g., `Environment=ONE=one TWO=two`). The `--split-options` option allows the user to specify which Quadlet options they would like Podlet to write on separate lines.

### Bug Fixes

- Support `podman pod create --name` option by [@ananthb](https://github.com/ananthb) in [#89](https://github.com/containers/podlet/pull/89).
- *(compose)* Append `.network` to container network name. ([#90](https://github.com/containers/podlet/issues/90))
- *(generate)* Accept array or object from `podman inspect` output. ([#94](https://github.com/containers/podlet/issues/94))
  - Podman v5.0.0 and newer always returns an array from `podman inspect`. Older versions of Podman may return a single JSON object if there is only one result, notably for `podman pod inspect`.
- *(compose)* Prepend container dependencies with pod name. ([#114](https://github.com/containers/podlet/issues/114))
  - When the `podlet compose --pod` option is used, the names of the services from the Compose file are prepended with the pod name. If a service had a dependency via the `depends_on` attribute, the name of the dependency was not similarly prepended with the pod name when added to the `[Unit]` section of the Quadlet file.
- *(compose)* Support `services[].memswap_limit`.
  - The `services[].memswap_limit` attribute is translated to `podman run --memory-swap`.
- *(container)* Support short flag `-h` for `podman run --hostname`. ([#105](https://github.com/containers/podlet/issues/105))
  - Changed short help flag to `-?` for `podlet podman run`. This could be a potentially minor breaking change for some users.
- *(compose)* `.build` Quadlet file error reporting. ([#126](https://github.com/containers/podlet/issues/126))
  - If there was an error converting the `build` section of a Compose service it would not be reported to the user. The rest of the service would fail to convert to a `.container` Quadlet file as the `image` would not be set (it's set after the `build` section is successfully converted). The user would receive a "`image` or `build` is required" error as the service error would be reported first.

### Documentation

- *(readme)* Add homebrew to the installation section by [@cprecioso](https://github.com/cprecioso) in [#80](https://github.com/containers/podlet/pull/80).
- *(contributing)* Add MSRV policy.
  - Set MSRV to 1.85, following the Rust version in Debian stable.
- *(readme)* Update demo, features, and usage.

### Refactor

- Add generic impls of `HostPaths` trait.
- Destruct tuples to enhance readability by [@Integral-Tech](https://github.com/Integral-Tech).
- `impl Serialize for podlet::quadlet::File`.
  - Changed `podlet::serde::quadlet::Serializer` to accept sequences and tuples in addition to structs. Elements in a sequence are serialized as separate sections. Tuples are combined into a single section (for combining `podlet::quadlet::Resource` and `podlet::quadlet::Globals`).

### Miscellaneous

- *(release-container)* Add `--all` to `buildah manifest push`. ([#82](https://github.com/containers/podlet/issues/82))
- Add Podman v5.1.0 to `PodmanVersion`.
- *(deps)* Update `compose_spec` to v0.3.0. ([#91](https://github.com/containers/podlet/issues/91), [#96](https://github.com/containers/podlet/issues/96), [#106](https://github.com/containers/podlet/issues/106), [#117](https://github.com/containers/podlet/issues/117))
- Add Podman v5.2.0 to `PodmanVersion`.
- *(ci)* Add `msrv` job.
- Update to Rust 2024 edition.
  - Includes changes to formatting for the [Rust 2024 style edition](https://doc.rust-lang.org/stable/edition-guide/rust-2024/rustfmt-style-edition.html).
- *(deps)* Replace `nix` with `rustix`.
- *(lints)* Fix Clippy lints.
- *(release)* Update dist.
  - Formerly known as cargo-dist.
- *(ci)* Bump actions/checkout to v6.
- *(ci)* Add `spellcheck` job.
- *(deps)* Update dependencies.

### New Contributors

- @TheRealBecks made their first contribution in [#167](https://github.com/containers/podlet/pull/167)
- @Integral-Tech made their first contribution in [#129](https://github.com/containers/podlet/pull/129)
- @ananthb made their first contribution in [#89](https://github.com/containers/podlet/pull/89)
- @cprecioso made their first contribution in [#80](https://github.com/containers/podlet/pull/80)

## [0.3.0] - 2024-05-21

Big release for Podlet!

In case you didn't already notice, Podlet is now officially a part of the [Containers](https://github.com/containers/) community! As a part of the transition, a new code of conduct, security policy, and contribution guidelines were added ([#76](https://github.com/containers/podlet/pull/76)). Additionally, the Podlet container image is now available at ghcr.io/containers/podlet. The existing images at quay.io/k9withabone/podlet and docker.io/k9withabone/podlet are deprecated and will not be receiving updates.

Under the hood, the library used to deserialize Compose files was changed ([#73](https://github.com/containers/podlet/pull/73)). As a result, only Compose files which follow the [Compose specification](https://github.com/compose-spec/compose-spec) are supported. The top-level `version` field is completely ignored. Most Compose files should still work as before. This was a large change so look out for bugs.

Added support for Quadlet options introduced in Podman v5.0.0 ([#75](https://github.com/containers/podlet/pull/75)). The headline feature is support for generating Quadlet `.pod` files. They can be generated from:

- A Podman command with `podlet podman pod create`.
- A Compose file with `podlet compose --pod`.
- An existing pod with `podlet generate pod`.
  - This creates a `.pod` file and a `.container` file for each container in the pod.

Note that the existing option for generating Kubernetes Pod YAML from a Compose file was renamed to `podlet compose --kube`. Both the `--pod` and `--kube` options of `podlet compose` do not take an argument and instead require the top-level `name` field in the Compose file. The `name` is used as the name of the pod and in the filename of the created files.

### Features
- Add `podlet --binds-to` option.
- **BREAKING** *(compose)* Rename `podlet compose --pod` to `podlet compose --kube`.
- *(container)* Add `Entrypoint=` Quadlet option.
- *(container)* Add `StopTimeout=` Quadlet option.
- *(container)* Support `Notify=healthy` Quadlet option.
- *(container)* Support `no-dereference` option for `Mount=`.
- *(container)* Add `podman run --preserve-fd` option.
- *(container)* Add `podman run --gpus` option.
- *(container)* Add `podman run --retry` option.
- *(container)* Add `podman run --retry-delay` option.
- Add `podman --config` global option.
- *(pod)* Generate `.pod` Quadlet file from command.
  - Adds the `podlet podman pod create` subcommand.
  - The `--infra-conmon-pidfile` and `--pod-id-file` options were deliberately not implemented as they are set by Quadlet in the generated `{name}-pod.service` file and can't be set multiple times.
- **BREAKING** *(compose)* Re-add `podlet compose --pod` option.
  - The `--pod` option causes podlet to create a `.pod` Quadlet file in addition to the `.container`, `.volume`, and `.network` files. The containers are linked to the pod and their published ports are moved.
- *(generate)* Quadlet files from an existing pod and its containers.
  - Adds the `podlet generate pod` subcommand.
    - Runs `podman pod inspect` on the given pod.
    - Deserializes the output.
    - Parses the pod creation command.
    - Does the same for each of the pod's containers.

### Bug Fixes
- Use Quadlet serializer for `Unit` `Display` implementation ([#64](https://github.com/containers/podlet/issues/64)).
  - Brings `Unit` inline with the other sections of the generated Quadlet file.
- *(container)* Add `podman run --uts` option.
- *(container)* `--pids-limit` range is `-1..=u32::MAX`.
- *(container)* Enforce `--blkio-weight` range `10..=1000`.
- *(container)* `--blkio-weight-device` can be specified multiple times.
- *(container)* Don't add empty `PodmanArgs=` when downgrading Podman version.
- Correct use of `eyre::bail!()` on non-Unix platforms.

### Documentation
- *(clippy)* Fix Clippy lint warning for `Idmap`.
- *(compose)* `--kube` help add `name` requirement.
- Add code of conduct.
- Add security policy.
- Update links to the repository.
  - The repository is now at https://github.com/containers/podlet.
- *(contributing)* Add contribution guidelines.
  - Adapted from the Buildah/Podman contribution guidelines.
  - Suggests the use of conventional commits and clarifies that the `Signed-off-by` footer is required for a PR to be merged.
  - Moved and expanded upon the building and continuous integration sections from the `README.md` file to the new `CONTRIBUTING.md` file.
- *(readme)* Update container image location.
  - The Podlet container image is now located at ghcr.io/containers/podlet.
- Fix Podman and Quadlet capitalization.
- *(readme)* Update demo, features, and usage.

### Refactor
- **BREAKING** *(deps)* Remove `docker_compose_types`.
- **BREAKING** *(compose)* Deserialize `compose_spec::Compose`.
- `cli::Unit::is_empty()`
  - Check each field instead of comparing to the default.
- *(compose)* Conversion to `quadlet::File`s from `compose_spec::Compose`.
- *(compose)* `quadlet::Globals` from `compose_spec::Service`.
- *(compose)* Container Quadlet options from `compose_spec::Service`.
- *(compose)* Container Podman args from `compose_spec::Service`.
- *(compose)* `quadlet::Network` from `compose_spec::Network`.
- *(compose)* `quadlet::Volume` from `compose_spec::Volume`.
- *(compose)* Kubernetes YAML from `compose_spec::Compose`.
- *(container)* Destructure in Quadlet option conversion.
- *(compose)* Move `podlet compose` args into their own struct.

### Miscellaneous
- *(deps)* Remove `duration-str` dependency.
  - All usages were replaced with `compose_spec::duration`.
- Add Podman v5.0.0 to Podman versions.
  - Also added v4.9.X aliases to 4.8 and v5.0.X aliases to 5.0.
- *(container)* Reorder fields to match Quadlet docs.
- *(lints)* Fix new rust 1.78 clippy lints.
- **BREAKING** *(release-container)* Push to ghcr.io/containers/podlet.
  - The docker.io/k9withabone/podlet and quay.io/k9withabone/podlet container images will no longer be updated.
- *(release-container)* Add annotations/labels to manifest/image.
  - Adds labels to the Podlet container image and annotations to the multi-arch manifest as suggested by the GitHub packages documentation: <https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#labelling-container-images>.
- *(ci)* Bump actions/checkout to v4.
- *(ci)* Use Buildah container to build Podlet container.
- *(deps)* Update dependencies.
- *(release)* Update cargo-dist.
- *(release-container)* Fix manifest annotation quoting.
- *(ci)* Use consistent formatting.
- *(ci)* Add image to manifest when building container.
- *(ci)* Add `build` job.

## [0.2.4] - 2024-01-30

### Features

- Set compatibility with `--podman-version` ([#45](https://github.com/containers/podlet/issues/45))
- Add support for Quadlet options introduced in Podman v4.8.0 ([#30](https://github.com/containers/podlet/issues/30))
    - Container
        - `GIDMap=`
        - `ReadOnlyTmpfs=`
        - `SubGIDMap=`
        - `SubUIDMap=`
        - `UIDMap=`
        - Remove `VolatileTmp=`
    - Volume
        - `Driver=`
        - `Image=`
    - Image
        - Brand new!
        - Generate `.image` Quadlet files with:
            - `podlet podman image pull`
            - `podlet generate image`
    - All Quadlet file types
        - `ContainersConfModule=`
        - `GlobalArgs=`
- Convert relative host paths to absolute paths with `--absolute-host-paths` ([#52](https://github.com/containers/podlet/issues/52))
    - Does not affect paths in the `PodmanArgs=` Quadlet option or Kubernetes YAML files.
    - As part of the work to implement this, the following Quadlet options are now fully parsed and validated:
        - `AddDevice=`
        - `Mount=`
        - `Rootfs=`
        - `Volume=`
        - `DecryptionKey=`

### Security

- Remove ASCII control characters (except whitespace) from container commands

### Documentation

- *(readme)* Map user into Podlet container ([#50](https://github.com/containers/podlet/pull/50), thanks [@rugk](https://github.com/rugk)!)
- *(readme)* Update demo, features, and usage

### Refactor

- *(container)* Parse security opts with `str::strip_prefix`
- Remove arg serializer's map functionality

### Miscellaneous Tasks

- Add Podman v4.9.0 to Podman versions
- Update dependencies
- *(ci)* Update cargo-dist

## [0.2.3] - 2023-12-31

### Features

- Add support for Quadlet options introduced in Podman v4.7.0 ([#29](https://github.com/containers/podlet/issues/29))
    - Container
        - `DNS=`
        - `DNSOption=`
        - `DNSSearch=`
        - `PidsLimit=`
        - `ShmSize=`
        - `Ulimit=`
    - Kube
        - `AutoUpdate=`
    - Network
        - `DNS=`
- Add `podlet generate` subcommands for generating Quadlet files from existing:
    - Containers ([#23](https://github.com/containers/podlet/issues/23))
    - Networks
    - Volumes

### Bug Fixes

- *(compose)* `network_mode` accept all Podman values ([#38](https://github.com/containers/podlet/issues/38))
    - Improved error message for unsupported values
- *(network)* Support `<start-IP>-<end-IP>` syntax for `--ip-range`

### Documentation

- *(readme)* Podman v4.7.0
- *(readme)* Update demo and usage

### Miscellaneous Tasks

- *(ci)* Skip container run for conmon v2.1.9
- *(lint)* Fix new rust 1.75 clippy warnings
- Update dependencies

## [0.2.2] - 2023-12-15

### Features

- Add support for Quadlet options introduced in Podman v4.6.0 ([#28](https://github.com/containers/podlet/issues/28))
    - Container
        - `Sysctl=` ([#22](https://github.com/containers/podlet/pull/22), thanks [@b-rad15](https://github.com/b-rad15)!)
        - `AutoUpdate=`
        - `HostName=`
        - `Pull=`
        - `WorkingDir=`
        - `SecurityLabelNested=`
        - `Mask=`
        - `Unmask=`
    - Kube, Network, and Volume
        - `PodmanArgs=`
- *(compose)* Support volume `driver` field

### Bug Fixes

- *(container)* Arg `--tls-verify` requires =
- *(network)* Filter out empty `Options=` Quadlet option
- Escape newlines in joined Quadlet values ([#32](https://github.com/containers/podlet/issues/32))
- *(compose)* Support `cap_drop`, `userns_mode`, and `group_add` service fields ([#31](https://github.com/containers/podlet/issues/31), [#34](https://github.com/containers/podlet/issues/34))
- *(compose)* Split `command` string ([#36](https://github.com/containers/podlet/issues/36))
    - When the command is converted to the `Exec=` Quadlet option, it is now properly quoted. When converting to k8s, it is properly split into args.

### Documentation

- *(readme)* Podman v4.6.0
- *(changelog)* Add `git-cliff` configuration

### Refactor

- Use custom serializer for `PodmanArgs=`
- Use custom serializer for Quadlet sections

### Miscellaneous Tasks

- Update dependencies

## [0.2.1] - 2023-11-28

### Features

- Compose: Read compose file from stdin ([#18](https://github.com/containers/podlet/discussions/18))
    - For `podlet compose`, if a compose file is not provided and stdin is not a terminal, or `-` is provided, Podlet will attempt to read a compose file from stdin.
    - For example `cat compose-example.yaml | podlet compose` or `cat compose-example.yaml | podlet compose -`

### Bug Fixes

- Truncate when overwriting existing files
- Compose service volumes can be mixed long and short form ([#26](https://github.com/containers/podlet/issues/26))

### Documentation

- Readme: Add sample Podlet container usage instructions ([#17](https://github.com/containers/podlet/pull/17), thanks [@Nitrousoxide](https://github.com/Nitrousoxide)!)
- Readme: Update description, add build and local ci instructions

### Miscellaneous Tasks

- CI: Update Podman for build and publish of container
- CI: Add container builds to regular checks
- Update dependencies
- CI: Update cargo-dist to v0.5.0

### Refactor

- `quadlet::writeln_escape_spaces` write to formatter
- Consistent use of `eyre::bail` and `eyre::ensure`
- Add `quadlet::Kube::new()`
- Simplify `cli::File::write()`
- Split `compose_try_into_quadlet_files()`
- Move compose functions into their own module
- Move lints to Cargo.toml, add additional lints

### Styling

- Fix let-else formatting

## [0.2.0] - 2023-06-15

### Added

- Check for existing systemd unit files with the same name as the service generated by Quadlet from the Podlet generated Quadlet file and throw an error if there is a conflict ([#14](https://github.com/containers/podlet/issues/14)).
    - Use `--skip-services-check` to opt-out.
- Convert a (docker) compose file ([#9](https://github.com/containers/podlet/issues/9)) to:
    - Multiple Quadlet files
    - A pod with a Quadlet kube file and Kubernetes YAML

### Changed

- **Breaking**: files are no longer overwritten by default, added `--overwrite` flag if overwriting is desired.

## [0.1.1] - 2023-04-19

### Added

- A container image of Podlet now available on [quay.io](https://quay.io/repository/k9withabone/podlet) and [docker hub](https://hub.docker.com/r/k9withabone/podlet).
- Option flag for outputting to Podman unit directory `--unit-directory`.
    - Places the generated file in the appropriate directory (i.e. `/etc/containers/systemd`, `~/.config/containers/systemd`) for use by Quadlet.

## [0.1.0] - 2023-04-14

The initial release of Podlet! Designed for Podman v4.5.0 and newer.

### Initial Features

- Create Quadlet files:
    - `.container` - `podman run`
    - `.kube` - `podman kube play`
    - `.network` - `podman network create`
    - `.volume` - `podman volume create`
- Write to stdout, or to a file.
    - The file name, if not provided, is pulled from the container name or image, kube file, or network or volume name.
- Options for common systemd unit options
    - [Unit]
        - Description=
        - Wants=
        - Requires=
        - Before=
        - After=
    - [Service]
        - Restart=
    - [Install]
        - WantedBy=
        - RequiredBy=

[@k9withabone]: https://github.com/k9withabone
[@TheRealBecks]: https://github.com/TheRealBecks

[0.3.1]: https://github.com/containers/podlet/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/containers/podlet/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/containers/podlet/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/containers/podlet/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/containers/podlet/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/containers/podlet/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/containers/podlet/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/containers/podlet/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/containers/podlet/compare/f9a7aadf5fca4966c3e8c7e6e495749d93029c80...v0.1.0
