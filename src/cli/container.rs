mod podman;
mod quadlet;
mod security_opt;

use std::fmt::{self, Display, Formatter};

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

impl Display for Container {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "[Container]")?;
        writeln!(f, "Image={}", self.image)?;

        write!(f, "{}", self.quadlet_options)?;

        let mut podman_args = self.podman_args.to_string();

        for output in self.security_opt.iter().map(Output::from) {
            output.write_or_add_arg("--security-opt", f, &mut podman_args)?;
        }

        if !podman_args.is_empty() {
            writeln!(f, "PodmanArgs={}", podman_args.trim())?;
        }

        if !self.command.is_empty() {
            let command = shlex::join(self.command.iter().map(String::as_str));
            writeln!(f, "Exec={command}")?;
        }

        Ok(())
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

#[derive(Debug, Clone, PartialEq)]
enum Output {
    QuadletOptions(String),
    PodmanArg(String),
}

impl Output {
    fn write_or_add_arg(
        &self,
        arg: &str,
        f: &mut Formatter,
        args: &mut String,
    ) -> Result<(), fmt::Error> {
        match self {
            Output::QuadletOptions(options) => writeln!(f, "{options}"),
            Output::PodmanArg(arg_value) => {
                *args += &format!(" {arg} {arg_value}");
                Ok(())
            }
        }
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
