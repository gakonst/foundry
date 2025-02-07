use clap::{CommandFactory, Parser};
use clap_complete::generate;
use eyre::Result;
use foundry_cli::{handler, utils};
use foundry_common::shell;
use foundry_evm::inspectors::cheatcodes::{set_execution_context, ForgeContext};

mod cmd;
use cmd::{cache::CacheSubcommands, generate::GenerateSubcommands, watch};

mod opts;
use opts::{Forge, ForgeSubcommand};

#[macro_use]
extern crate foundry_common;

#[macro_use]
extern crate tracing;

#[cfg(all(feature = "jemalloc", unix))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    if let Err(err) = run() {
        let _ = foundry_common::sh_err!("{err:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    handler::install();
    utils::load_dotenv();
    utils::subscriber();
    utils::enable_paint();

    let args = Forge::parse();
    args.global.init()?;
    init_execution_context(&args.cmd);

    match args.cmd {
        ForgeSubcommand::Test(cmd) => {
            if cmd.is_watch() {
                utils::block_on(watch::watch_test(cmd))
            } else {
                let silent = cmd.junit || shell::is_json();
                let outcome = utils::block_on(cmd.run())?;
                outcome.ensure_ok(silent)
            }
        }
        ForgeSubcommand::Script(cmd) => utils::block_on(cmd.run_script()),
        ForgeSubcommand::Coverage(cmd) => {
            if cmd.is_watch() {
                utils::block_on(watch::watch_coverage(cmd))
            } else {
                utils::block_on(cmd.run())
            }
        }
        ForgeSubcommand::Bind(cmd) => cmd.run(),
        ForgeSubcommand::Build(cmd) => {
            if cmd.is_watch() {
                utils::block_on(watch::watch_build(cmd))
            } else {
                cmd.run().map(drop)
            }
        }
        ForgeSubcommand::VerifyContract(args) => utils::block_on(args.run()),
        ForgeSubcommand::VerifyCheck(args) => utils::block_on(args.run()),
        ForgeSubcommand::VerifyBytecode(cmd) => utils::block_on(cmd.run()),
        ForgeSubcommand::Clone(cmd) => utils::block_on(cmd.run()),
        ForgeSubcommand::Cache(cmd) => match cmd.sub {
            CacheSubcommands::Clean(cmd) => cmd.run(),
            CacheSubcommands::Ls(cmd) => cmd.run(),
        },
        ForgeSubcommand::Create(cmd) => utils::block_on(cmd.run()),
        ForgeSubcommand::Update(cmd) => cmd.run(),
        ForgeSubcommand::Install(cmd) => cmd.run(),
        ForgeSubcommand::Remove(cmd) => cmd.run(),
        ForgeSubcommand::Remappings(cmd) => cmd.run(),
        ForgeSubcommand::Init(cmd) => cmd.run(),
        ForgeSubcommand::Completions { shell } => {
            generate(shell, &mut Forge::command(), "forge", &mut std::io::stdout());
            Ok(())
        }
        ForgeSubcommand::GenerateFigSpec => {
            clap_complete::generate(
                clap_complete_fig::Fig,
                &mut Forge::command(),
                "forge",
                &mut std::io::stdout(),
            );
            Ok(())
        }
        ForgeSubcommand::Clean { root } => {
            let config = utils::load_config_with_root(root.as_deref())?;
            let project = config.project()?;
            config.cleanup(&project)?;
            Ok(())
        }
        ForgeSubcommand::Snapshot(cmd) => {
            if cmd.is_watch() {
                utils::block_on(watch::watch_gas_snapshot(cmd))
            } else {
                utils::block_on(cmd.run())
            }
        }
        ForgeSubcommand::Fmt(cmd) => {
            if cmd.is_watch() {
                utils::block_on(watch::watch_fmt(cmd))
            } else {
                cmd.run()
            }
        }
        ForgeSubcommand::Config(cmd) => cmd.run(),
        ForgeSubcommand::Flatten(cmd) => cmd.run(),
        ForgeSubcommand::Inspect(cmd) => cmd.run(),
        ForgeSubcommand::Tree(cmd) => cmd.run(),
        ForgeSubcommand::Geiger(cmd) => {
            let n = cmd.run()?;
            if n > 0 {
                std::process::exit(n as i32);
            }
            Ok(())
        }
        ForgeSubcommand::Doc(cmd) => {
            if cmd.is_watch() {
                utils::block_on(watch::watch_doc(cmd))
            } else {
                utils::block_on(cmd.run())?;
                Ok(())
            }
        }
        ForgeSubcommand::Selectors { command } => utils::block_on(command.run()),
        ForgeSubcommand::Generate(cmd) => match cmd.sub {
            GenerateSubcommands::Test(cmd) => cmd.run(),
        },
        ForgeSubcommand::Compiler(cmd) => cmd.run(),
        ForgeSubcommand::Soldeer(cmd) => utils::block_on(cmd.run()),
        ForgeSubcommand::Eip712(cmd) => cmd.run(),
        ForgeSubcommand::BindJson(cmd) => cmd.run(),
    }
}

/// Set the program execution context based on `forge` subcommand used.
/// The execution context can be set only once per program, and it can be checked by using
/// cheatcodes.
fn init_execution_context(subcommand: &ForgeSubcommand) {
    let context = match subcommand {
        ForgeSubcommand::Test(_) => ForgeContext::Test,
        ForgeSubcommand::Coverage(_) => ForgeContext::Coverage,
        ForgeSubcommand::Snapshot(_) => ForgeContext::Snapshot,
        ForgeSubcommand::Script(cmd) => {
            if cmd.broadcast {
                ForgeContext::ScriptBroadcast
            } else if cmd.resume {
                ForgeContext::ScriptResume
            } else {
                ForgeContext::ScriptDryRun
            }
        }
        _ => ForgeContext::Unknown,
    };
    set_execution_context(context);
}
