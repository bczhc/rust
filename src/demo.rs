use std::cmp::max;
use std::collections::LinkedList;

struct Solution;

impl Solution {
    pub fn add_binary(a: String, b: String) -> String {
        let mut carry = false;
        let a = a.as_bytes();
        let b = b.as_bytes();
        let a_len = a.len() as isize;
        let b_len = b.len() as isize;

        let mut result = LinkedList::new();

        let t = max(a_len, b_len);
        for i in 0..t {
            let index = a_len - 1 - i;
            let a_bit = if index < 0 { false } else { a[index as usize] == b'1' };
            let index = b_len - 1 - i;
            let b_bit = if index < 0 { false } else { b[index as usize] == b'1' };

            let half_add_bit = a_bit ^ b_bit;
            let full_add_bit = half_add_bit ^ carry;
            result.push_front(if full_add_bit { b'1' } else { b'0' });

            carry = (a_bit && b_bit) || (half_add_bit && carry);
        }

        if carry {
            result.push_front(b'1');
        }

        unsafe { String::from_utf8_unchecked(result.into_iter().collect()) }
    }
}

fn main() {
    println!(
        "{}",
        Solution::add_binary(String::from("1110101010"), String::from("010101011"))
    );
}
