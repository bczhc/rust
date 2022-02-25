use clap::{App, Arg, SubCommand};

#[derive(Debug)]
enum RetErr<A, B, C> {
    Rust(A),
    Pcre(B),
    Text(C),
}

type RetErrType = RetErr<regex::Error, pcre2::Error, String>;

fn main() -> Result<(), RetErrType> {
    // regex test REGEX TEXT...
    // regex match REGEX TEXT [-s X,Y]
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
                .arg(Arg::with_name("text").value_name("text").required(true))
                .arg(
                    Arg::with_name("seek")
                        .long("seek")
                        .short("s")
                        .value_name("seek")
                        .help("Print the matched result with a specified position")
                        .long_help(
                            "Format: M,G. M is the index in matches; G is the index in groups.",
                        ),
                ),
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
            Engine::Pcre => {
                let r = PcreRegex::new(regex);
                if let Err(e) = r {
                    return Err(RetErr::Pcre(e));
                }
                Ok(Box::new(r.unwrap()) as Box<dyn Regex>)
            }
        }
    };

    let get_engine = |engine: &str| -> Result<Engine, RetErrType> {
        match engine.to_lowercase().as_str() {
            "rust" => Ok(Engine::Rust),
            "pcre" => Ok(Engine::Pcre),
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
            if texts.is_none() {
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
            let matches_seek = matcher.value_of("seek");

            let engine = get_engine(matcher.value_of("engine").unwrap())?;
            let regex = get_regex(engine, regex)?;
            let regex = regex.as_ref() as &dyn Regex;
            let matches = regex.capture(text);

            match matches_seek {
                Some(seek) => {
                    let seek_matches = RustRegex::new("([0-9]+)(, *| +)([0-9]+)")
                        .unwrap()
                        .capture(seek);
                    let format_check = !seek_matches.is_empty();
                    if !format_check {
                        return Err(RetErr::Text(String::from("Wrong seek format")));
                    }
                    let groups = &seek_matches[0];
                    assert_eq!(groups.len(), 4);
                    let match_index: usize = groups[1].parse().unwrap();
                    let group_index: usize = groups.last().unwrap().parse().unwrap();
                    println!("{}", matches[match_index][group_index]);
                }
                None => {
                    println!("{:#?}", matches);
                }
            }
        }
        (_, _) => {}
    }

    Ok(())
}

trait Regex {
    fn test(&self, text: &str) -> bool;

    fn capture(&self, text: &str) -> Vec<Vec<String>>;
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

    fn capture(&self, text: &str) -> Vec<Vec<String>> {
        let mut result = Vec::new();
        let matches = self.regex.captures_iter(text);
        for groups in matches {
            let mut v = Vec::new();
            for captured in groups.iter() {
                v.push(String::from(captured.unwrap().as_str()));
            }
            result.push(v);
        }
        result
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

    fn capture(&self, text: &str) -> Vec<Vec<String>> {
        let mut result = Vec::new();
        let matches = self.regex.captures_iter(text.as_bytes());
        for groups in matches {
            let groups = groups.unwrap();
            let len = groups.len();
            let mut v = Vec::new();
            for i in 0..len {
                let matched = groups.get(i).unwrap();
                v.push(String::from_utf8_lossy(matched.as_bytes()).to_string());
            }
            result.push(v);
        }
        result
    }
}

enum Engine {
    Rust,
    Pcre,
}
