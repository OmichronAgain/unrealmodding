use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::implement_get;
use crate::uasset::Asset;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::exports::normal_export::NormalExport;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::unreal_types::{NamespacedString};
use crate::uasset::error::Error;
use crate::uasset::exports::ExportTrait;
use super::ExportNormalTrait;
use super::ExportUnknownTrait;

pub struct LevelExport {
    normal_export: NormalExport,

    index_data: Vec<i32>,
    level_type: NamespacedString,
    flags_probably: u64,
    misc_category_data: Vec<i32>
}

implement_get!(LevelExport);

impl LevelExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset, next_starting: u64) -> Result<Self, Error> {
        let normal_export = NormalExport::from_unk(unk, asset)?;

        asset.cursor.read_i32::<LittleEndian>()?;

        let num_index_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let mut index_data = Vec::with_capacity(num_index_entries as usize);
        for _i in 0..num_index_entries as usize {
            index_data.push(asset.cursor.read_i32::<LittleEndian>()?);
        }

        let nms = asset.cursor.read_string()?;
        asset.cursor.read_i32::<LittleEndian>()?; // null
        let val = asset.cursor.read_string()?;
        let level_type = NamespacedString::new(nms, val);

        asset.cursor.read_i64::<LittleEndian>()?; // null
        let flags_probably = asset.cursor.read_u64::<LittleEndian>()?;
        let mut misc_category_data = Vec::new();
        while asset.cursor.position() < next_starting - 1 {
            misc_category_data.push(asset.cursor.read_i32::<LittleEndian>()?);
        }
        asset.cursor.read_exact(&mut [0u8; 1])?;

        Ok(LevelExport {
            normal_export,
            index_data,
            level_type,
            flags_probably,
            misc_category_data
        })
    }
}

impl ExportTrait for LevelExport {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        self.normal_export.write(asset, cursor)?;

        cursor.write_i32::<LittleEndian>(0)?;
        cursor.write_i32::<LittleEndian>(self.index_data.len() as i32)?;
        for index in &self.index_data {
            cursor.write_i32::<LittleEndian>(*index)?;
        }

        cursor.write_string(&self.level_type.namespace)?;
        cursor.write_i32::<LittleEndian>(0)?;
        cursor.write_string(&self.level_type.value)?;

        cursor.write_u64::<LittleEndian>(0)?;
        cursor.write_u64::<LittleEndian>(self.flags_probably)?;

        for data in &self.misc_category_data {
            cursor.write_i32::<LittleEndian>(*data)?;
        }
        cursor.write_u8(0)?;
        Ok(())
    }
}