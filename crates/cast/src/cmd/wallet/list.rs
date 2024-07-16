use clap::Parser;
use eyre::Result;

use foundry_common::fs;
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
        if self.dir.is_some() || self.all || (!self.ledger && !self.trezor && !self.aws) {
            let _ = self.list_local_senders();
        }

        // Create options for multi wallet - ledger, trezor and AWS
        let list_opts = MultiWalletOptsBuilder::default()
            .ledger(self.ledger || self.all)
            .mnemonic_indexes(Some(vec![0]))
            .trezor(self.trezor || self.all)
            .aws(self.aws || self.all)
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
                                .for_each(|sender| println!("{} ({})", sender, $label));
                        }
                    }
                    Err(e) => {
                        if !self.all {
                            println!("{}", e)
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
                        println!("{name} (Local)");
                    }
                }
            }
        }

        Ok(())
    }
}
