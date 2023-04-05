mod podman;
mod quadlet;
mod security_opt;
pub mod user_namespace;

use std::fmt::{self, Display, Formatter};

use clap::Args;

use self::security_opt::SecurityOpt;

#[derive(Args, Default, Debug, Clone, PartialEq)]
pub struct Container {
    #[command(flatten)]
    quadlet_options: quadlet::QuadletOptions,

    /// Converts to "PodmanArgs=ARGS"
    #[command(flatten)]
    podman_args: podman::PodmanArgs,

    /// Set the user namespace mode for the container
    #[arg(long, value_name = "MODE")]
    userns: Option<user_namespace::Mode>,

    /// Security options
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

fn map_arg_output<'a, T, U>(iter: T, arg: &'a str) -> impl Iterator<Item = (&'a str, Output)>
where
    T: IntoIterator<Item = &'a U>,
    Output: From<&'a U>,
    U: 'a,
{
    iter.into_iter().map(move |item| (arg, Output::from(item)))
}

impl Display for Container {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "[Container]")?;
        writeln!(f, "Image={}", self.image)?;

        write!(f, "{}", self.quadlet_options)?;

        let mut podman_args = self.podman_args.to_string();

        let userns = map_arg_output(&self.userns, "--userns");
        let security_opt = map_arg_output(&self.security_opt, "--security-opt");
        let outputs = userns.chain(security_opt);
        for (arg, output) in outputs {
            output.write_or_add_arg(arg, f, &mut podman_args)?;
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
                .expect("Split will has at least one element");
            // Remove image tag
            image.split_once(':').map_or(image, |(name, _)| name)
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    QuadletOptions(String),
    PodmanArg(String),
}

impl Output {
    pub fn write_or_add_arg(
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
