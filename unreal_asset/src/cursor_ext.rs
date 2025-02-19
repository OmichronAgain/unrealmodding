use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::mem::size_of;

use super::error::Error;

pub trait CursorExt {
    fn read_string(&mut self) -> Result<Option<String>, Error>;
    fn read_bool(&mut self) -> Result<bool, Error>;

    fn write_string(&mut self, string: &Option<String>) -> Result<usize, Error>;
    fn write_bool(&mut self, value: bool) -> Result<(), Error>;
}

impl CursorExt for Cursor<Vec<u8>> {
    // read string of format <length i32><string><null>
    fn read_string(&mut self) -> Result<Option<String>, Error> {
        let len = self.read_i32::<LittleEndian>()?;
        if len == i32::MIN {
            return Err(Error::invalid_file(format!(
                "Tried to read string with length {}",
                len
            )));
        }

        if !(-131072..=131072).contains(&len) {
            return Err(Error::invalid_file(format!(
                "Tried to read string with length {}",
                len
            )));
        }

        if len == 0 {
            return Ok(None); // maybe it actually is an empty string?
        }

        if len < 0 {
            let len = -len;

            let len = len * size_of::<u16>() as i32 - 2;
            let mut buf = vec![0u8; len as usize];
            self.read_exact(&mut buf)?;

            let string = String::from_utf16(
                &buf.chunks(2)
                    .map(|e| u16::from_le_bytes([e[0], e[1]]))
                    .collect::<Vec<_>>(),
            )?;
            self.seek(SeekFrom::Current(2))?;

            Ok(Some(string))
        } else {
            let mut buf = vec![0u8; len as usize - 1];
            self.read_exact(&mut buf)?;
            self.seek(SeekFrom::Current(1))?;
            Ok(Some(
                String::from_utf8(buf).unwrap_or_else(|_| String::from("None")),
            ))
        }
    }

    fn read_bool(&mut self) -> Result<bool, Error> {
        let res = self.read_u8()?;
        Ok(res > 0)
    }

    fn write_string(&mut self, string: &Option<String>) -> Result<usize, Error> {
        if string.is_none() {
            self.write_i32::<LittleEndian>(0)?;
            return Ok(size_of::<i32>());
        }
        let string = string.clone().unwrap();
        let is_unicode = string.len() != string.chars().count();

        if is_unicode {
            println!("Writing unicode string at {}", self.position());
            unsafe {
                let utf16 = string.encode_utf16().collect::<Vec<_>>();
                let aligned = utf16.align_to::<u8>();
                self.write_i32::<LittleEndian>(-(aligned.1.len() as i32 / 2) - 1)?;
                self.write_all(aligned.1)?;
            }

            self.write_all(&[0u8; 2])?;
        } else {
            self.write_i32::<LittleEndian>(string.len() as i32 + 1)?;
            let bytes = string.as_bytes();
            self.write_all(bytes)?;
            self.write_all(&[0u8; 1])?;
        }

        let written = match is_unicode {
            true => string.len() * 2 + 2,
            false => string.len() + 1,
        };

        Ok(written)
    }

    fn write_bool(&mut self, value: bool) -> Result<(), Error> {
        self.write_u8(match value {
            true => 1,
            false => 0,
        })?;
        Ok(())
    }
}
