use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(
    author = "bczhc <bczhc0@126.com>",
    about = "Some Bitcoin-related tools"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    WifToEc(WifToEc),
    EcToWif(EcToWif),
    WifToPublic(WifToPublic),
    Base58Encode(HexEncodedStringArg),
    Base58EncodeCheck(HexEncodedStringArg),
    Base58Decode(Base58StringArg),
    Base58DecodeCheck(Base58StringArg),
    PublicToAddress(PublicToAddress),
    WifToAddress(WifToAddressArgs),
    EcToAddress(EcToAddressArgs),
    GenerateAddress(GenerateAddressArgs),
    ValidateAddress(ValidateAddressArgs),
    BrainWallet(BrainWalletArgs),
}

#[derive(Debug, clap::Args)]
pub struct WifToEc {
    #[command(flatten)]
    pub wif: WifArg,
}

#[derive(Debug, clap::Args)]
pub struct EcToWif {
    #[arg(help = "Raw 32-byte hex-encoded private key")]
    pub hex: String,
    #[arg(short, long, required = false)]
    pub compressed: bool,
}

#[derive(Debug, clap::Args)]
pub struct EcToAddressArgs {
    #[command(flatten)]
    pub ec_to_wif_args: EcToWif,
    #[command(flatten)]
    pub r#type_args: AddressTypeArg,
}

#[derive(Debug, clap::Args)]
pub struct WifToPublic {
    #[command(flatten)]
    pub wif: WifArg,
}

/// Generate bitcoin addresses
///
/// The output (to stdout) format is `<WIF/BIP-38> <address>`
#[derive(Debug, clap::Args)]
pub struct GenerateAddressArgs {
    /// The string to be contained in the address (p2wpkh)
    pub substring: String,
    /// Use BIP-38
    #[arg(short, long)]
    pub bip38: bool,
    /// Number of parallel jobs; default is the number of all online processors
    #[arg(short, long)]
    pub jobs: Option<usize>,
}

/// Validate the correctness of: private key -> address, to avoid scams
///
/// This reads from stdin. Line format: <WIF/BIP-38> <address>
///
/// If the address is not the expected one derived from
/// the private key, or any error occurs,
/// this program aborts with the reason; otherwise,
/// all is okay.
#[derive(Debug, clap::Args)]
pub struct ValidateAddressArgs {
    /// Decrypt BIP-38
    #[arg(short, long)]
    pub bip38: bool,
    /// The amount of parallel workers; defaults to the CPU number
    #[arg(short, long)]
    pub jobs: Option<usize>,
    /// Decode BIP38. NOTE: this will print secret keys (in WIF) out
    #[arg(long)]
    pub decode: bool,
}

/// Brain-wallet derivation
///
/// For detailed algorithm and implementation, please refer to the
/// source code :).
#[derive(Debug, clap::Args)]
pub struct BrainWalletArgs {
    /// Iteration number
    ///
    /// This is to make a deliberately-slow hash function
    #[arg(short, long, default_value = "10000")]
    pub iter_num: usize,
    #[command(flatten)]
    pub r#type: AddressTypeArg,
    #[arg(short, long)]
    pub compressed: bool,
    /// Read passphrase from file
    ///
    /// This reads the first line of the given file.
    #[arg(short = 'f', long)]
    pub pass_file: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct WifArg {
    #[arg(help = "Private key in WIF format")]
    pub wif: String,
}

#[derive(Debug, clap::Args)]
pub struct HexEncodedStringArg {
    #[arg(help = "Hex-encoded bytes")]
    pub data: String,
}

#[derive(Debug, clap::Args)]
pub struct Base58StringArg {
    #[arg(help = "Base58-encoded string")]
    pub base58: String,
}

#[derive(Debug, clap::Args)]
pub struct PublicToAddress {
    #[arg(help = "Raw hex-encoded public key")]
    pub key: String,
    #[command(flatten)]
    pub r#type: AddressTypeArg,
}

#[derive(Debug, clap::Args)]
pub struct AddressTypeArg {
    #[arg(help = "Address type")]
    pub r#type: AddressType,
}

#[derive(Debug, clap::Args)]
pub struct WifToAddressArgs {
    #[command(flatten)]
    pub wif: WifArg,
    #[command(flatten)]
    pub r#type: AddressTypeArg,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum AddressType {
    P2pkh,
    P2wpkh,
}
