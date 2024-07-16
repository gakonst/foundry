use cast::opts::CastArgs;
use clap::Parser;
use eyre::Result;
use foundry_cli::{handler, utils};

#[cfg(all(feature = "jemalloc", unix))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<()> {
    handler::install();
    utils::load_dotenv();
    utils::subscriber();
    utils::enable_paint();

    let args = CastArgs::parse();
    args.cmd.run().await
}
