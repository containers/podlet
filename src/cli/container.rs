use clap::Args;

#[derive(Args, Debug, Clone, PartialEq)]
pub struct QuadletOptions {
    #[arg(
        short,
        long,
        value_name = "[[SOURCE-VOLUME|HOST-DIR:]CONTAINER-DIR[:OPTIONS]]"
    )]
    volume: Vec<String>,
}

#[derive(Args, Debug, Clone, PartialEq)]
pub struct PodmanArgs {
    #[arg(long, value_name = "host:ip")]
    add_host: Vec<String>,
}
