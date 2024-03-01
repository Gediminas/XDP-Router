mod build_ebpf;
mod run;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Options {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    BuildEbpf(build_ebpf::Options),
    Run(run::Options),
}

const XDP_PROGRAM: &str = "router-xdp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Options::parse().command {
        Command::BuildEbpf(bo) => build_ebpf::build_ebpf(bo, XDP_PROGRAM)?,
        Command::Run(ro) => run::run(ro, XDP_PROGRAM)?,
    }
    Ok(())
}
