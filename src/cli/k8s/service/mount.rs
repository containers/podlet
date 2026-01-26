//! Utilities for converting a volume [`Mount`] from a [`compose_spec::Service`] into a Kubernetes
//! [`VolumeMount`] and [`Volume`] for a [`Container`](k8s_openapi::api::core::v1::Container) and
//! its [`PodSpec`](k8s_openapi::api::core::v1::PodSpec).

use color_eyre::eyre::{WrapErr, ensure, eyre};
use compose_spec::{
    Identifier, ItemOrList,
    service::{
        AbsolutePath, Volumes,
        volumes::{
            self, Mount,
            mount::{self, Bind, BindOptions, Common, Tmpfs, TmpfsOptions, VolumeOptions},
        },
    },
};
use k8s_openapi::{
    api::core::v1::{
        EmptyDirVolumeSource, HostPathVolumeSource, PersistentVolumeClaimVolumeSource, Volume,
        VolumeMount,
    },
    apimachinery::pkg::api::resource::Quantity,
};

/// Attempt to convert the `tmpfs` and `volumes` fields from a [`compose_spec::Service`] into
/// [`VolumeMount`]s.
///
/// The corresponding [`Volume`]s are added to `pod_volumes`.
///
/// # Errors
///
/// Returns an error if an unsupported option is present or the [`Mount`] type is not supported.
pub(super) fn tmpfs_and_volumes_try_into_volume_mounts(
    tmpfs: Option<ItemOrList<AbsolutePath>>,
    volumes: Volumes,
    container_name: &Identifier,
    pod_volumes: &mut Option<Vec<Volume>>,
) -> color_eyre::Result<Vec<VolumeMount>> {
    tmpfs
        .into_iter()
        .flat_map(ItemOrList::into_list)
        .map(Tmpfs::from_target)
        .map(Into::into)
        .chain(volumes::into_long_iter(volumes))
        .map(|mount| {
            let (volume_mount, volume) = try_into_volume_mount(mount, container_name)?;
            pod_volumes.get_or_insert_with(Vec::new).push(volume);
            Ok(volume_mount)
        })
        .collect()
}

/// Attempt to convert a volume [`Mount`] from a [`compose_spec::Service`] into a [`VolumeMount`]
/// and its corresponding [`Volume`].
///
/// # Errors
///
/// Returns an error if an unsupported option is present or the [`Mount`] type is not supported.
fn try_into_volume_mount(
    mount: Mount,
    container_name: &Identifier,
) -> color_eyre::Result<(VolumeMount, Volume)> {
    match mount {
        Mount::Volume(volume) => volume_try_into_volume_mount(volume, container_name)
            .wrap_err("error converting `volume` type volume mount"),
        Mount::Bind(bind) => bind_try_into_volume_mount(bind, container_name)
            .wrap_err("error converting `bind` type volume mount"),
        Mount::Tmpfs(tmpfs) => tmpfs_try_into_volume_mount(tmpfs, container_name)
            .wrap_err("error converting `tmpfs` type volume mount"),
        Mount::NamedPipe(_) => Err(eyre!("`npipe` volume mount type is not supported")),
        Mount::Cluster(_) => Err(eyre!("`cluster` volume mount type is not supported")),
    }
}

/// Attempt to convert a [`mount::Volume`] into a [`VolumeMount`].
///
/// # Errors
///
/// Returns an error if an unsupported option is present.
fn volume_try_into_volume_mount(
    mount::Volume {
        source,
        volume,
        common,
    }: mount::Volume,
    container_name: &Identifier,
) -> color_eyre::Result<(VolumeMount, Volume)> {
    ensure!(
        volume.as_ref().is_none_or(VolumeOptions::is_empty),
        "additional `volume` options are not supported"
    );

    let anonymous_volume = source.is_none();
    let source = source.map_or(Source::Other { container_name }, Source::Volume);
    let volume_mount = common_try_into_volume_mount(common, source)?;

    let name = volume_mount.name.clone();
    let volume = if anonymous_volume {
        Volume {
            name,
            empty_dir: Some(EmptyDirVolumeSource::default()),
            ..Volume::default()
        }
    } else {
        Volume {
            name: name.clone(),
            persistent_volume_claim: Some(PersistentVolumeClaimVolumeSource {
                claim_name: name,
                read_only: None,
            }),
            ..Volume::default()
        }
    };

    Ok((volume_mount, volume))
}

/// Attempt to convert a [`Bind`] volume [`Mount`] into a [`VolumeMount`].
///
/// # Errors
///
/// Returns an error if an unsupported option is present.
fn bind_try_into_volume_mount(
    Bind {
        source,
        bind,
        common,
    }: Bind,
    container_name: &Identifier,
) -> color_eyre::Result<(VolumeMount, Volume)> {
    let BindOptions {
        propagation,
        create_host_path,
        selinux,
        extensions,
    } = bind.unwrap_or_default();

    ensure!(propagation.is_none(), "`bind.propagation` is not supported");
    ensure!(
        create_host_path,
        "`bind.create_host_path: false` is not supported"
    );
    ensure!(
        extensions.is_empty(),
        "compose extensions are not supported"
    );

    let mut volume_mount = common_try_into_volume_mount(common, Source::Other { container_name })?;
    if let Some(selinux) = selinux {
        let mount_path = &mut volume_mount.mount_path;
        mount_path.push(':');
        mount_path.push(selinux.as_char());
    }

    let volume = Volume {
        name: volume_mount.name.clone(),
        host_path: Some(HostPathVolumeSource {
            path: source
                .into_inner()
                .into_os_string()
                .into_string()
                .map_err(|_| eyre!("`source` must only contain valid UTF-8"))?,
            type_: None,
        }),
        ..Volume::default()
    };

    Ok((volume_mount, volume))
}

/// Attempt to convert a [`Tmpfs`] volume [`Mount`] into a [`VolumeMount`].
///
/// # Errors
///
/// Returns an error if an unsupported option is present.
fn tmpfs_try_into_volume_mount(
    Tmpfs { tmpfs, common }: Tmpfs,
    container_name: &Identifier,
) -> color_eyre::Result<(VolumeMount, Volume)> {
    let TmpfsOptions {
        size,
        mode,
        extensions,
    } = tmpfs.unwrap_or_default();

    ensure!(mode.is_none(), "`tmpfs.mode` is not supported");
    ensure!(
        extensions.is_empty(),
        "compose extensions are not supported"
    );

    let volume_mount = common_try_into_volume_mount(common, Source::Other { container_name })?;

    let volume = Volume {
        name: volume_mount.name.clone(),
        empty_dir: Some(EmptyDirVolumeSource {
            medium: Some("Memory".to_owned()),
            size_limit: size.map(|size| Quantity(size.to_string())),
        }),
        ..Volume::default()
    };

    Ok((volume_mount, volume))
}

/// Attempt to convert [`Common`] volume [`Mount`] options into a [`VolumeMount`].
///
/// `source` is used to create the [`VolumeMount`]'s `name`.
///
/// # Errors
///
/// Returns an error if an unsupported [`Common`] option is present.
fn common_try_into_volume_mount(
    Common {
        target,
        read_only,
        consistency,
        extensions,
    }: Common,
    source: Source,
) -> color_eyre::Result<VolumeMount> {
    ensure!(consistency.is_none(), "`consistency` is not supported");
    ensure!(
        extensions.is_empty(),
        "compose extensions are not supported"
    );

    let mount_path = target
        .into_inner()
        .into_os_string()
        .into_string()
        .map_err(|_| eyre!("`target` must only contain valid UTF-8"))?;

    let name = source.into_volume_name(&mount_path);

    Ok(VolumeMount {
        mount_path,
        name,
        read_only: read_only.then_some(true),
        ..VolumeMount::default()
    })
}

/// Source for a [`VolumeMount`].
enum Source<'a> {
    /// Source is a [`Volume`] with a [`PersistentVolumeClaimVolumeSource`].
    Volume(Identifier),
    /// Source is a [`Volume`] with some other source type.
    Other { container_name: &'a Identifier },
}

impl Source<'_> {
    /// Convert source into a `name` for a [`Volume`].
    ///
    /// If [`Other`](Self::Other), the `container_name` is combined with the `mount_path` to create
    /// the `name`.
    fn into_volume_name(self, mount_path: &str) -> String {
        match self {
            Self::Volume(volume) => volume.into(),
            Self::Other { container_name } => {
                format!("{container_name}{}", mount_path.replace(['/', '\\'], "-"))
            }
        }
    }
}
