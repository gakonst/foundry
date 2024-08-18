#[macro_use]
extern crate tracing;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use eyre::Result;
use foundry_cli::{handler, utils};
use foundry_evm::inspectors::cheatcodes::{set_execution_context, ForgeContext};

mod cmd;
use cmd::{cache::CacheSubcommands, generate::GenerateSubcommands, watch};

mod opts;
use opts::{Forge, ForgeSubcommand};

#[cfg(all(feature = "jemalloc", unix))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() -> Result<()> {
    handler::install();
    utils::load_dotenv();
    utils::subscriber();
    utils::enable_paint();

    let opts = Forge::parse();
    init_execution_context(&opts.cmd);

    match opts.cmd {
        ForgeSubcommand::Test(cmd) => {
            if cmd.is_watch() {
                utils::block_on(watch::watch_test(cmd))
            } else {
                let outcome = utils::block_on(cmd.run())?;
                outcome.ensure_ok()
            }
        }
        ForgeSubcommand::Script(cmd) => {
            // install the shell before executing the command
            foundry_common::shell::set_shell(foundry_common::shell::Shell::from_args(
                cmd.opts.silent,
                cmd.json,
            ))?;
            utils::block_on(cmd.run_script())
        }
        ForgeSubcommand::Coverage(cmd) => utils::block_on(cmd.run()),
        ForgeSubcommand::Bind(cmd) => cmd.run(),
        ForgeSubcommand::Build(cmd) => {
            if cmd.is_watch() {
                utils::block_on(watch::watch_build(cmd))
            } else {
                cmd.run().map(drop)
            }
        }
        ForgeSubcommand::Debug(cmd) => utils::block_on(cmd.run()),
        ForgeSubcommand::VerifyContract(args) => utils::block_on(args.run()),
        ForgeSubcommand::VerifyCheck(args) => utils::block_on(args.run()),
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
            let config = utils::load_config_with_root(root);
            let project = config.project()?;
            config.cleanup(&project)?;
            Ok(())
        }
        ForgeSubcommand::Snapshot(cmd) => {
            if cmd.is_watch() {
                utils::block_on(watch::watch_snapshot(cmd))
            } else {
                utils::block_on(cmd.run())
            }
        }
        ForgeSubcommand::Fmt(cmd) => cmd.run(),
        ForgeSubcommand::Config(cmd) => cmd.run(),
        ForgeSubcommand::Flatten(cmd) => cmd.run(),
        ForgeSubcommand::Inspect(cmd) => cmd.run(),
        ForgeSubcommand::Tree(cmd) => cmd.run(),
        ForgeSubcommand::Geiger(cmd) => {
            let check = cmd.check;
            let n = cmd.run()?;
            if check && n > 0 {
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
        ForgeSubcommand::VerifyBytecode(cmd) => utils::block_on(cmd.run()),
        ForgeSubcommand::Soldeer(cmd) => cmd.run(),
        ForgeSubcommand::Eip712(cmd) => cmd.run(),
        ForgeSubcommand::BindJson(cmd) => cmd.run(),
        ForgeSubcommands::Mutate(cmd) => {
            utils::block_on(cmd.run())
        }

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
