// https://github.com/rust-lang/rust/issues/114406
#![allow(incomplete_features, const_evaluatable_unchecked)]
#![feature(generic_const_exprs)]

use std::io::{stdin, stdout, Write};

use clap::Parser;

use hash_tools::cli::Subcommand;
use hash_tools::{cli, fixed_output_hash, xof_output_hash};

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    let reader = stdin().lock();
    let iter_num = args.iter_count;
    let raw = args.raw;

    macro_rules! flh_box {
        ($t:ty) => {
            Box::new(fixed_output_hash::<$t, _>(reader, iter_num)?)
        };
    }

    let bytes: Box<dyn AsRef<[u8]>> = match args.subcommand {
        Subcommand::Md5 => flh_box!(md5::Md5),
        Subcommand::Sha1 => flh_box!(sha1::Sha1),
        Subcommand::Sha256 => flh_box!(sha2::Sha256),
        Subcommand::Sha512 => flh_box!(sha2::Sha512),
        Subcommand::Blake3(a) => Box::new(xof_output_hash::<blake3::Hasher, _>(
            reader, a.length, iter_num,
        )?),
        Subcommand::Sha3_256 => flh_box!(sha3::Sha3_256),
        Subcommand::Sha3_512 => flh_box!(sha3::Sha3_512),
        Subcommand::Ripemd128 => flh_box!(ripemd::Ripemd128),
        Subcommand::Ripemd160 => flh_box!(ripemd::Ripemd160),
        Subcommand::Ripemd256 => flh_box!(ripemd::Ripemd256),
        Subcommand::Blake2b512 => flh_box!(blake2::Blake2b512),
        Subcommand::Blake2s256 => flh_box!(blake2::Blake2s256),
    };

    let bytes = (*bytes).as_ref();
    if raw {
        stdout().write_all(bytes)?;
    } else {
        println!("{}", hex::encode(bytes));
    };

    Ok(())
}
