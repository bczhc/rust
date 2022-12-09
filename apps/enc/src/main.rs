use std::io;
use std::io::{stdin, stdout, ErrorKind, Read, Write};

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit, KeySizeUser};
use aes::Aes256;
use argon2::password_hash::{Salt, SaltString};
use argon2::PasswordHasher;
use cipher::consts::U32;
use cipher::BlockDecrypt;
use rand::rngs::OsRng;

use bczhc_lib::io::TryReadExact;
use enc::cli::build_cli;
use enc::errors::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = build_cli().get_matches();
    let decryption_mode = matches.get_flag("decrypt");

    /*let password = rpassword::prompt_password("enter password: ")?;
    if rpassword::prompt_password("enter again to verify: ")? != password {
        return Err(Error::PasswordNotMatch.into());
    }*/

    let password = "1";

    let mut stdin = stdin();
    let mut stdout = stdout();

    let salt = if decryption_mode {
        read_salt(&mut stdin)?
    } else {
        SaltString::generate(OsRng)
    };
    let key = derive_key(password, &salt)?;

    let cipher = Aes256::new(&key);

    if decryption_mode {
        decrypt_to(cipher, &mut stdin, &mut stdout)?;
    } else {
        // bring salt to the encrypted output
        // zero-terminated
        stdout.write_all(salt.as_bytes())?;
        stdout.write_all(&[0_u8])?;
        encrypt_to(cipher, &mut stdin, &mut stdout)?;
    }

    Ok(())
}

pub fn read_salt(reader: &mut impl Read) -> Result<SaltString> {
    let mut buf = Vec::new();
    for x in reader.bytes() {
        match x? {
            0 => {
                break;
            }
            b => {
                buf.push(b);
            }
        }
    }
    let read = String::from_utf8(buf).map_err(|_| Error::InvalidSalt)?;
    SaltString::new(&read).map_err(|_| Error::InvalidSalt)
}

pub fn derive_key(password: &str, salt: &SaltString) -> Result<GenericArray<u8, U32>> {
    let password_hash = argon2::Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(PasswordHashError::from)?;
    let password_hash_bytes = password_hash.hash.unwrap();
    let mut password_hash = GenericArray::from([0_u8; 32]);
    assert_eq!(Aes256::key_size(), password_hash_bytes.len());
    password_hash.copy_from_slice(password_hash_bytes.as_bytes());
    Ok(password_hash)
}

pub fn encrypt_to<R, W>(cipher: Aes256, from: &mut R, to: &mut W) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    let mut block = GenericArray::from([0_u8; 16]);

    loop {
        let read_size = from.try_read_exact(&mut block)?;
        if read_size == 16 {
            // full block
            cipher.encrypt_block(&mut block);
            to.write_all(&block)?;
        } else if read_size != 0 {
            // EOF, partial buf, do padding
            let pad_byte = (16 - read_size) as u8;
            block[read_size..].iter_mut().for_each(|x| *x = pad_byte);
            cipher.encrypt_block(&mut block);
            to.write_all(&block)?;
            break;
        } else if read_size == 0 {
            // EOF, no data is read. but this previous loop reads a whole buffer
            // do a full block padding
            let pad_byte = block.len() as u8;
            block.iter_mut().for_each(|x| *x = pad_byte);
            cipher.encrypt_block(&mut block);
            to.write_all(&block)?;
            break;
        }
    }
    Ok(())
}

pub fn decrypt_to<R, W>(cipher: Aes256, from: &mut R, to: &mut W) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    let mut init = true;
    let mut block = GenericArray::from([0_u8; 16]);
    let mut buf = GenericArray::from([0_u8; 16]);
    loop {
        if let Err(e) = from.read_exact(&mut buf) {
            if e.kind() != ErrorKind::UnexpectedEof {
                return Err(e);
            }
            // EOF
            // check padding and data integrity
            let last = *block.last().unwrap();
            if last as usize == block.len() {
                // a full block with the byte of B; B is the block size
                let mut ok = true;
                for x in block {
                    if x != last {
                        ok = false;
                    }
                }
                if !ok {
                    // TODO: return this error instead of panicking
                    panic!("Padding check error");
                }
            } else {
                // format like: | D D D D D 03 03 03 |
                let mut ok = true;
                for x in &block[(block.len() - last as usize)..] {
                    if *x != last {
                        ok = false;
                    }
                }
                if !ok {
                    panic!("Padding check error");
                }
            }
            // integrity check succeeds
            if last as usize != block.len() {
                // still have data to write; write the left
                to.write_all(&block[..(block.len() - last as usize)])?;
            }
            break;
        }
        if !init {
            // write the previous decrypted block if existing (that's, except the initial loop)
            to.write_all(&block)?;
        }
        // just assume the ciphertext length is a multiple of the block size
        // so it reads full blocks
        cipher.decrypt_block_b2b(&mut buf, &mut block);
        init = false;
    }

    Ok(())
}
