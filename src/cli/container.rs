mod compose;
mod podman;
mod quadlet;
pub mod security_opt;

use clap::Args;
use color_eyre::eyre::{Context, OptionExt};

use crate::escape::command_join;

use self::{podman::PodmanArgs, quadlet::QuadletOptions, security_opt::SecurityOpt};

use super::image_to_name;

#[allow(clippy::doc_markdown)]
#[derive(Args, Default, Debug, Clone, PartialEq)]
pub struct Container {
    #[command(flatten)]
    quadlet_options: QuadletOptions,

    /// Converts to "PodmanArgs=ARGS"
    #[command(flatten)]
    podman_args: PodmanArgs,

    /// Security options
    ///
    /// Converts to a number of different Quadlet options or,
    /// if a Quadlet option for the specified security option doesn't exist,
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

impl Container {
    /// The name that should be used for the generated [`File`](crate::quadlet::File).
    ///
    /// It is either the set container name or taken from the image.
    pub fn name(&self) -> &str {
        self.quadlet_options
            .name
            .as_deref()
            .unwrap_or_else(|| image_to_name(&self.image))
    }

    /// Set the `--pod` option.
    pub(super) fn set_pod(&mut self, pod: Option<String>) {
        self.podman_args.set_pod(pod);
    }
}

impl TryFrom<compose_spec::Service> for Container {
    type Error = color_eyre::Report;

    fn try_from(value: compose_spec::Service) -> Result<Self, Self::Error> {
        let compose::Service {
            unsupported,
            quadlet,
            podman_args,
            container:
                compose::Container {
                    command,
                    image,
                    security_opt,
                },
        } = compose::Service::from(value);

        unsupported.ensure_empty()?;

        let security_opt = security_opt
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
            quadlet_options: quadlet.try_into()?,
            podman_args: podman_args.try_into()?,
            security_opt,
            image: image.ok_or_eyre("`image` or `build` is required")?.into(),
            command: command
                .map(super::compose::command_try_into_vec)
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
            exec: (!command.is_empty()).then(|| command_join(command)),
            ..quadlet_options.into()
        }
    }
}

impl From<Container> for crate::quadlet::Resource {
    fn from(value: Container) -> Self {
        crate::quadlet::Container::from(value).into()
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
