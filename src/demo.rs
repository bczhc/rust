use clap::{App, Arg, ArgGroup};

fn main() {
    let matches = App::new("My Super Program")
        .version("1.0.0")
        .author("bczhc <bczhc0@126.com>")
        .about("My first CLI program")
        .arg(Arg::with_name("src").value_name("src-path").required(true))
        .arg(
            Arg::with_name("dest")
                .value_name("dest-path")
                .required(true),
        )
        .get_matches();

    let src_path = matches.value_of("src").unwrap();
    let dest_path = matches.value_of("dest").unwrap();

    println!("{:?}", (src_path, dest_path));
}
