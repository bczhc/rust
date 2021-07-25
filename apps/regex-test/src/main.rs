use std::env::Args;
use std::fmt::Error;
use std::fs::File;

use clap::{App, Arg, ArgMatches, SubCommand};

#[derive(Debug)]
enum RetErr<A, B, C> {
    Rust(A),
    PCRE(B),
    Text(C),
}

type RetErrType = RetErr<regex::Error, pcre2::Error, String>;

fn main() -> Result<(), RetErrType> {
    // regex test REGEX TEXT...
    // regex match REGEX TEXT
    // -e, --engine <engine>
    // engine: rust|pcre (default: pcre)

    trait ConfEngineArg<'a, 'b> {
        fn conf_engine_arg(self) -> Self;
    }

    impl<'a, 'b> ConfEngineArg<'a, 'b> for App<'a, 'b> {
        fn conf_engine_arg(self) -> Self {
            self.arg(
                Arg::with_name("engine")
                    .short("e")
                    .long("engine")
                    .value_name("engine")
                    .default_value("pcre")
                    .required(false)
                    .help("Set the used regular expression engine (rust|pcre)"),
            )
        }
    }

    let matches = App::new("regex")
        .author("bczhc <bczhc0@126.com>")
        .about("An application to do regular expression \"text\" or \"match\" operation")
        .subcommand(
            SubCommand::with_name("test")
                .conf_engine_arg()
                .arg(Arg::with_name("regex").value_name("regex").required(true))
                .arg(
                    Arg::with_name("texts")
                        .value_name("text")
                        .required(false)
                        .multiple(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("match")
                .conf_engine_arg()
                .arg(Arg::with_name("regex").value_name("regex").required(true))
                .arg(Arg::with_name("text").value_name("text").required(true)),
        )
        .get_matches();

    let get_regex = |e: Engine, regex: &str| -> Result<Box<dyn Regex>, RetErrType> {
        match e {
            Engine::Rust => {
                let r = RustRegex::new(regex);
                if let Err(e) = r {
                    return Err(RetErr::Rust(e));
                }
                Ok(Box::new(r.unwrap()) as Box<dyn Regex>)
            }
            Engine::PCRE => {
                let r = PcreRegex::new(regex);
                if let Err(e) = r {
                    return Err(RetErr::PCRE(e));
                }
                Ok(Box::new(r.unwrap()) as Box<dyn Regex>)
            }
        }
    };

    let get_engine = |engine: &str| -> Result<Engine, RetErrType> {
        match engine.to_lowercase().as_str() {
            "rust" => Ok(Engine::Rust),
            "pcre" => Ok(Engine::PCRE),
            _ => Err(RetErr::Text(format!("Unknown engine: {}", engine))),
        }
    };

    let subcommands = matches.subcommand();
    match subcommands {
        ("test", matcher) => {
            let matcher = matcher.unwrap();
            let engine = get_engine(matcher.value_of("engine").unwrap())?;
            let regex = matcher.value_of("regex").unwrap();
            let regex = get_regex(engine, regex)?;
            let regex = regex.as_ref() as &dyn Regex;

            let texts = matcher.values_of("texts");
            if let None = texts {
                return Ok(());
            }
            let texts = texts.unwrap();

            for x in texts {
                let test_result = regex.test(x);
                println!("{}: {}", x, test_result);
            }
        }
        ("match", matcher) => {
            let matcher = matcher.unwrap();
            let text = matcher.value_of("text").unwrap();
            let regex = matcher.value_of("regex").unwrap();
            let engine = get_engine(matcher.value_of("engine").unwrap())?;
            let regex = get_regex(engine, regex)?;
            let regex = regex.as_ref() as &dyn Regex;
            regex.print_matches(text);
        }
        (_, _) => {}
    }

    Ok(())
}

trait Regex {
    fn test(&self, text: &str) -> bool;

    fn print_matches(&self, text: &str);
}

struct RustRegex {
    regex: regex::Regex,
}

impl RustRegex {
    fn new(regex: &str) -> Result<Self, regex::Error> {
        let r = regex::Regex::new(regex)?;
        Ok(Self { regex: r })
    }
}

impl Regex for RustRegex {
    fn test(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    fn print_matches(&self, text: &str) {
        let matches = self.regex.captures_iter(text);
        for groups in matches {
            let len = groups.len();
            print!("[");
            if len != 0 {
                print!("{}", groups.get(0).unwrap().as_str());
                for i in 1..len {
                    print!(", {}", groups.get(i).unwrap().as_str());
                }
            }
            println!("]");
        }
    }
}

struct PcreRegex {
    regex: pcre2::bytes::Regex,
}

impl PcreRegex {
    fn new(regex: &str) -> Result<Self, pcre2::Error> {
        let result = pcre2::bytes::RegexBuilder::new().utf(true).build(regex);
        let r = result?;
        Ok(Self { regex: r })
    }
}

impl Regex for PcreRegex {
    fn test(&self, text: &str) -> bool {
        self.regex.is_match(text.as_bytes()).unwrap()
    }

    fn print_matches(&self, text: &str) {
        let matches = self.regex.captures_iter(text.as_bytes());
        for x in matches {
            let groups = x.unwrap();
            let len = groups.len();
            print!("[");
            if len != 0 {
                print!(
                    "{}",
                    String::from_utf8_lossy(groups.get(0).unwrap().as_bytes())
                );
                for i in 1..len {
                    print!(
                        ", {}",
                        String::from_utf8_lossy(groups.get(i).unwrap().as_bytes())
                    );
                }
            }
            println!("]");
        }
    }
}

enum Engine {
    Rust,
    PCRE,
}
