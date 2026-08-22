use anyhow::{bail, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct CollectArgs {}

pub fn run(_args: CollectArgs) -> Result<()> {
    bail!("not implemented")
}
