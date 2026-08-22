mod collect;
mod invert;

use anyhow::Result;
use clap::Parser;
use collect::CollectArgs;
use invert::InvertArgs;

#[derive(Parser)]
#[command(name = "chi")]
enum Cmd {
    Collect(CollectArgs),
    Invert(InvertArgs),
}

fn main() -> Result<()> {
    match Cmd::parse() {
        Cmd::Collect(args) => collect::run(args),
        Cmd::Invert(args) => invert::run(args),
    }
}
