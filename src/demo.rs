fn main() {
    let s = "Bin输入🍎";
    for x in s.as_bytes() {
        let mut s = format!("{:0>8b}", x);
        s.insert(4, ' ');
        println!("{}", s);
    }
}
