use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    io::{self, SeekFrom},
};

use byteorder::LittleEndian;

use crate::{
    custom_version::{CustomVersion, CustomVersionTrait},
    error::Error,
    reader::asset_reader::AssetReader,
    reader::asset_trait::AssetTrait,
    ue4version::VER_UE4_NAME_HASHES_SERIALIZED,
    unreal_types::{FName, Guid, PackageIndex},
    Import,
};

pub struct NameTableReader<'reader, Reader: AssetReader> {
    reader: &'reader mut Reader,
    pub(crate) name_map: Vec<String>,
    pub(crate) name_map_lookup: HashMap<u64, i32>,
}

impl<'reader, Reader: AssetReader> NameTableReader<'reader, Reader> {
    pub(crate) fn new(reader: &'reader mut Reader) -> Result<Self, Error> {
        let name_offset = reader.read_i64::<LittleEndian>()?;
        // todo: length checking

        let mut name_map = Vec::new();
        let mut name_map_lookup = HashMap::new();
        if name_offset > 0 {
            let original_offset = reader.position();
            reader.seek(SeekFrom::Start(name_offset as u64))?;

            let name_count = reader.read_i32::<LittleEndian>()?;
            if name_count < 0 {
                return Err(Error::invalid_file("Corrupted file".to_string()));
            }

            for i in 0..name_count {
                let mut s = DefaultHasher::new();

                let name = reader.read_string()?.ok_or_else(|| {
                    Error::invalid_file(format!("Name table entry {} is missing a name", i))
                })?;
                name.hash(&mut s);
                name_map_lookup.insert(s.finish(), i);

                match reader.get_engine_version() >= VER_UE4_NAME_HASHES_SERIALIZED {
                    true => {
                        let _non_case_preserving_hash = reader.read_u16::<LittleEndian>()?;
                        let _case_preserving_hash = reader.read_u16::<LittleEndian>()?;
                    }
                    false => {}
                };

                name_map.push(name);
            }

            reader.seek(SeekFrom::Start(original_offset))?;
        }
        Ok(NameTableReader {
            name_map,
            reader,
            name_map_lookup,
        })
    }
}

impl<'reader, Reader: AssetReader> AssetTrait for NameTableReader<'reader, Reader> {
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        self.reader.get_custom_version::<T>()
    }

    fn position(&self) -> u64 {
        self.reader.position()
    }

    fn set_position(&mut self, pos: u64) {
        self.reader.set_position(pos)
    }

    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        self.reader.seek(style)
    }

    fn get_map_key_override(&self) -> &HashMap<String, String> {
        self.reader.get_map_key_override()
    }

    fn get_map_value_override(&self) -> &HashMap<String, String> {
        self.reader.get_map_value_override()
    }

    fn get_engine_version(&self) -> i32 {
        self.reader.get_engine_version()
    }

    fn get_import(&self, index: PackageIndex) -> Option<&Import> {
        self.reader.get_import(index)
    }

    fn get_export_class_type(&self, index: PackageIndex) -> Option<FName> {
        self.reader.get_export_class_type(index)
    }
}

impl<'reader, Reader: AssetReader> AssetReader for NameTableReader<'reader, Reader> {
    fn read_property_guid(&mut self) -> Result<Option<Guid>, Error> {
        self.reader.read_property_guid()
    }

    fn read_fname(&mut self) -> Result<FName, Error> {
        let name_index = self.reader.read_i32::<LittleEndian>()?;
        let number = self.reader.read_i32::<LittleEndian>()?;
        // todo: length checks

        let name = self.name_map[name_index as usize].clone();

        Ok(FName::new(name, number))
    }

    fn read_array_with_length<T>(
        &mut self,
        length: i32,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let mut array = Vec::with_capacity(length as usize);
        for i in 0..length {
            if i == 2 {
                println!("HERE");
            }
            array.push(getter(self)?);
        }
        Ok(array)
    }

    fn read_array<T>(
        &mut self,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let length = self.reader.read_i32::<LittleEndian>()?;
        self.read_array_with_length(length, getter)
    }

    fn read_u8(&mut self) -> Result<u8, io::Error> {
        self.reader.read_u8()
    }

    fn read_i8(&mut self) -> Result<i8, io::Error> {
        self.reader.read_i8()
    }

    fn read_u16<T: byteorder::ByteOrder>(&mut self) -> Result<u16, io::Error> {
        self.reader.read_u16::<T>()
    }

    fn read_i16<T: byteorder::ByteOrder>(&mut self) -> Result<i16, io::Error> {
        self.reader.read_i16::<T>()
    }

    fn read_u32<T: byteorder::ByteOrder>(&mut self) -> Result<u32, io::Error> {
        self.reader.read_u32::<T>()
    }

    fn read_i32<T: byteorder::ByteOrder>(&mut self) -> Result<i32, io::Error> {
        self.reader.read_i32::<T>()
    }

    fn read_u64<T: byteorder::ByteOrder>(&mut self) -> Result<u64, io::Error> {
        self.reader.read_u64::<T>()
    }

    fn read_i64<T: byteorder::ByteOrder>(&mut self) -> Result<i64, io::Error> {
        self.reader.read_i64::<T>()
    }

    fn read_f32<T: byteorder::ByteOrder>(&mut self) -> Result<f32, io::Error> {
        self.reader.read_f32::<T>()
    }

    fn read_f64<T: byteorder::ByteOrder>(&mut self) -> Result<f64, io::Error> {
        self.reader.read_f64::<T>()
    }

    fn read_string(&mut self) -> Result<Option<String>, Error> {
        self.reader.read_string()
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), io::Error> {
        self.reader.read_exact(buf)
    }

    fn read_bool(&mut self) -> Result<bool, Error> {
        self.reader.read_bool()
    }
}
