use std::io::{stdin, stdout, Read, Write};

use clap::Parser;

use anybase::cli::CliConfig;
use anybase::{
    base10_to_other, check_mapping_table, map_bytes_to_string, map_string_to_bytes, to_base10,
};

fn main() -> anyhow::Result<()> {
    let config = CliConfig::parse();

    if let Some(t) = config.from_table.as_ref() {
        check_mapping_table(t, config.from_base)?;
    }
    if let Some(t) = config.to_table.as_ref() {
        check_mapping_table(t, config.to_base)?;
    }

    let mut data = Vec::new();
    stdin().read_to_end(&mut data)?;

    // if from_table present, map input string to bytes
    let from_data = match config.from_table.as_ref() {
        None => data,
        Some(from_table) => {
            let chars = from_table.chars().collect::<Vec<_>>();
            match String::from_utf8(data) {
                Ok(mut d) => {
                    // when using `echo`, the argument may end with a newline
                    // we need to process it
                    if d.ends_with('\n') && !from_table.contains('\n') {
                        d.remove(d.len() - 1);
                    }
                    map_string_to_bytes(&d, &chars)
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Invalid UTF-8 encoding in input string: {}",
                        e
                    ));
                }
            }
        }
    };
    // use base10 bignum as an conversion intermediate
    let base10 = to_base10(&from_data, config.from_base);

    let to_bytes = base10_to_other(base10, config.to_base);
    match config.to_table.as_ref() {
        None => {
            stdout().write_all(&to_bytes)?;
        }
        Some(to_table) => {
            let chars = to_table.chars().collect::<Vec<_>>();
            let output_string = map_bytes_to_string(&to_bytes, &chars);
            println!("{}", output_string);
        }
    }

    Ok(())
}
