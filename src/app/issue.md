stdin().read(&mut [u8]) reads inefficient data

Now I have a file with enough size to read, but when I use `stdin().read(...)`, I found the read size it returned has wrong size.

Code:
```rust
fn main() {
    let mut f = stdin();
    let mut buf: [u8; 3] = [0, 0, 0];
    let mut c = 0;
    loop {
        let i = f.read(&mut buf).unwrap();
        if i == 0 { break; }
        println!("{} {}", i, c);
        c += 1;
    }   
}
```
Command: `cat some-file | ./main`

The output will look like this:
>3 1
>
>3 2
>
>3 3
>
>... Omit a lot of 3s'
>
>3 2729
>
>2 2730 // here: it reads only two bytes but not the expected size 3
>
>3 2731
>
>3 2731
>
> ...
>
>2 5461 // the same case
>
>3
>
>3
>
>...

Command: `cat some-file | ./main | grep "2 "`
Output:
>2 2730
>
>2 5461
>
>2 8192
>
>2 10923
>
>2 13654
>
>2 16385
>
>2 19116
>
>2 21847
>
>2 24578
>
>2 27309
>
>2 30040
>
>...

I don't know whether it's a bug or there is another right way to read bytes from stdin so that the case above can be avoided.