mod podman;
mod quadlet;
mod security_opt;

use clap::Args;

use self::{podman::PodmanArgs, quadlet::QuadletOptions, security_opt::SecurityOpt};

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
        self.quadlet_options.name.as_deref().unwrap_or_else(|| {
            let image = self
                .image
                .rsplit('/')
                .next()
                .expect("Split will have at least one element");
            // Remove image tag
            image.split_once(':').map_or(image, |(name, _)| name)
        })
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
