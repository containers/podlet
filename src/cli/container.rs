use std::fmt::Display;

use clap::Args;

mod podman;
mod quadlet;
mod user_namespace;

#[derive(Args, Debug, Clone, PartialEq)]
pub struct Container {
    #[command(flatten)]
    quadlet_options: quadlet::QuadletOptions,

    /// Converts to "PodmanArgs=ARGS"
    #[command(flatten)]
    podman_args: podman::PodmanArgs,

    /// Set the user namespace mode for the container
    #[arg(long, value_name = "MODE")]
    userns: Option<user_namespace::Mode>,

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[Container]")?;
        writeln!(f, "Image={}", self.image)?;
        write!(f, "{}", self.quadlet_options)?;
        let mut podman_args = self.podman_args.to_string();
        let userns = self.userns.as_ref().map(user_namespace::Mode::to_output);
        if let Some(user_namespace::Output::QuadletOptions(options)) = &userns {
            writeln!(f, "{options}")?;
        }
        if let Some(user_namespace::Output::PodmanArg(arg)) = &userns {
            podman_args += &format!(
                "{}--userns {arg}",
                if podman_args.is_empty() { "" } else { " " }
            );
        }
        if !podman_args.is_empty() {
            writeln!(f, "PodmanArgs={podman_args}")?;
        }
        if !self.command.is_empty() {
            let command = shlex::join(self.command.iter().map(String::as_str));
            writeln!(f, "Exec={command}")?;
        }
        Ok(())
    }
}
