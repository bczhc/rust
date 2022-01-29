use hound::WavReader;
use bczhc_lib::utils::get_args_without_self_path;

fn main() {
    let args = get_args_without_self_path();
    let path = &args[0];

    let reader = WavReader::open(path);
    let samples_len = reader.unwrap().samples::<i32>().len();
    println!("{}", samples_len);
}