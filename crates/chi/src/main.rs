mod collect;
mod invert;
mod replay;

use anyhow::Result;
use clap::Parser;
use collect::CollectArgs;
use invert::InvertArgs;
use replay::ReplayArgs;

#[derive(Parser)]
#[command(name = "chi")]
enum Cmd {
    Collect(CollectArgs),
    Invert(InvertArgs),
    Replay(ReplayArgs),
}

fn main() -> Result<()> {
    match Cmd::parse() {
        Cmd::Collect(args) => collect::run(args),
        Cmd::Invert(args) => invert::run(args),
        Cmd::Replay(args) => replay::run(args),
    }
}
