use bitcoin::{base58, PrivateKey, PublicKey};
use clap::Parser;

use btc_tools::cli::{Args, Subcommands};
use btc_tools::{ec_to_wif, public_to_address, wif_to_public};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Subcommands::GenerateVanityWallet(a) => btc_tools::vanity_wallet::main(a)?,
        Subcommands::WifToEc(a) => {
            let private_key = PrivateKey::from_wif(&a.wif.wif)?;
            println!("{}", hex::encode(private_key.to_bytes()));
        }
        Subcommands::EcToWif(a) => {
            let wif = ec_to_wif(&a.hex, a.compressed)?;
            println!("{}", wif);
        }
        Subcommands::WifToPublic(a) => {
            let public_key = wif_to_public(&a.wif.wif)?;
            println!("{}", public_key);
        }
        Subcommands::Base58Encode(a) => {
            println!("{}", base58::encode(&hex::decode(a.data)?));
        }
        Subcommands::Base58EncodeCheck(a) => {
            println!("{}", base58::encode_check(&hex::decode(a.data)?));
        }
        Subcommands::Base58Decode(a) => {
            println!("{}", hex::encode(base58::decode(&a.base58)?));
        }
        Subcommands::Base58DecodeCheck(a) => {
            println!("{}", hex::encode(base58::decode_check(&a.base58)?));
        }
        Subcommands::PublicToAddress(a) => {
            let public_key = PublicKey::from_slice(&hex::decode(a.key)?)?;
            println!("{}", public_to_address(&public_key, a.r#type.r#type)?);
        }
        Subcommands::WifToAddress(a) => {
            let public_key = wif_to_public(&a.wif.wif)?;
            println!("{}", public_to_address(&public_key, a.r#type.r#type)?);
        }
        Subcommands::EcToAddress(a) => {
            let wif_args = a.ec_to_wif_args;
            let wif = ec_to_wif(&wif_args.hex, wif_args.compressed)?;
            let public_key = wif_to_public(&wif)?;
            println!("{}", public_to_address(&public_key, a.type_args.r#type)?);
        }
    }
    Ok(())
}
