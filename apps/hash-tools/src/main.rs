// https://github.com/rust-lang/rust/issues/114406
#![allow(incomplete_features, const_evaluatable_unchecked)]
#![feature(generic_const_exprs)]

use std::io::{stderr, stdin, stdout, Write};

use clap::Parser;

use bczhc_lib::io::duplicator::DuplicationReader;
use hash_tools::cli::Subcommand;
use hash_tools::{cli, fixed_output_hash, xof_output_hash};

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    let iter_num = args.iter_count;
    let raw = args.raw;
    let pipe_input = args.pipe_input;
    let stdin = stdin().lock();

    macro_rules! flh_box {
        ($t:ty) => {
            if pipe_input {
                let stdout = stdout().lock();
                let dupe_reader = DuplicationReader::new(stdin, stdout);
                Box::new(fixed_output_hash::<$t, _>(dupe_reader, iter_num)?)
            } else {
                Box::new(fixed_output_hash::<$t, _>(stdin, iter_num)?)
            }
        };
    }

    let bytes: Box<dyn AsRef<[u8]>> = match args.subcommand {
        Subcommand::Md5 => flh_box!(md5::Md5),
        Subcommand::Sha1 => flh_box!(sha1::Sha1),
        Subcommand::Sha256 => flh_box!(sha2::Sha256),
        Subcommand::Sha512 => flh_box!(sha2::Sha512),
        Subcommand::Blake3(a) => Box::new(xof_output_hash::<blake3::Hasher, _>(
            stdin, a.length, iter_num,
        )?),
        Subcommand::Sha3_256 => flh_box!(sha3::Sha3_256),
        Subcommand::Sha3_512 => flh_box!(sha3::Sha3_512),
        Subcommand::Ripemd128 => flh_box!(ripemd::Ripemd128),
        Subcommand::Ripemd160 => flh_box!(ripemd::Ripemd160),
        Subcommand::Ripemd256 => flh_box!(ripemd::Ripemd256),
        Subcommand::Blake2b512 => flh_box!(blake2::Blake2b512),
        Subcommand::Blake2s256 => flh_box!(blake2::Blake2s256),
    };

    let mut result_writer: Box<dyn Write> = if pipe_input {
        Box::new(stderr())
    } else {
        Box::new(stdout())
    };

    let bytes = (*bytes).as_ref();
    if raw {
        result_writer.write_all(bytes)?;
        result_writer.flush()?;
    } else {
        writeln!(&mut result_writer, "{}", hex::encode(bytes))?;
    };

    Ok(())
}
