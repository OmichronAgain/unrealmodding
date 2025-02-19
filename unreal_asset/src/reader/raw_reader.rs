use std::{
    collections::HashMap,
    io::{self, Cursor, Read, Seek},
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{
    cursor_ext::CursorExt,
    custom_version::{CustomVersion, CustomVersionTrait},
    error::Error,
    unreal_types::{FName, Guid, PackageIndex},
    Import,
};

use super::{asset_reader::AssetReader, asset_trait::AssetTrait};

pub struct RawReader {
    cursor: Cursor<Vec<u8>>,
    engine_version: i32,

    empty_map: HashMap<String, String>,
}

impl RawReader {
    pub fn new(cursor: Cursor<Vec<u8>>, engine_version: i32) -> Self {
        RawReader {
            cursor,
            engine_version,
            empty_map: HashMap::new(),
        }
    }
}

impl AssetTrait for RawReader {
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        CustomVersion::new([0u8; 16], 0)
    }

    fn position(&self) -> u64 {
        self.cursor.position()
    }

    fn set_position(&mut self, pos: u64) {
        self.cursor.set_position(pos)
    }

    fn seek(&mut self, style: io::SeekFrom) -> io::Result<u64> {
        self.cursor.seek(style)
    }

    fn get_map_key_override(&self) -> &HashMap<String, String> {
        &self.empty_map
    }

    fn get_map_value_override(&self) -> &HashMap<String, String> {
        &self.empty_map
    }

    fn get_engine_version(&self) -> i32 {
        self.engine_version
    }

    fn get_import(&self, _index: PackageIndex) -> Option<&Import> {
        None
    }

    fn get_export_class_type(&self, _index: PackageIndex) -> Option<FName> {
        None
    }
}

impl AssetReader for RawReader {
    fn read_property_guid(&mut self) -> Result<Option<Guid>, Error> {
        Ok(None)
    }

    fn read_fname(&mut self) -> Result<FName, Error> {
        let string = self.read_string()?.unwrap_or_else(|| "None".to_string());
        Ok(FName::new(string, 0))
    }

    fn read_array_with_length<T>(
        &mut self,
        length: i32,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let mut result = Vec::new();
        for _ in 0..length {
            result.push(getter(self)?);
        }
        Ok(result)
    }

    fn read_array<T>(
        &mut self,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let length = self.read_i32::<LittleEndian>()?;
        self.read_array_with_length(length, getter)
    }

    fn read_u8(&mut self) -> Result<u8, io::Error> {
        self.cursor.read_u8()
    }

    fn read_i8(&mut self) -> Result<i8, io::Error> {
        self.cursor.read_i8()
    }

    fn read_u16<T: byteorder::ByteOrder>(&mut self) -> Result<u16, io::Error> {
        self.cursor.read_u16::<T>()
    }

    fn read_i16<T: byteorder::ByteOrder>(&mut self) -> Result<i16, io::Error> {
        self.cursor.read_i16::<T>()
    }

    fn read_u32<T: byteorder::ByteOrder>(&mut self) -> Result<u32, io::Error> {
        self.cursor.read_u32::<T>()
    }

    fn read_i32<T: byteorder::ByteOrder>(&mut self) -> Result<i32, io::Error> {
        self.cursor.read_i32::<T>()
    }

    fn read_u64<T: byteorder::ByteOrder>(&mut self) -> Result<u64, io::Error> {
        self.cursor.read_u64::<T>()
    }

    fn read_i64<T: byteorder::ByteOrder>(&mut self) -> Result<i64, io::Error> {
        self.cursor.read_i64::<T>()
    }

    fn read_f32<T: byteorder::ByteOrder>(&mut self) -> Result<f32, io::Error> {
        self.cursor.read_f32::<T>()
    }

    fn read_f64<T: byteorder::ByteOrder>(&mut self) -> Result<f64, io::Error> {
        self.cursor.read_f64::<T>()
    }

    fn read_string(&mut self) -> Result<Option<String>, Error> {
        self.cursor.read_string()
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), io::Error> {
        self.cursor.read_exact(buf)
    }

    fn read_bool(&mut self) -> Result<bool, Error> {
        Ok(self.read_u8()? != 0)
    }
}
