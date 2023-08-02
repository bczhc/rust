use bitcoin::key::Secp256k1;
use bitcoin::secp256k1::SecretKey;
use bitcoin::{base58, Address, Network, PrivateKey, PublicKey};
use btc_tools::cli::{AddressType, Args, Subcommands};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Subcommands::GenerateVanityWallet(a) => btc_tools::vanity_wallet::main(a)?,
        Subcommands::WifToEc(a) => {
            let private_key = PrivateKey::from_wif(&a.wif.wif)?;
            println!("{}", hex::encode(private_key.to_bytes()));
        }
        Subcommands::EcToWif(a) => {
            let secret_key = SecretKey::from_slice(&hex::decode(a.hex)?)?;
            let wif = if a.compressed {
                PrivateKey::new(secret_key, Network::Bitcoin)
            } else {
                PrivateKey::new_uncompressed(secret_key, Network::Bitcoin)
            }
            .to_wif();
            println!("{}", wif);
        }
        Subcommands::WifToPublic(a) => {
            let private_key = PrivateKey::from_wif(&a.wif.wif)?;
            let k1 = Secp256k1::new();
            let public_key = private_key.public_key(&k1);
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
            match a.r#type {
                AddressType::P2pkh => {
                    println!("{}", Address::p2pkh(&public_key, Network::Bitcoin));
                }
                AddressType::P2wpkh => {
                    println!("{}", Address::p2wpkh(&public_key, Network::Bitcoin)?);
                }
            }
        }
    }
    Ok(())
}
