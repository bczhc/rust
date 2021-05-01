extern crate curl;

use curl::easy::Easy;
use std::cell::RefCell;

fn main() {
    let r = read_url_to_string("http://61.177.44.242:8080/BusSysWebService/bus/allStationOfRPName?name=6");

    let json = json::parse(r.as_str()).unwrap();
    let lines = &json["result"]["lines"];
    let len = lines.len();
    for i in 0..len {
        let line = &lines[i];
        let bus_name = &line["runPathName"];
        println!("{}", bus_name);
    }
}

fn read_url_to_string(url: &str) -> String {
    let s = RefCell::new(String::new());

    let mut easy = Easy::new();
    easy.url(url).unwrap();

    let mut transfer = easy.transfer();

    transfer.write_function(|data| {
        s.borrow_mut().push_str(String::from_utf8_lossy(data).as_ref());
        return Ok(data.len());
    }).unwrap();
    transfer.perform().unwrap();

    return s.take();
}