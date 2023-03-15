use prettytable::format::FormatBuilder;
use prettytable::{row, Table};
use std::io::{stdin, BufRead, Cursor};
use unicode_name::{char_utf32_string, CharInfoIter, Config, CONFIG};

fn main() -> anyhow::Result<()> {
    let matches = unicode_name::cli::build_cli().get_matches();
    // TODO: grapheme support
    let _enable_grapheme = matches.get_flag("grapheme");
    let text = matches.get_one::<String>("text");
    let ucd_database = matches.get_one::<String>("ucd db").unwrap();
    CONFIG.lock().unwrap().replace(Config {
        ucd_database: ucd_database.clone(),
    });

    let mut reader: Box<dyn BufRead> = match text {
        Some(text) => {
            let cursor = Cursor::new(text.as_bytes());
            Box::new(cursor)
        }
        None => {
            let stdin = stdin().lock();
            Box::new(stdin)
        }
    };

    let mut table = Table::new();
    table.set_format(FormatBuilder::new().column_separator('|').build());
    table.add_row(row!["character", "byte", "UTF-32", "name", "block"]);

    // TODO: print table row on each line. Now it only prints the
    //  whole table after iterating all characters (that's, at EOF)
    let iter = CharInfoIter::new(&mut reader);
    for c in iter {
        let c = c?;
        table.add_row(row![
            c.char_offset,
            c.byte_offset,
            char_utf32_string(c.char),
            c.name.unwrap_or_else(|| String::from("UNKNOWN NAME")),
            c.block.unwrap_or("UNKNOWN BLOCK")
        ]);
    }
    table.printstd();
    Ok(())
}
