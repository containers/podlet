use std::{
    collections::HashMap,
    fs::File,
    io::{self, IsTerminal},
    mem,
    path::Path,
};

use color_eyre::{
    eyre::{bail, eyre, OptionExt, WrapErr},
    Help,
};
use compose_spec::{Compose, Identifier, Network, Networks, Resource, Service, Volumes};

use crate::quadlet::{self, container::volume::Source, Globals};

use super::{Container, GlobalArgs, Unit};

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
        bail!("cannot read compose from stdin, stdin is a terminal");
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
*/

/// Attempt to convert a [`Compose`] file into an [`Iterator`] of [`quadlet::File`]s.
///
/// # Errors
///
/// The [`Iterator`] returns an [`Err`] if a [`Service`], [`Network`], or
/// [`Volume`](compose_spec::Volume) could not be converted into a [`quadlet::File`].
pub fn try_into_quadlet_files<'a>(
    compose: Compose,
    unit: Option<&'a Unit>,
    install: Option<&'a quadlet::Install>,
) -> impl Iterator<Item = color_eyre::Result<quadlet::File>> + 'a {
    // Get a map of volumes to whether the volume has options associated with it for use in
    // converting a service into a quadlet file. Extra volume options must be specified in a
    // separate quadlet file which is referenced from the container quadlet file.
    let volume_has_options = compose
        .volumes
        .iter()
        .map(|(name, volume)| {
            let has_options = volume
                .as_ref()
                .and_then(Resource::as_compose)
                .is_some_and(|volume| !volume.is_empty());
            (name.clone(), has_options)
        })
        .collect();

    compose
        .services
        .into_iter()
        .map(move |(name, service)| {
            service_try_into_quadlet_file(
                service,
                name,
                unit.cloned(),
                install.cloned(),
                &volume_has_options,
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

/// Attempt to convert a compose [`Service`] into a [`quadlet::File`].
///
/// `volume_has_options` should be a map from volume [`Identifier`]s to whether the volume has any
/// options set. It is used to determine whether to link to a [`quadlet::Volume`] in the created
/// [`quadlet::Container`].
///
/// # Errors
///
/// Returns an error if there was an error [adding](Unit::add_dependency()) a service
/// [`Dependency`](compose_spec::service::Dependency) to the [`Unit`] or converting the [`Service`]
/// into a [`quadlet::Container`].
fn service_try_into_quadlet_file(
    mut service: Service,
    name: Identifier,
    mut unit: Option<Unit>,
    install: Option<quadlet::Install>,
    volume_has_options: &HashMap<Identifier, bool>,
) -> color_eyre::Result<quadlet::File> {
    // Add any service dependencies to the [Unit] section of the quadlet file.
    let dependencies = mem::take(&mut service.depends_on).into_long();
    if !dependencies.is_empty() {
        let unit = unit.get_or_insert_with(Unit::default);
        for (ident, dependency) in dependencies {
            unit.add_dependency(&ident, dependency).wrap_err_with(|| {
                format!("error adding dependency on `{ident}` to service `{name}`")
            })?;
        }
    }

    let global_args = GlobalArgs::from_compose(&mut service);

    let restart = service.restart;

    let mut container = Container::try_from(service)
        .map(quadlet::Container::from)
        .wrap_err_with(|| format!("error converting service `{name}` into a quadlet container"))?;

    // For each named volume, check to see if it has any options set.
    // If it does, add `.volume` to the source to link this `.container` file to the generated
    // `.volume` file.
    for volume in &mut container.volume {
        if let Some(Source::NamedVolume(source)) = &mut volume.source {
            let volume_has_options = volume_has_options
                .get(source.as_str())
                .copied()
                .unwrap_or_default();
            if volume_has_options {
                source.push_str(".volume");
            }
        }
    }

    Ok(quadlet::File {
        name: name.into(),
        unit,
        resource: container.into(),
        globals: global_args.into(),
        service: restart.map(Into::into),
        install,
    })
}

/// Attempt to convert compose [`Networks`] into an [`Iterator`] of [`quadlet::File`]s.
///
/// # Errors
///
/// The [`Iterator`] returns an [`Err`] if a [`Network`] could not be converted into a
/// [`quadlet::Network`].
fn networks_try_into_quadlet_files<'a>(
    networks: Networks,
    unit: Option<&'a Unit>,
    install: Option<&'a quadlet::Install>,
) -> impl Iterator<Item = color_eyre::Result<quadlet::File>> + 'a {
    networks.into_iter().map(move |(name, network)| {
        let network = match network {
            Some(Resource::Compose(network)) => network,
            None => Network::default(),
            Some(Resource::External { .. }) => {
                bail!("external networks (`{name}`) are not supported");
            }
        };
        let network = quadlet::Network::try_from(network).wrap_err_with(|| {
            format!("error converting network `{name}` into a quadlet network")
        })?;

        Ok(quadlet::File {
            name: name.into(),
            unit: unit.cloned(),
            resource: network.into(),
            globals: Globals::default(),
            service: None,
            install: install.cloned(),
        })
    })
}

/// Attempt to convert compose [`Volumes`] into an [`Iterator`] of [`quadlet::File`]s.
///
/// [`Volume`](compose_spec::Volume)s which are [empty](compose_spec::Volume::is_empty()) are
/// filtered out as they do not need a `.volume` quadlet file to define extra options.
///
/// # Errors
///
/// The [`Iterator`] returns an [`Err`] if a [`Volume`](compose_spec::Volume) could not be converted
/// to a [`quadlet::Volume`].
fn volumes_try_into_quadlet_files<'a>(
    volumes: Volumes,
    unit: Option<&'a Unit>,
    install: Option<&'a quadlet::Install>,
) -> impl Iterator<Item = color_eyre::Result<quadlet::File>> + 'a {
    volumes.into_iter().filter_map(move |(name, volume)| {
        volume.and_then(|volume| match volume {
            Resource::Compose(volume) => (!volume.is_empty()).then(|| {
                quadlet::Volume::try_from(volume)
                    .wrap_err_with(|| {
                        format!("error converting volume `{name}` into a quadlet volume")
                    })
                    .map(|volume| quadlet::File {
                        name: name.into(),
                        unit: unit.cloned(),
                        resource: volume.into(),
                        globals: Globals::default(),
                        service: None,
                        install: install.cloned(),
                    })
            }),
            Resource::External { .. } => {
                Some(Err(eyre!("external volumes (`{name}`) are not supported")))
            }
        })
    })
}
