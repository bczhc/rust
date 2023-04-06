use anybase::cli::CliConfig;
use anyhow::anyhow;
use clap::Parser;
use num::{BigInt, Integer, ToPrimitive, Zero};
use std::collections::HashMap;
use std::io::{stdin, stdout, BufWriter, Read, Write};

fn main() -> anyhow::Result<()> {
    let config = CliConfig::parse();
    let to_base = config.base;
    let mut alphabet_table = config.alphabet_table.as_ref();
    if let Some(a) = alphabet_table.as_mut() {
        let chars = a.chars().collect::<Vec<_>>();
        if chars.len() != to_base as usize {
            return Err(anyhow!("Alphabet table not equal to the output base"));
        }
        let has_duplicates = (1..chars.len()).any(|i| chars[i..].contains(&chars[i - 1]));
        if has_duplicates {
            return Err(anyhow!("Alphabet table characters should be all unique"));
        }
    }

    let mut data = Vec::new();
    stdin().read_to_end(&mut data)?;

    let mut sum = BigInt::from(0);
    for (i, &b) in data.iter().rev().enumerate() {
        sum += BigInt::from(b) * BigInt::from(256).pow(i as u32);
    }

    let mut num = sum.clone();
    let mut digits = Vec::new();
    loop {
        let (q, r) = num.div_rem(&BigInt::from(config.base));
        let terminate = q.is_zero();
        num = q;
        digits.push(r.to_u8().unwrap());
        if terminate {
            break;
        }
    }

    let stdout = stdout().lock();
    let mut stdout = BufWriter::new(stdout);
    let rev_digits = digits.into_iter().rev();
    let alphabet_table = config.alphabet_table.as_ref();
    if let Some(table) = alphabet_table {
        let char_table = table.chars().collect::<Vec<_>>();
        for b in rev_digits {
            write!(&mut stdout, "{}", char_table[b as usize])?;
        }
        writeln!(&mut stdout)?;
    } else {
        for b in rev_digits {
            stdout.write_all(&[b])?;
        }
    }

    Ok(())
}
