use alloy_primitives::{hex, Address};
use alloy_signer::{k256::ecdsa::SigningKey, utils::secret_key_to_address};
use alloy_signer_local::PrivateKeySigner;
use clap::Parser;
use eyre::Result;
use itertools::Either;
use rayon::iter::{self, ParallelIterator};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

/// Type alias for the result of [generate_wallet].
pub type GeneratedWallet = (SigningKey, Address);

/// CLI arguments for `cast wallet vanity`.
#[derive(Clone, Debug, Parser)]
pub struct VanityArgs {
    /// Prefix regex pattern or hex string.
    #[arg(long, value_name = "PATTERN", required_unless_present = "ends_with")]
    pub starts_with: Option<String>,

    /// Suffix regex pattern or hex string.
    #[arg(long, value_name = "PATTERN")]
    pub ends_with: Option<String>,

    // 2^64-1 is max possible nonce per [eip-2681](https://eips.ethereum.org/EIPS/eip-2681).
    /// Generate a vanity contract address created by the generated keypair with the specified
    /// nonce.
    #[arg(long)]
    pub nonce: Option<u64>,

    /// Path to save the generated vanity contract address to.
    ///
    /// If provided, the generated vanity addresses will appended to a JSON array in the specified
    /// file.
    #[arg(
        long,
        value_hint = clap::ValueHint::FilePath,
        value_name = "PATH",
    )]
    pub save_path: Option<PathBuf>,
}

/// WalletData contains address and private_key information for a wallet.
#[derive(Serialize, Deserialize)]
struct WalletData {
    address: String,
    private_key: String,
}

/// Wallets is a collection of WalletData.
#[derive(Default, Serialize, Deserialize)]
struct Wallets {
    wallets: Vec<WalletData>,
}

impl WalletData {
    pub fn new(wallet: &PrivateKeySigner) -> Self {
        Self {
            address: wallet.address().to_checksum(None),
            private_key: format!("0x{}", hex::encode(wallet.credential().to_bytes())),
        }
    }
}

impl VanityArgs {
    pub fn run(self) -> Result<PrivateKeySigner> {
        let Self { starts_with, ends_with, nonce, save_path } = self;

        let mut left_exact_hex = None;
        let mut left_regex = None;
        if let Some(prefix) = starts_with {
            match parse_pattern(&prefix, true)? {
                Either::Left(left) => left_exact_hex = Some(left),
                Either::Right(re) => left_regex = Some(re),
            }
        }

        let mut right_exact_hex = None;
        let mut right_regex = None;
        if let Some(suffix) = ends_with {
            match parse_pattern(&suffix, false)? {
                Either::Left(right) => right_exact_hex = Some(right),
                Either::Right(re) => right_regex = Some(re),
            }
        }

        macro_rules! find_vanity {
            ($m:ident, $nonce: ident) => {
                if let Some(nonce) = $nonce {
                    find_vanity_address_with_nonce($m, nonce)
                } else {
                    find_vanity_address($m)
                }
            };
        }

        println!("Starting to generate vanity address...");
        let timer = Instant::now();

        let wallet = match (left_exact_hex, left_regex, right_exact_hex, right_regex) {
            (Some(left), _, Some(right), _) => {
                let matcher = HexMatcher { left, right };
                find_vanity!(matcher, nonce)
            }
            (Some(left), _, _, Some(right)) => {
                let matcher = LeftExactRightRegexMatcher { left, right };
                find_vanity!(matcher, nonce)
            }
            (_, Some(left), _, Some(right)) => {
                let matcher = RegexMatcher { left, right };
                find_vanity!(matcher, nonce)
            }
            (_, Some(left), Some(right), _) => {
                let matcher = LeftRegexRightExactMatcher { left, right };
                find_vanity!(matcher, nonce)
            }
            (Some(left), None, None, None) => {
                let matcher = LeftHexMatcher { left };
                find_vanity!(matcher, nonce)
            }
            (None, None, Some(right), None) => {
                let matcher = RightHexMatcher { right };
                find_vanity!(matcher, nonce)
            }
            (None, Some(re), None, None) => {
                let matcher = SingleRegexMatcher { re };
                find_vanity!(matcher, nonce)
            }
            (None, None, None, Some(re)) => {
                let matcher = SingleRegexMatcher { re };
                find_vanity!(matcher, nonce)
            }
            _ => unreachable!(),
        }
        .expect("failed to generate vanity wallet");

        // If a save path is provided, save the generated vanity wallet to the specified path.
        if let Some(save_path) = save_path {
            save_wallet_to_file(&wallet, &save_path)?;
        }

        println!(
            "Successfully found vanity address in {:.3} seconds.{}{}\nAddress: {}\nPrivate Key: 0x{}",
            timer.elapsed().as_secs_f64(),
            if nonce.is_some() { "\nContract address: " } else { "" },
            if nonce.is_some() {
                wallet.address().create(nonce.unwrap()).to_checksum(None)
            } else {
                String::new()
            },
            wallet.address().to_checksum(None),
            hex::encode(wallet.credential().to_bytes()),
        );

        Ok(wallet)
    }
}

/// Saves the specified `wallet` to a 'vanity_addresses.json' file at the given `save_path`.
/// If the file exists, the wallet data is appended to the existing content;
/// otherwise, a new file is created.
fn save_wallet_to_file(wallet: &PrivateKeySigner, path: &Path) -> Result<()> {
    let mut wallets = if path.exists() {
        let data = fs::read_to_string(path)?;
        serde_json::from_str::<Wallets>(&data).unwrap_or_default()
    } else {
        Wallets::default()
    };

    wallets.wallets.push(WalletData::new(wallet));

    fs::write(path, serde_json::to_string_pretty(&wallets)?)?;
    Ok(())
}

/// Generates random wallets until `matcher` matches the wallet address, returning the wallet.
pub fn find_vanity_address<T: VanityMatcher>(matcher: T) -> Option<PrivateKeySigner> {
    wallet_generator().find_any(create_matcher(matcher)).map(|(key, _)| key.into())
}

/// Generates random wallets until `matcher` matches the contract address created at `nonce`,
/// returning the wallet.
pub fn find_vanity_address_with_nonce<T: VanityMatcher>(
    matcher: T,
    nonce: u64,
) -> Option<PrivateKeySigner> {
    wallet_generator().find_any(create_nonce_matcher(matcher, nonce)).map(|(key, _)| key.into())
}

/// Creates a nonce matcher function, which takes a reference to a [GeneratedWallet] and returns
/// whether it found a match or not by using `matcher`.
#[inline]
pub fn create_matcher<T: VanityMatcher>(matcher: T) -> impl Fn(&GeneratedWallet) -> bool {
    move |(_, addr)| matcher.is_match(addr)
}

/// Creates a nonce matcher function, which takes a reference to a [GeneratedWallet] and a nonce and
/// returns whether it found a match or not by using `matcher`.
#[inline]
pub fn create_nonce_matcher<T: VanityMatcher>(
    matcher: T,
    nonce: u64,
) -> impl Fn(&GeneratedWallet) -> bool {
    move |(_, addr)| {
        let contract_addr = addr.create(nonce);
        matcher.is_match(&contract_addr)
    }
}

/// Returns an infinite parallel iterator which yields a [GeneratedWallet].
#[inline]
pub fn wallet_generator() -> iter::Map<iter::Repeat<()>, impl Fn(()) -> GeneratedWallet> {
    iter::repeat(()).map(|()| generate_wallet())
}

/// Generates a random K-256 signing key and derives its Ethereum address.
pub fn generate_wallet() -> GeneratedWallet {
    let key = SigningKey::random(&mut rand::thread_rng());
    let address = secret_key_to_address(&key);
    (key, address)
}

/// A trait to match vanity addresses.
pub trait VanityMatcher: Send + Sync {
    fn is_match(&self, addr: &Address) -> bool;
}

/// Matches start and end hex.
pub struct HexMatcher {
    pub left: Vec<u8>,
    pub right: Vec<u8>,
}

impl VanityMatcher for HexMatcher {
    #[inline]
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_slice();
        bytes.starts_with(&self.left) && bytes.ends_with(&self.right)
    }
}

/// Matches only start hex.
pub struct LeftHexMatcher {
    pub left: Vec<u8>,
}

impl VanityMatcher for LeftHexMatcher {
    #[inline]
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_slice();
        bytes.starts_with(&self.left)
    }
}

/// Matches only end hex.
pub struct RightHexMatcher {
    pub right: Vec<u8>,
}

impl VanityMatcher for RightHexMatcher {
    #[inline]
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_slice();
        bytes.ends_with(&self.right)
    }
}

/// Matches start hex and end regex.
pub struct LeftExactRightRegexMatcher {
    pub left: Vec<u8>,
    pub right: Regex,
}

impl VanityMatcher for LeftExactRightRegexMatcher {
    #[inline]
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_slice();
        bytes.starts_with(&self.left) && self.right.is_match(&hex::encode(bytes))
    }
}

/// Matches start regex and end hex.
pub struct LeftRegexRightExactMatcher {
    pub left: Regex,
    pub right: Vec<u8>,
}

impl VanityMatcher for LeftRegexRightExactMatcher {
    #[inline]
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_slice();
        bytes.ends_with(&self.right) && self.left.is_match(&hex::encode(bytes))
    }
}

/// Matches a single regex.
pub struct SingleRegexMatcher {
    pub re: Regex,
}

impl VanityMatcher for SingleRegexMatcher {
    #[inline]
    fn is_match(&self, addr: &Address) -> bool {
        let addr = hex::encode(addr);
        self.re.is_match(&addr)
    }
}

/// Matches start and end regex.
pub struct RegexMatcher {
    pub left: Regex,
    pub right: Regex,
}

impl VanityMatcher for RegexMatcher {
    #[inline]
    fn is_match(&self, addr: &Address) -> bool {
        let addr = hex::encode(addr);
        self.left.is_match(&addr) && self.right.is_match(&addr)
    }
}

fn parse_pattern(pattern: &str, is_start: bool) -> Result<Either<Vec<u8>, Regex>> {
    if let Ok(decoded) = hex::decode(pattern) {
        if decoded.len() > 20 {
            return Err(eyre::eyre!("Hex pattern must be less than 20 bytes"));
        }
        Ok(Either::Left(decoded))
    } else {
        let (prefix, suffix) = if is_start { ("^", "") } else { ("", "$") };
        Ok(Either::Right(Regex::new(&format!("{prefix}{pattern}{suffix}"))?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_simple_vanity_start() {
        let args: VanityArgs = VanityArgs::parse_from(["foundry-cli", "--starts-with", "00"]);
        let wallet = args.run().unwrap();
        let addr = wallet.address();
        let addr = format!("{addr:x}");
        assert!(addr.starts_with("00"));
    }

    #[test]
    fn find_simple_vanity_start2() {
        let args: VanityArgs = VanityArgs::parse_from(["foundry-cli", "--starts-with", "9"]);
        let wallet = args.run().unwrap();
        let addr = wallet.address();
        let addr = format!("{addr:x}");
        assert!(addr.starts_with('9'));
    }

    #[test]
    fn find_simple_vanity_end() {
        let args: VanityArgs = VanityArgs::parse_from(["foundry-cli", "--ends-with", "00"]);
        let wallet = args.run().unwrap();
        let addr = wallet.address();
        let addr = format!("{addr:x}");
        assert!(addr.ends_with("00"));
    }

    #[test]
    fn save_path() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let args: VanityArgs = VanityArgs::parse_from([
            "foundry-cli",
            "--starts-with",
            "00",
            "--save-path",
            tmp.path().to_str().unwrap(),
        ]);
        args.run().unwrap();
        assert!(tmp.path().exists());
        let s = fs::read_to_string(tmp.path()).unwrap();
        let wallets: Wallets = serde_json::from_str(&s).unwrap();
        assert!(!wallets.wallets.is_empty());
    }
}
