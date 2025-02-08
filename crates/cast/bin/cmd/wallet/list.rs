use clap::Parser;
use eyre::Result;
use std::env;

use foundry_common::{fs, sh_err, sh_println};
use foundry_config::Config;
use foundry_wallets::multi_wallet::MultiWalletOptsBuilder;

/// CLI arguments for `cast wallet list`.
#[derive(Clone, Debug, Parser)]
pub struct ListArgs {
    /// List all the accounts in the keystore directory.
    /// Default keystore directory is used if no path provided.
    #[arg(long, default_missing_value = "", num_args(0..=1))]
    dir: Option<String>,

    /// List accounts from a Ledger hardware wallet.
    #[arg(long, short, group = "hw-wallets")]
    ledger: bool,

    /// List accounts from a Trezor hardware wallet.
    #[arg(long, short, group = "hw-wallets")]
    trezor: bool,

    /// List accounts from AWS KMS.
    #[arg(long, hide = !cfg!(feature = "aws-kms"))]
    aws: bool,

    /// List accounts from Google Cloud KMS.
    #[arg(long, hide = !cfg!(feature = "gcp-kms"))]
    gcp: bool,

    /// List all configured accounts.
    #[arg(long, group = "hw-wallets")]
    all: bool,

    /// Max number of addresses to display from hardware wallets.
    #[arg(long, short, default_value = "3", requires = "hw-wallets")]
    max_senders: Option<usize>,
}

impl ListArgs {
    pub async fn run(self) -> Result<()> {
        // list local accounts as files in keystore dir, no need to unlock / provide password
        if self.dir.is_some() ||
            self.all ||
            (!self.ledger && !self.trezor && !self.aws && !self.gcp)
        {
            let _ = self.list_local_senders();
        }

        // Create options for multi wallet - ledger, trezor and AWS
        let list_opts = MultiWalletOptsBuilder::default()
            .ledger(self.ledger || self.all)
            .mnemonic_indexes(Some(vec![0]))
            .trezor(self.trezor || self.all)
            .aws(self.aws || self.all)
            .gcp(self.gcp || (self.all && gcp_env_vars_set()))
            .interactives(0)
            .build()
            .expect("build multi wallet");

        // macro to print senders for a list of signers
        macro_rules! list_senders {
            ($signers:expr, $label:literal) => {
                match $signers.await {
                    Ok(signers) => {
                        for signer in signers.unwrap_or_default().iter() {
                            signer
                                .available_senders(self.max_senders.unwrap())
                                .await?
                                .iter()
                                .for_each(|sender| {
                                    let _ = sh_println!("{} ({})", sender, $label);
                                })
                        }
                    }
                    Err(e) => {
                        if !self.all {
                            sh_err!("{}", e)?;
                        }
                    }
                }
            };
        }

        list_senders!(list_opts.ledgers(), "Ledger");
        list_senders!(list_opts.trezors(), "Trezor");
        list_senders!(list_opts.aws_signers(), "AWS");

        Ok(())
    }

    fn list_local_senders(&self) -> Result<()> {
        let keystore_path = self.dir.clone().unwrap_or_default();
        let keystore_dir = if keystore_path.is_empty() {
            // Create the keystore default directory if it doesn't exist
            let default_dir = Config::foundry_keystores_dir().unwrap();
            fs::create_dir_all(&default_dir)?;
            default_dir
        } else {
            dunce::canonicalize(keystore_path)?
        };

        // List all files within the keystore directory.
        for entry in std::fs::read_dir(keystore_dir)? {
            let path = entry?.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(name) = file_name.to_str() {
                        sh_println!("{name} (Local)")?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn gcp_env_vars_set() -> bool {
    let required_vars =
        ["GCP_PROJECT_ID", "GCP_LOCATION", "GCP_KEY_RING", "GCP_KEY_NAME", "GCP_KEY_VERSION"];

    required_vars.iter().all(|&var| env::var(var).is_ok())
}
