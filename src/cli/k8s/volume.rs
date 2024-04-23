//! Utilities for converting a compose [`Volume`] into a Kubernetes [`PersistentVolumeClaim`].

use color_eyre::eyre::{bail, ensure, Context};
use compose_spec::{Identifier, MapKey, Number, StringOrNumber, Volume};
use indexmap::IndexMap;
use k8s_openapi::{
    api::core::v1::PersistentVolumeClaim, apimachinery::pkg::apis::meta::v1::ObjectMeta,
};

/// Attempt to convert a compose [`Volume`] into a [`PersistentVolumeClaim`].
///
/// # Errors
///
/// Returns an error if the [`Volume`] has unsupported options set or there was an error converting
/// an option.
pub(super) fn try_into_persistent_volume_claim(
    name: Identifier,
    Volume {
        driver,
        driver_opts,
        labels,
        name: volume_name,
        extensions,
    }: Volume,
) -> color_eyre::Result<PersistentVolumeClaim> {
    ensure!(volume_name.is_none(), "`name` is not supported");
    ensure!(
        extensions.is_empty(),
        "compose extensions are not supported"
    );

    Ok(PersistentVolumeClaim {
        metadata: ObjectMeta {
            name: Some(name.into()),
            annotations: (driver.is_some() || !driver_opts.is_empty())
                .then(|| {
                    DriverOpts::try_from_compose(driver, driver_opts)
                        .map(|driver_opts| driver_opts.into_annotations().collect())
                })
                .transpose()
                .wrap_err("error converting `driver_opts`")?,
            labels: (!labels.is_empty())
                .then(|| {
                    labels.into_map().map(|labels| {
                        labels
                            .into_iter()
                            .map(|(key, value)| {
                                (key.into(), value.map(Into::into).unwrap_or_default())
                            })
                            .collect()
                    })
                })
                .transpose()
                .wrap_err("error converting `labels`")?,
            ..ObjectMeta::default()
        },
        spec: None,
        status: None,
    })
}

/// Supported volume driver options for a [`PersistentVolumeClaim`] through the use of Kubernetes
/// annotations.
///
/// See the "Kubernetes Persistent Volume Claims" section of the docs for
/// [**podman-kube-play**(1)](https://docs.podman.io/en/stable/markdown/podman-kube-play.1.html).
#[derive(Debug, Default)]
struct DriverOpts {
    driver: Option<String>,
    device: Option<String>,
    fs_type: Option<String>,
    uid: Option<u32>,
    gid: Option<u32>,
    mount_options: Option<String>,
    import_source: Option<String>,
    image: Option<String>,
}

impl DriverOpts {
    /// Attempt to create [`DriverOpts`] from a [`compose_spec::Volume`]'s `driver` and
    /// `driver_opts` fields.
    fn try_from_compose(
        driver: Option<String>,
        driver_opts: IndexMap<MapKey, StringOrNumber>,
    ) -> color_eyre::Result<Self> {
        driver_opts.into_iter().try_fold(
            Self {
                driver,
                ..Self::default()
            },
            |mut driver_opts, (key, value)| {
                driver_opts.parse_add(key.as_str(), value)?;
                Ok(driver_opts)
            },
        )
    }

    /// Parse `key` as a driver option and add `value` to `self` as appropriate.
    ///
    /// # Errors
    ///
    /// Returns an error if the `key` is an unknown option or there is an error converting the
    /// `value`.
    fn parse_add(&mut self, key: &str, value: StringOrNumber) -> color_eyre::Result<()> {
        match key {
            "device" => self.device = Some(value.into()),
            "type" => self.fs_type = Some(value.into()),
            "uid" => {
                let StringOrNumber::Number(Number::UnsignedInt(uid)) = value else {
                    bail!("`uid` must be a positive integer");
                };
                self.uid = uid
                    .try_into()
                    .map(Some)
                    .wrap_err_with(|| format!("UID `{uid}` is too large"))?;
            }
            "gid" => {
                let StringOrNumber::Number(Number::UnsignedInt(gid)) = value else {
                    bail!("`gid` must be a positive integer");
                };
                self.gid = gid
                    .try_into()
                    .map(Some)
                    .wrap_err_with(|| format!("GID `{gid}` is too large"))?;
            }
            "import-source" => self.import_source = Some(value.into()),
            "image" => self.image = Some(value.into()),
            "o" => {
                let StringOrNumber::String(mount_options) = value else {
                    bail!("`o` value must be a string");
                };
                self.add_mount_options(&mount_options)?;
            }
            key => bail!("unknown volume driver option `{key}`"),
        }
        Ok(())
    }

    /// Add the `mount_options` to `self`.
    ///
    /// # Errors
    ///
    /// Returns an error if a `uid=` or `gid=` mount option value could not be parsed as a [`u32`].
    fn add_mount_options(&mut self, mount_options: &str) -> color_eyre::Result<()> {
        for mount_option in mount_options.split(',') {
            if let Some(uid) = mount_option.strip_prefix("uid=") {
                self.uid = uid.parse().map(Some).wrap_err_with(|| {
                    format!("error parsing UID `{uid}` as an unsigned integer")
                })?;
            } else if let Some(gid) = mount_option.strip_prefix("gid=") {
                self.gid = gid.parse().map(Some).wrap_err_with(|| {
                    format!("error parsing GID `{gid}` as an unsigned integer")
                })?;
            } else if let Some(mount_options) = &mut self.mount_options {
                mount_options.push(',');
                mount_options.push_str(mount_option);
            } else {
                self.mount_options = Some(mount_option.to_owned());
            }
        }
        Ok(())
    }

    /// Convert driver options into an [`Iterator`] of key-value pairs for use as
    /// [`PersistentVolumeClaim`] annotations.
    fn into_annotations(self) -> impl Iterator<Item = (String, String)> {
        let Self {
            driver,
            device,
            fs_type,
            uid,
            gid,
            mount_options,
            import_source,
            image,
        } = self;

        [
            ("volume.podman.io/driver", driver),
            ("volume.podman.io/device", device),
            ("volume.podman.io/type", fs_type),
            ("volume.podman.io/uid", uid.as_ref().map(u32::to_string)),
            ("volume.podman.io/gid", gid.as_ref().map(u32::to_string)),
            ("volume.podman.io/mount-options", mount_options),
            ("volume.podman.io/import-source", import_source),
            ("volume.podman.io/image", image),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key.to_owned(), value)))
    }
}
