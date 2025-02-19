use crate::error::Error;
use crate::exports::base_export::BaseExport;
use crate::exports::{ExportBaseTrait, ExportTrait};
use crate::reader::asset_reader::AssetReader;
use crate::reader::asset_writer::AssetWriter;

use super::ExportNormalTrait;

#[derive(Clone)]
pub struct RawExport {
    pub base_export: BaseExport,

    pub data: Vec<u8>,
}

impl ExportNormalTrait for RawExport {
    fn get_normal_export(&'_ self) -> Option<&'_ super::normal_export::NormalExport> {
        None
    }

    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut super::normal_export::NormalExport> {
        None
    }
}

impl ExportBaseTrait for RawExport {
    fn get_base_export(&'_ self) -> &'_ BaseExport {
        &self.base_export
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport {
        &mut self.base_export
    }
}

impl RawExport {
    pub fn from_base<Reader: AssetReader>(
        base: BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let mut data = vec![0u8; base.serial_size as usize];
        asset.read_exact(&mut data)?;

        Ok(RawExport {
            base_export: base,
            data,
        })
    }
}

impl ExportTrait for RawExport {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_all(&self.data)?;
        Ok(())
    }
}
