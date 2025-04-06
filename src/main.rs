use anyhow::Result;
use src2md::{cli::parse_args, run_src2md};

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args()?;
    run_src2md(config).await
}
