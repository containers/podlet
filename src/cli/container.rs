mod podman;
mod quadlet;
pub mod security_opt;

use std::mem;

use clap::Args;
use color_eyre::eyre::{self, Context, OptionExt};

use crate::cli::compose;

use self::{podman::PodmanArgs, quadlet::QuadletOptions, security_opt::SecurityOpt};
use super::{image_to_name, ComposeService};

#[derive(Args, Default, Debug, Clone, PartialEq)]
pub struct Container {
    #[command(flatten)]
    quadlet_options: QuadletOptions,

    /// Converts to "PodmanArgs=ARGS"
    #[command(flatten)]
    podman_args: PodmanArgs,

    /// Security options
    ///
    /// Converts to a number of different quadlet options or,
    /// if a quadlet option for the specified security option doesn't exist,
    /// is placed in "PodmanArgs="
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "OPTION")]
    security_opt: Vec<SecurityOpt>,

    /// The image to run in the container
    ///
    /// Converts to "Image=IMAGE"
    image: String,

    /// Optionally, the command to run in the container
    ///
    /// Converts to "Exec=COMMAND..."
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

impl TryFrom<ComposeService> for Container {
    type Error = color_eyre::Report;

    fn try_from(mut value: ComposeService) -> Result<Self, Self::Error> {
        let service = &value.service;
        let unsupported_options = [
            ("deploy", service.deploy.is_none()),
            ("build", service.build_.is_none()),
            ("profiles", service.profiles.is_empty()),
            ("links", service.links.is_empty()),
            ("net", service.net.is_none()),
            ("volumes_from", service.volumes_from.is_empty()),
            ("extends", service.extends.is_empty()),
            ("scale", service.scale == 0),
        ];
        for (option, not_present) in unsupported_options {
            eyre::ensure!(not_present, "`{option}` is unsupported");
        }
        eyre::ensure!(
            service.extensions.is_empty(),
            "compose extensions are not supported"
        );

        let security_opt = mem::take(&mut value.service.security_opt)
            .into_iter()
            .filter_map(|s| {
                if s == "no-new-privileges:true" {
                    Some(Ok(SecurityOpt::NoNewPrivileges))
                } else if s == "no-new-privileges:false" {
                    None
                } else {
                    Some(s.replacen(':', "=", 1).parse())
                }
            })
            .collect::<Result<_, _>>()
            .wrap_err("invalid security option")?;

        Ok(Self {
            quadlet_options: (&mut value).try_into()?,
            podman_args: (&mut value.service).try_into()?,
            security_opt,
            image: value.service.image.ok_or_eyre("image is required")?,
            command: value
                .service
                .command
                .map(compose::command_try_into_vec)
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

impl From<Container> for crate::quadlet::Container {
    fn from(
        Container {
            quadlet_options,
            podman_args,
            security_opt,
            image,
            command,
        }: Container,
    ) -> Self {
        let mut podman_args = podman_args.to_string();

        let security_opt::QuadletOptions {
            mask,
            no_new_privileges,
            seccomp_profile,
            security_label_disable,
            security_label_file_type,
            security_label_level,
            security_label_nested,
            security_label_type,
            unmask,
            podman_args: security_podman_args,
        } = security_opt.into_iter().fold(
            security_opt::QuadletOptions::default(),
            |mut security_options, security_opt| {
                security_options.add_security_opt(security_opt);
                security_options
            },
        );

        for arg in security_podman_args {
            podman_args.push_str(" --security-opt ");
            podman_args.push_str(&arg);
        }

        Self {
            image,
            mask,
            no_new_privileges,
            seccomp_profile,
            security_label_disable,
            security_label_file_type,
            security_label_level,
            security_label_nested,
            security_label_type,
            unmask,
            podman_args: (!podman_args.is_empty()).then(|| podman_args.trim().to_string()),
            exec: (!command.is_empty()).then(|| shlex::join(command.iter().map(String::as_str))),
            ..quadlet_options.into()
        }
    }
}

impl From<Container> for crate::quadlet::Resource {
    fn from(value: Container) -> Self {
        crate::quadlet::Container::from(value).into()
    }
}

impl Container {
    pub fn name(&self) -> &str {
        self.quadlet_options
            .name
            .as_deref()
            .unwrap_or_else(|| image_to_name(&self.image))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod name {
        use super::*;

        #[test]
        fn container_name() {
            let name = "test";
            let mut sut = Container::default();
            sut.quadlet_options.name = Some(String::from(name));

            assert_eq!(sut.name(), name);
        }

        #[test]
        fn image_no_tag() {
            let sut = Container {
                image: String::from("quay.io/podman/hello"),
                ..Default::default()
            };
            assert_eq!(sut.name(), "hello");
        }

        #[test]
        fn image_with_tag() {
            let sut = Container {
                image: String::from("quay.io/podman/hello:latest"),
                ..Default::default()
            };
            assert_eq!(sut.name(), "hello");
        }
    }
}
