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

    let bytes: Box<dyn AsRef<[u8]>> = match args.subcommand {
        Subcommand::Md5 => Box::new(fixed_output_hash::<md5::Md5, _>(reader, iter_num)?),
        Subcommand::Sha1 => Box::new(fixed_output_hash::<sha1::Sha1, _>(reader, iter_num)?),
        Subcommand::Sha256 => Box::new(fixed_output_hash::<sha2::Sha256, _>(reader, iter_num)?),
        Subcommand::Sha512 => Box::new(fixed_output_hash::<sha2::Sha512, _>(reader, iter_num)?),
        Subcommand::Blake3(a) => Box::new(xof_output_hash::<blake3::Hasher, _>(
            reader,
            a.xof_args.length,
            iter_num,
        )?),
    };

    let bytes = (*bytes).as_ref();
    if raw {
        stdout().write_all(bytes)?;
    } else {
        println!("{}", hex::encode(bytes));
    }

    Ok(())
}
