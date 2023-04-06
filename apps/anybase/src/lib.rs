use anyhow::anyhow;
use num::{BigUint, Integer, ToPrimitive, Zero};
use std::collections::HashMap;

pub mod cli;

pub fn to_base10(data: &[u8], from_base: u16) -> BigUint {
    let mut sum = BigUint::from(0_u8);
    for (i, &b) in data.iter().rev().enumerate() {
        sum += BigUint::from(b) * BigUint::from(from_base).pow(i as u32);
    }
    sum
}

pub fn base10_to_other(base10: BigUint, to_base: u16) -> Vec<u8> {
    let mut num = base10;
    let mut digits = Vec::new();
    loop {
        let (q, r) = num.div_rem(&BigUint::from(to_base));
        let terminate = q.is_zero();
        num = q;
        digits.push(r.to_u8().unwrap());
        if terminate {
            break;
        }
    }
    digits.reverse();
    digits
}

pub fn map_bytes_to_string(bytes: &[u8], table: &[char]) -> String {
    let mut result = String::new();
    for &b in bytes {
        result.push(table[b as usize]);
    }
    result
}

pub fn map_string_to_bytes(string: &str, table: &[char]) -> Vec<u8> {
    let mut reverse_map = HashMap::<char, u8>::new();
    for (i, &c) in table.iter().enumerate() {
        reverse_map.insert(c, i as u8);
    }
    let mut bytes = Vec::new();
    for c in string.chars() {
        let &b = reverse_map
            .get(&c)
            .unwrap_or_else(|| panic!("Missing characters in mapping table: {}", c));
        bytes.push(b);
    }
    bytes
}

pub fn check_mapping_table(table: &str, base: u16) -> anyhow::Result<()> {
    let chars = table.chars().collect::<Vec<_>>();
    if chars.len() != base as usize {
        return Err(anyhow!("Alphabet table not equal to base"));
    }
    let has_duplicates = (1..chars.len()).any(|i| chars[i..].contains(&chars[i - 1]));
    if has_duplicates {
        return Err(anyhow!("Alphabet table characters should be all unique"));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{base10_to_other, map_bytes_to_string, map_string_to_bytes, to_base10};
    use num::BigUint;

    #[test]
    fn test() {
        let data = b"hello, world";
        assert_eq!(base10_to_other(to_base10(data, 256), 256), data);

        assert_eq!(
            map_bytes_to_string(&[0, 1, 2, 3, 4], &['a', 'b', 'c', 'd', 'e']),
            "abcde"
        );

        assert_eq!(
            &map_string_to_bytes("abcde", &['a', 'b', 'c', 'd', 'e']),
            &[0, 1, 2, 3, 4]
        );

        let base58_table = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
            .chars()
            .collect::<Vec<_>>();

        assert_eq!(to_base10(b"abc", 256), BigUint::from(6382179_u32));

        assert_eq!(
            map_bytes_to_string(
                &base10_to_other(BigUint::from(6382179_u32), 58),
                &base58_table
            ),
            "ZiCa"
        );
    }
}
