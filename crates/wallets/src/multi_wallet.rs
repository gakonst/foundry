use crate::{
    utils,
    wallet_signer::{PendingSigner, WalletSigner},
};
use alloy_primitives::{map::AddressHashMap, Address};
use alloy_signer::Signer;
use clap::Parser;
use derive_builder::Builder;
use eyre::Result;
use foundry_config::Config;
use serde::Serialize;
use std::path::PathBuf;

/// Container for multiple wallets.
#[derive(Debug, Default)]
pub struct MultiWallet {
    /// Vector of wallets that require an action to be unlocked.
    /// Those are lazily unlocked on the first access of the signers.
    pending_signers: Vec<PendingSigner>,
    /// Contains unlocked signers.
    signers: AddressHashMap<WalletSigner>,
}

impl MultiWallet {
    pub fn new(pending_signers: Vec<PendingSigner>, signers: Vec<WalletSigner>) -> Self {
        let signers = signers.into_iter().map(|signer| (signer.address(), signer)).collect();
        Self { pending_signers, signers }
    }

    fn maybe_unlock_pending(&mut self) -> Result<()> {
        for pending in self.pending_signers.drain(..) {
            let signer = pending.unlock()?;
            self.signers.insert(signer.address(), signer);
        }
        Ok(())
    }

    pub fn signers(&mut self) -> Result<&AddressHashMap<WalletSigner>> {
        self.maybe_unlock_pending()?;
        Ok(&self.signers)
    }

    pub fn into_signers(mut self) -> Result<AddressHashMap<WalletSigner>> {
        self.maybe_unlock_pending()?;
        Ok(self.signers)
    }

    pub fn add_signer(&mut self, signer: WalletSigner) {
        self.signers.insert(signer.address(), signer);
    }
}

/// A macro that initializes multiple wallets
///
/// Should be used with a [`MultiWallet`] instance
macro_rules! create_hw_wallets {
    ($self:ident, $create_signer:expr, $signers:ident) => {
        let mut $signers = vec![];

        if let Some(hd_paths) = &$self.hd_paths {
            for path in hd_paths {
                let hw = $create_signer(Some(path), 0).await?;
                $signers.push(hw);
            }
        }

        if let Some(mnemonic_indexes) = &$self.mnemonic_indexes {
            for index in mnemonic_indexes {
                let hw = $create_signer(None, *index).await?;
                $signers.push(hw);
            }
        }

        if $signers.is_empty() {
            let hw = $create_signer(None, 0).await?;
            $signers.push(hw);
        }
    };
}

/// The wallet options can either be:
/// 1. Ledger
/// 2. Trezor
/// 3. Mnemonics (via file path)
/// 4. Keystores (via file path)
/// 5. Private Keys (cleartext in CLI)
/// 6. Private Keys (interactively via secure prompt)
/// 7. AWS KMS
#[derive(Builder, Clone, Debug, Default, Serialize, Parser)]
#[command(next_help_heading = "Wallet options", about = None, long_about = None)]
pub struct MultiWalletOpts {
    /// The sender accounts.
    #[arg(
        long,
        short = 'a',
        help_heading = "Wallet options - raw",
        value_name = "ADDRESSES",
        env = "ETH_FROM",
        num_args(0..),
    )]
    #[builder(default = "None")]
    pub froms: Option<Vec<Address>>,

    /// Open an interactive prompt to enter your private key.
    ///
    /// Takes a value for the number of keys to enter.
    #[arg(
        long,
        short,
        help_heading = "Wallet options - raw",
        default_value = "0",
        value_name = "NUM"
    )]
    pub interactives: u32,

    /// Use the provided private keys.
    #[arg(long, help_heading = "Wallet options - raw", value_name = "RAW_PRIVATE_KEYS")]
    #[builder(default = "None")]
    pub private_keys: Option<Vec<String>>,

    /// Use the provided private key.
    #[arg(
        long,
        help_heading = "Wallet options - raw",
        conflicts_with = "private_keys",
        value_name = "RAW_PRIVATE_KEY"
    )]
    #[builder(default = "None")]
    pub private_key: Option<String>,

    /// Use the mnemonic phrases of mnemonic files at the specified paths.
    #[arg(long, alias = "mnemonic-paths", help_heading = "Wallet options - raw")]
    #[builder(default = "None")]
    pub mnemonics: Option<Vec<String>>,

    /// Use a BIP39 passphrases for the mnemonic.
    #[arg(long, help_heading = "Wallet options - raw", value_name = "PASSPHRASE")]
    #[builder(default = "None")]
    pub mnemonic_passphrases: Option<Vec<String>>,

    /// The wallet derivation path.
    ///
    /// Works with both --mnemonic-path and hardware wallets.
    #[arg(
        long = "mnemonic-derivation-paths",
        alias = "hd-paths",
        help_heading = "Wallet options - raw",
        value_name = "PATH"
    )]
    #[builder(default = "None")]
    pub hd_paths: Option<Vec<String>>,

    /// Use the private key from the given mnemonic index.
    ///
    /// Can be used with --mnemonics, --ledger, --aws and --trezor.
    #[arg(
        long,
        conflicts_with = "hd_paths",
        help_heading = "Wallet options - raw",
        default_value = "0",
        value_name = "INDEXES"
    )]
    pub mnemonic_indexes: Option<Vec<u32>>,

    /// Use the keystore by its filename in the given folder.
    #[arg(
        long = "keystore",
        visible_alias = "keystores",
        help_heading = "Wallet options - keystore",
        value_name = "PATHS",
        env = "ETH_KEYSTORE"
    )]
    #[builder(default = "None")]
    pub keystore_paths: Option<Vec<String>>,

    /// Use a keystore from the default keystores folder (~/.foundry/keystores) by its filename.
    #[arg(
        long = "account",
        visible_alias = "accounts",
        help_heading = "Wallet options - keystore",
        value_name = "ACCOUNT_NAMES",
        env = "ETH_KEYSTORE_ACCOUNT",
        conflicts_with = "keystore_paths"
    )]
    #[builder(default = "None")]
    pub keystore_account_names: Option<Vec<String>>,

    /// The keystore password.
    ///
    /// Used with --keystore.
    #[arg(
        long = "password",
        help_heading = "Wallet options - keystore",
        requires = "keystore_paths",
        value_name = "PASSWORDS"
    )]
    #[builder(default = "None")]
    pub keystore_passwords: Option<Vec<String>>,

    /// The keystore password file path.
    ///
    /// Used with --keystore.
    #[arg(
        long = "password-file",
        help_heading = "Wallet options - keystore",
        requires = "keystore_paths",
        value_name = "PATHS",
        env = "ETH_PASSWORD"
    )]
    #[builder(default = "None")]
    pub keystore_password_files: Option<Vec<String>>,

    /// Use a Ledger hardware wallet.
    #[arg(long, short, help_heading = "Wallet options - hardware wallet")]
    pub ledger: bool,

    /// Use a Trezor hardware wallet.
    #[arg(long, short, help_heading = "Wallet options - hardware wallet")]
    pub trezor: bool,

    /// Use AWS Key Management Service.
    #[arg(long, help_heading = "Wallet options - remote", hide = !cfg!(feature = "aws-kms"))]
    pub aws: bool,

    /// Use Google Cloud Key Management Service.
    #[arg(long, help_heading = "Wallet options - remote", hide = !cfg!(feature = "gcp-kms"))]
    pub gcp: bool,
}

impl MultiWalletOpts {
    /// Returns [MultiWallet] container configured with provided options.
    pub async fn get_multi_wallet(&self) -> Result<MultiWallet> {
        let mut pending = Vec::new();
        let mut signers: Vec<WalletSigner> = Vec::new();

        if let Some(ledgers) = self.ledgers().await? {
            signers.extend(ledgers);
        }
        if let Some(trezors) = self.trezors().await? {
            signers.extend(trezors);
        }
        if let Some(aws_signers) = self.aws_signers().await? {
            signers.extend(aws_signers);
        }
        if let Some(gcp_signer) = self.gcp_signers().await? {
            signers.extend(gcp_signer);
        }
        if let Some((pending_keystores, unlocked)) = self.keystores()? {
            pending.extend(pending_keystores);
            signers.extend(unlocked);
        }
        if let Some(pks) = self.private_keys()? {
            signers.extend(pks);
        }
        if let Some(mnemonics) = self.mnemonics()? {
            signers.extend(mnemonics);
        }
        if self.interactives > 0 {
            pending.extend(std::iter::repeat_n(
                PendingSigner::Interactive,
                self.interactives as usize,
            ));
        }

        Ok(MultiWallet::new(pending, signers))
    }

    pub fn private_keys(&self) -> Result<Option<Vec<WalletSigner>>> {
        let mut pks = vec![];
        if let Some(private_key) = &self.private_key {
            pks.push(private_key);
        }
        if let Some(private_keys) = &self.private_keys {
            for pk in private_keys {
                pks.push(pk);
            }
        }
        if !pks.is_empty() {
            let wallets = pks
                .into_iter()
                .map(|pk| utils::create_private_key_signer(pk))
                .collect::<Result<Vec<_>>>()?;
            Ok(Some(wallets))
        } else {
            Ok(None)
        }
    }

    fn keystore_paths(&self) -> Result<Option<Vec<PathBuf>>> {
        if let Some(keystore_paths) = &self.keystore_paths {
            return Ok(Some(keystore_paths.iter().map(PathBuf::from).collect()));
        }
        if let Some(keystore_account_names) = &self.keystore_account_names {
            let default_keystore_dir = Config::foundry_keystores_dir()
                .ok_or_else(|| eyre::eyre!("Could not find the default keystore directory."))?;
            return Ok(Some(
                keystore_account_names
                    .iter()
                    .map(|keystore_name| default_keystore_dir.join(keystore_name))
                    .collect(),
            ));
        }
        Ok(None)
    }

    /// Returns all wallets read from the provided keystores arguments
    ///
    /// Returns `Ok(None)` if no keystore provided.
    pub fn keystores(&self) -> Result<Option<(Vec<PendingSigner>, Vec<WalletSigner>)>> {
        if let Some(keystore_paths) = self.keystore_paths()? {
            let mut pending = Vec::new();
            let mut signers = Vec::new();

            let mut passwords_iter =
                self.keystore_passwords.clone().unwrap_or_default().into_iter();

            let mut password_files_iter =
                self.keystore_password_files.clone().unwrap_or_default().into_iter();

            for path in &keystore_paths {
                let (maybe_signer, maybe_pending) = utils::create_keystore_signer(
                    path,
                    passwords_iter.next().as_deref(),
                    password_files_iter.next().as_deref(),
                )?;
                if let Some(pending_signer) = maybe_pending {
                    pending.push(pending_signer);
                } else if let Some(signer) = maybe_signer {
                    signers.push(signer);
                }
            }
            return Ok(Some((pending, signers)));
        }
        Ok(None)
    }

    pub fn mnemonics(&self) -> Result<Option<Vec<WalletSigner>>> {
        if let Some(ref mnemonics) = self.mnemonics {
            let mut wallets = vec![];

            let mut hd_paths_iter = self.hd_paths.clone().unwrap_or_default().into_iter();

            let mut passphrases_iter =
                self.mnemonic_passphrases.clone().unwrap_or_default().into_iter();

            let mut indexes_iter = self.mnemonic_indexes.clone().unwrap_or_default().into_iter();

            for mnemonic in mnemonics {
                let wallet = utils::create_mnemonic_signer(
                    mnemonic,
                    passphrases_iter.next().as_deref(),
                    hd_paths_iter.next().as_deref(),
                    indexes_iter.next().unwrap_or(0),
                )?;
                wallets.push(wallet);
            }
            return Ok(Some(wallets));
        }
        Ok(None)
    }

    pub async fn ledgers(&self) -> Result<Option<Vec<WalletSigner>>> {
        if self.ledger {
            let mut args = self.clone();

            if let Some(paths) = &args.hd_paths {
                if paths.len() > 1 {
                    eyre::bail!("Ledger only supports one signer.");
                }
                args.mnemonic_indexes = None;
            }

            create_hw_wallets!(args, utils::create_ledger_signer, wallets);
            return Ok(Some(wallets));
        }
        Ok(None)
    }

    pub async fn trezors(&self) -> Result<Option<Vec<WalletSigner>>> {
        if self.trezor {
            create_hw_wallets!(self, utils::create_trezor_signer, wallets);
            return Ok(Some(wallets));
        }
        Ok(None)
    }

    pub async fn aws_signers(&self) -> Result<Option<Vec<WalletSigner>>> {
        #[cfg(feature = "aws-kms")]
        if self.aws {
            let mut wallets = vec![];
            let aws_keys = std::env::var("AWS_KMS_KEY_IDS")
                .or(std::env::var("AWS_KMS_KEY_ID"))?
                .split(',')
                .map(|k| k.to_string())
                .collect::<Vec<_>>();

            for key in aws_keys {
                let aws_signer = WalletSigner::from_aws(key).await?;
                wallets.push(aws_signer)
            }

            return Ok(Some(wallets));
        }

        Ok(None)
    }

    /// Returns a list of GCP signers if the GCP flag is set.
    ///
    /// The GCP signers are created from the following environment variables:
    /// - GCP_PROJECT_ID: The GCP project ID. e.g. `my-project-123456`.
    /// - GCP_LOCATION: The GCP location. e.g. `us-central1`.
    /// - GCP_KEY_RING: The GCP key ring name. e.g. `my-key-ring`.
    /// - GCP_KEY_NAME: The GCP key name. e.g. `my-key`.
    /// - GCP_KEY_VERSION: The GCP key version. e.g. `1`.
    ///
    /// For more information on GCP KMS, see the [official documentation](https://cloud.google.com/kms/docs).
    pub async fn gcp_signers(&self) -> Result<Option<Vec<WalletSigner>>> {
        #[cfg(feature = "gcp-kms")]
        if self.gcp {
            let mut wallets = vec![];

            let project_id = std::env::var("GCP_PROJECT_ID")?;
            let location = std::env::var("GCP_LOCATION")?;
            let key_ring = std::env::var("GCP_KEY_RING")?;
            let key_names = std::env::var("GCP_KEY_NAME")?;
            let key_version = std::env::var("GCP_KEY_VERSION")?;

            let gcp_signer = WalletSigner::from_gcp(
                project_id,
                location,
                key_ring,
                key_names,
                key_version.parse()?,
            )
            .await?;
            wallets.push(gcp_signer);

            return Ok(Some(wallets));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::Path, str::FromStr};

    #[test]
    fn parse_keystore_args() {
        let args: MultiWalletOpts =
            MultiWalletOpts::parse_from(["foundry-cli", "--keystores", "my/keystore/path"]);
        assert_eq!(args.keystore_paths, Some(vec!["my/keystore/path".to_string()]));

        std::env::set_var("ETH_KEYSTORE", "MY_KEYSTORE");
        let args: MultiWalletOpts = MultiWalletOpts::parse_from(["foundry-cli"]);
        assert_eq!(args.keystore_paths, Some(vec!["MY_KEYSTORE".to_string()]));

        std::env::remove_var("ETH_KEYSTORE");
    }

    #[test]
    fn parse_keystore_password_file() {
        let keystore =
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../cast/tests/fixtures/keystore"));
        let keystore_file = keystore
            .join("UTC--2022-12-20T10-30-43.591916000Z--ec554aeafe75601aaab43bd4621a22284db566c2");

        let keystore_password_file = keystore.join("password-ec554").into_os_string();

        let args: MultiWalletOpts = MultiWalletOpts::parse_from([
            "foundry-cli",
            "--keystores",
            keystore_file.to_str().unwrap(),
            "--password-file",
            keystore_password_file.to_str().unwrap(),
        ]);
        assert_eq!(
            args.keystore_password_files,
            Some(vec![keystore_password_file.to_str().unwrap().to_string()])
        );

        let (_, unlocked) = args.keystores().unwrap().unwrap();
        assert_eq!(unlocked.len(), 1);
        assert_eq!(
            unlocked[0].address(),
            Address::from_str("0xec554aeafe75601aaab43bd4621a22284db566c2").unwrap()
        );
    }

    // https://github.com/foundry-rs/foundry/issues/5179
    #[test]
    fn should_not_require_the_mnemonics_flag_with_mnemonic_indexes() {
        let wallet_options = vec![
            ("ledger", "--mnemonic-indexes", 1),
            ("trezor", "--mnemonic-indexes", 2),
            ("aws", "--mnemonic-indexes", 10),
        ];

        for test_case in wallet_options {
            let args: MultiWalletOpts = MultiWalletOpts::parse_from([
                "foundry-cli",
                &format!("--{}", test_case.0),
                test_case.1,
                &test_case.2.to_string(),
            ]);

            match test_case.0 {
                "ledger" => assert!(args.ledger),
                "trezor" => assert!(args.trezor),
                "aws" => assert!(args.aws),
                _ => panic!("Should have matched one of the previous wallet options"),
            }

            assert_eq!(
                args.mnemonic_indexes.expect("--mnemonic-indexes should have been set")[0],
                test_case.2
            )
        }
    }
}
