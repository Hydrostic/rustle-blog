use clap::Parser;
#[derive(Debug, Parser)]
#[command(name = "rustle backend")]
pub struct Arg {
    #[arg(long, default_value_t = false)]
    pub debug: bool
}