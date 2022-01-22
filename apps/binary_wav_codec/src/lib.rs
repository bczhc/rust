#![feature(generic_const_exprs)]

use crate::errors::FormatError;
use crate::int_reader::FileIntReader;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use hound::{Error, SampleFormat, WavReader, WavSpec, WavWriter};
use std::io::{Cursor, Read, Seek, Write};

type Endianness = LittleEndian;

pub mod errors;
pub mod int_reader;

pub type Result<O> = std::result::Result<O, errors::Error>;

pub fn encode_stream<R, W>(reader: &mut R, writer: &mut W) -> Result<()>
where
    R: Read,
    W: Write + Seek,
{
    let mut read_len = 0;

    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 24,
        sample_format: SampleFormat::Int,
    };

    let mut wav_writer = WavWriter::new(writer, spec)?;

    let iter = FileIntReader::new_iterator::<int_reader::int_type::I24, Endianness, _>(reader);
    for i in iter {
        wav_writer.write_sample(i.0)?;
        read_len += i.1;
    }

    let length_header = split_u64_to_three_i24(read_len as u64);
    wav_writer.write_sample(length_header.0)?;
    wav_writer.write_sample(length_header.1)?;
    wav_writer.write_sample(length_header.2)?;

    Ok(())
}

pub fn decode_stream<R, W>(reader: &mut R, writer: &mut W) -> Result<()>
where
    R: Read,
    W: Write,
{
    let mut wav_reader = WavReader::new(reader)?;
    let mut samples = wav_reader.samples::<i32>();
    let sample_len = samples.len();

    // bytes are the original data until the 9-byte (three 24-bit samples) length info header at the end
    let data_sample_length = sample_len - 3;

    // because the original data size is unknown, so there should set a writing buffer so that
    // it can give us a one-loop-time to undo or adjust the end bytes,
    // to ensure the final output data is same as the original data (no additional ending zero bytes)
    let mut write_buf = [0_u8; 3];

    for j in 0..data_sample_length {
        match samples.next() {
            None => {
                return Err(FormatError::NoEnoughSamples.into());
            }
            Some(i) => {
                if j != 0 {
                    // new loop comes, meaning it not reach the end
                    // flush the writing buffer
                    writer.write(&write_buf)?;
                }

                let sample = i?;
                Endianness::write_i24(&mut write_buf, sample);
            }
        }
    }

    // reaching the end; handle the end bytes stored in the writing buffer
    let r1 = samples.next().unwrap()?;
    let r2 = samples.next().unwrap()?;
    let r3 = samples.next().unwrap()?;
    let data_length = compose_three_i24_to_u64((r1, r2, r3));
    assert!(data_length <= (data_sample_length * 3) as u64);

    let remain_len = 3_usize - (data_sample_length * 3 - data_length as usize);
    assert!(remain_len > 0 && remain_len <= 3);

    writer.write(&write_buf[0..remain_len])?;

    Ok(())
}

fn split_u64_to_three_i24(a: u64) -> (i32, i32, i32) {
    let mut buf = [0_u8; 9];
    Endianness::write_u64(&mut buf, a);
    buf[8] = 0;
    let mut cursor = Cursor::new(buf);
    let r1 = cursor.read_i24::<Endianness>().unwrap();
    let r2 = cursor.read_i24::<Endianness>().unwrap();
    let r3 = cursor.read_i24::<Endianness>().unwrap();
    (r1, r2, r3)
}

fn compose_three_i24_to_u64(a: (i32, i32, i32)) -> u64 {
    let mut buf = [0_u8; 9];
    Endianness::write_i24(&mut buf[0..3], a.0);
    Endianness::write_i24(&mut buf[3..6], a.1);
    Endianness::write_i24(&mut buf[6..9], a.2);
    assert_eq!(buf[8], 0);
    Endianness::read_u64(&buf)
}

#[cfg(test)]
mod test {
    use crate::{decode_stream, encode_stream};
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test() {
        let data_vec = (1..=100)
            .map(|x| (1..=x).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        for vec in data_vec {
            let mut output = Cursor::new(Vec::new());
            let mut reader = Cursor::new(&vec[..]);
            encode_stream(&mut reader, &mut output).unwrap();
            output.seek(SeekFrom::Start(0)).unwrap();

            let mut writer = Cursor::new(Vec::new());
            decode_stream(&mut output, &mut writer).unwrap();
            writer.seek(SeekFrom::Start(0)).unwrap();

            assert_eq!(writer.into_inner(), vec);
        }
    }
}
