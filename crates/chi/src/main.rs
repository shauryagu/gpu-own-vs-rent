mod collect;
mod invert;
mod replay;
mod stack;

use anyhow::Result;
use clap::Parser;
use collect::CollectArgs;
use invert::InvertArgs;
use replay::ReplayArgs;
use stack::StackArgs;

#[derive(Parser)]
#[command(name = "chi")]
enum Cmd {
    /// Collect free public OCPI and Epoch snapshots
    Collect(CollectArgs),
    /// Teaching inverse: leftover L and salvage R* from declared θ
    Invert(InvertArgs),
    /// Fold a chi_log directory into a catalog (replay only)
    Replay(ReplayArgs),
    /// Cost-stack panel: S = F_capital + e + L at named π
    Stack(StackArgs),
}

fn main() -> Result<()> {
    match Cmd::parse() {
        Cmd::Collect(args) => collect::run(args),
        Cmd::Invert(args) => invert::run(args),
        Cmd::Replay(args) => replay::run(args),
        Cmd::Stack(args) => stack::run(args),
    }
}
