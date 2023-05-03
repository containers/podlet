mod podman;
mod quadlet;
mod security_opt;

use clap::Args;
use color_eyre::eyre;

use self::{podman::PodmanArgs, quadlet::QuadletOptions, security_opt::SecurityOpt};
use super::image_to_name;

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

impl TryFrom<docker_compose_types::Service> for Container {
    type Error = color_eyre::Report;

    fn try_from(mut value: docker_compose_types::Service) -> Result<Self, Self::Error> {
        let unsupported_options = [
            ("deploy", value.deploy.is_some()),
            ("build", value.build_.is_some()),
            ("profiles", value.profiles.is_some()),
            ("links", value.links.is_some()),
            ("net", value.net.is_some()),
            ("volumes_from", !value.volumes_from.is_empty()),
            ("extends", value.extends.is_some()),
            ("scale", value.scale != 0),
        ];
        for (option, exists) in unsupported_options {
            if exists {
                return Err(unsupported_option(option));
            }
        }
        if !value.extensions.is_empty() {
            return Err(eyre::eyre!("compose extensions are not supported"));
        }

        Ok(Self {
            quadlet_options: (&mut value).try_into()?,
            podman_args: (&mut value).try_into()?,
            security_opt: Vec::new(),
            image: value.image.ok_or(eyre::eyre!("image is required"))?,
            command: value
                .command
                .map(|command| match command {
                    docker_compose_types::Command::Simple(s) => vec![s],
                    docker_compose_types::Command::Args(args) => args,
                })
                .unwrap_or_default(),
        })
    }
}

fn unsupported_option(option: &str) -> color_eyre::Report {
    eyre::eyre!("`{option}` is unsupported")
}

impl From<Container> for crate::quadlet::Container {
    fn from(value: Container) -> Self {
        let mut podman_args = value.podman_args.to_string();

        let mut security_options = security_opt::QuadletOptions::default();
        for security_opt in value.security_opt {
            security_options.add_security_opt(security_opt);
        }
        for arg in security_options.podman_args {
            podman_args += &format!(" --security-opt {arg}");
        }

        Self {
            image: value.image,
            no_new_privileges: security_options.no_new_privileges,
            seccomp_profile: security_options.seccomp_profile,
            security_label_disable: security_options.security_label_disable,
            security_label_file_type: security_options.security_label_file_type,
            security_label_level: security_options.security_label_level,
            security_label_type: security_options.security_label_type,
            podman_args: (!podman_args.is_empty()).then(|| podman_args.trim().to_string()),
            exec: (!value.command.is_empty())
                .then(|| shlex::join(value.command.iter().map(String::as_str))),
            ..value.quadlet_options.into()
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
