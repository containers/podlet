# Changelog

## [0.1.0] - 2023-04-14

The initial release of podlet! Designed for podman v4.5.0 and newer.

### Initial Features

- Create quadlet files:
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