use crate::lib::{read_config_file, search_config, search_config_index, write_config_file};
use crate::{Configs, Error, MyResult, check_option};
use clap::ArgMatches;

pub fn run(matches: &ArgMatches) -> MyResult<()> {
    // transfer config <key> <value>
    let key = matches.value_of("key").unwrap();
    let value = matches.value_of("value");

    let result = Configs::value_of(key);
    check_option(result, Error::NoConfigKey(String::from(key)))?;

    let mut config = read_config_file()?;
    if value == None {
        let value = search_config(&config, key);
        if let Some(value) = value {
            println!("{}", value);
        }
        return Ok(());
    }
    let value = value.unwrap();

    let index = search_config_index(&config, key);
    match index {
        None => {
            config.push((String::from(key), String::from(value)));
        }
        Some(index) => {
            config[index] = (String::from(key), String::from(value));
        }
    }
    write_config_file(&config)?;

    Ok(())
}
