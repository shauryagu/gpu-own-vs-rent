use anyhow::{bail, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct InvertArgs {}

pub fn run(_args: InvertArgs) -> Result<()> {
    bail!("not implemented")
}
