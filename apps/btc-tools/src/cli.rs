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
    GenerateVanityWallet(GenerateVanityWalletArgs),
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

#[derive(Debug, clap::Args)]
pub struct GenerateVanityWalletArgs {
    #[arg(help = "The string to be contained in the public key (p2wpkh)")]
    pub substring: String,
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
