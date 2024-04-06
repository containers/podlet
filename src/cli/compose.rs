use std::{
    fs::File,
    io::{self, IsTerminal},
    iter, mem,
    path::Path,
    rc::Rc,
};

use color_eyre::{
    eyre::{self, OptionExt, WrapErr},
    Help,
};
use compose_spec::Compose;

use crate::quadlet::{self, Globals};

use super::{image_to_name, unit::Unit, ComposeService, PodmanCommands};

/// Deserialize [`Compose`] from a file at the given [`Path`], stdin, or a list of default files.
///
/// If the path is '-', or stdin is not a terminal, the [`Compose`] is deserialized from stdin.
/// If a path is not provided, the files `compose.yaml`, `compose.yml`, `docker-compose.yaml`,
/// and `docker-compose.yml` are, in order, looked for in the current directory.
///
/// # Errors
///
/// Returns an error if:
///
/// - There was an error opening the given file.
/// - Stdin was selected and stdin is a terminal.
/// - No path was given and none of the default files could be opened.
/// - There was an error deserializing [`Compose`].
pub fn from_file_or_stdin(path: Option<&Path>) -> color_eyre::Result<Compose> {
    let (compose_file, path) = if let Some(path) = path {
        if path.as_os_str() == "-" {
            return from_stdin();
        }
        let compose_file = File::open(path)
            .wrap_err("could not open provided compose file")
            .suggestion("make sure you have the proper permissions for the given file")?;
        (compose_file, path)
    } else {
        const FILE_NAMES: [&str; 4] = [
            "compose.yaml",
            "compose.yml",
            "docker-compose.yaml",
            "docker-compose.yml",
        ];

        if !io::stdin().is_terminal() {
            return from_stdin();
        }

        let mut result = None;
        for file_name in FILE_NAMES {
            if let Ok(compose_file) = File::open(file_name) {
                result = Some((compose_file, file_name.as_ref()));
                break;
            }
        }

        result.ok_or_eyre(
            "a compose file was not provided and none of \
                `compose.yaml`, `compose.yml`, `docker-compose.yaml`, or `docker-compose.yml` \
                exist in the current directory or could not be read",
        )?
    };

    serde_yaml::from_reader(compose_file)
        .wrap_err_with(|| format!("File `{}` is not a valid compose file", path.display()))
}

/// Deserialize [`Compose`] from stdin.
///
/// # Errors
///
/// Returns an error if stdin is a terminal or there was an error deserializing [`Compose`].
fn from_stdin() -> color_eyre::Result<Compose> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        eyre::bail!("cannot read compose from stdin, stdin is a terminal");
    }

    serde_yaml::from_reader(stdin).wrap_err("data from stdin is not a valid compose file")
}

/*
/// Converts a [`Command`] into a `Vec<String>`, splitting the `String` variant as a shell would.
///
/// # Errors
///
/// Returns an error if, while splitting the string variant, the command ends while in a quote or
/// has a trailing unescaped '\\'.
pub fn command_try_into_vec(command: Command) -> color_eyre::Result<Vec<String>> {
    match command {
        Command::Simple(s) => shlex::split(&s)
            .ok_or_else(|| eyre::eyre!("invalid command: `{s}`"))
            .suggestion(
                "In the command, make sure quotes are closed properly and there are no \
                trailing \\. Alternatively, use an array instead of a string.",
            ),
        Command::Args(args) => Ok(args),
    }
}

/// Attempt to convert a [`Compose`] into an iterator of [`quadlet::File`].
pub fn try_into_quadlet_files<'a>(
    mut compose: Compose,
    unit: Option<&'a Unit>,
    install: Option<&'a quadlet::Install>,
) -> impl Iterator<Item = color_eyre::Result<quadlet::File>> + 'a {
    // Get a map of volumes to whether the volume has options associated with it for use in
    // converting a service into a quadlet file. Extra volume options must be specified in a
    // separate quadlet file which is referenced from the container quadlet file.
    let volume_has_options = compose
        .volumes
        .0
        .iter()
        .map(|(name, volume)| (name.clone(), matches!(volume, MapOrEmpty::Map(_))))
        .collect();

    services(&mut compose)
        .zip(iter::repeat(Rc::new(volume_has_options)))
        .map(move |(result, volume_has_options)| {
            let (name, service) = result?;
            service_try_into_quadlet_file(
                ComposeService {
                    service,
                    volume_has_options,
                },
                name,
                unit.cloned(),
                install.cloned(),
            )
        })
        .chain(networks_try_into_quadlet_files(
            compose.networks,
            unit,
            install,
        ))
        .chain(volumes_try_into_quadlet_files(
            compose.volumes,
            unit,
            install,
        ))
}

/// Extract an iterator of [`docker_compose_types::Service`] from a [`Compose`]
pub fn services(
    compose: &mut Compose,
) -> impl Iterator<Item = color_eyre::Result<(String, docker_compose_types::Service)>> {
    mem::take(&mut compose.services.0)
        .into_iter()
        .map(|(name, service)| {
            let service_name = name.clone();
            service.map(|service| (name, service)).ok_or_else(|| {
                eyre::eyre!(
                    "Service `{service_name}` does not have any corresponding options; \
                        minimally, `image` is required"
                )
            })
        })
        .chain(
            compose
                .service
                .take()
                .map(|service| Ok((String::from(image_to_name(service.image())), service))),
        )
}

/// Attempt to convert a [`ComposeService`] into a [`quadlet::File`].
fn service_try_into_quadlet_file(
    mut service: ComposeService,
    name: String,
    mut unit: Option<Unit>,
    install: Option<quadlet::Install>,
) -> color_eyre::Result<quadlet::File> {
    // Add any service dependencies the [Unit] section of the quadlet file.
    let depends_on = &mut service.service.depends_on;
    if !depends_on.is_empty() {
        unit.get_or_insert(Unit::default())
            .add_dependencies(mem::take(depends_on));
    }

    let command = PodmanCommands::try_from(service)
        .wrap_err_with(|| format!("Could not parse service `{name}` as a valid podman command"))?;

    let service = command.service().cloned();

    Ok(quadlet::File {
        name,
        unit,
        resource: command.into(),
        globals: Globals::default(),
        service,
        install,
    })
}

/// Attempt to convert [`ComposeNetworks`] into an iterator of [`quadlet::File`].
fn networks_try_into_quadlet_files<'a>(
    networks: ComposeNetworks,
    unit: Option<&'a Unit>,
    install: Option<&'a quadlet::Install>,
) -> impl Iterator<Item = color_eyre::Result<quadlet::File>> + 'a {
    networks.0.into_iter().map(move |(name, network)| {
        let network = Option::<docker_compose_types::NetworkSettings>::from(network)
            .map(quadlet::Network::try_from)
            .transpose()
            .wrap_err_with(|| {
                format!("Could not parse network `{name}` as a valid podman network")
            })?
            .unwrap_or_default();
        Ok(quadlet::File {
            name,
            unit: unit.cloned(),
            resource: network.into(),
            globals: Globals::default(),
            service: None,
            install: install.cloned(),
        })
    })
}

/// Attempt to convert compose volumes into an iterator of [`quadlet::File`].
fn volumes_try_into_quadlet_files<'a>(
    volumes: docker_compose_types::TopLevelVolumes,
    unit: Option<&'a Unit>,
    install: Option<&'a quadlet::Install>,
) -> impl Iterator<Item = color_eyre::Result<quadlet::File>> + 'a {
    volumes.0.into_iter().filter_map(move |(name, volume)| {
        Option::<docker_compose_types::ComposeVolume>::from(volume).map(|volume| {
            let volume = quadlet::Volume::try_from(volume).wrap_err_with(|| {
                format!("could not parse volume `{name}` as a valid podman volume")
            })?;
            Ok(quadlet::File {
                name,
                unit: unit.cloned(),
                resource: volume.into(),
                globals: Globals::default(),
                service: None,
                install: install.cloned(),
            })
        })
    })
}
*/
