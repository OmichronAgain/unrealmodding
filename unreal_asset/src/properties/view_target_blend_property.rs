use std::mem::size_of;

use byteorder::LittleEndian;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ordered_float::OrderedFloat;

use crate::error::Error;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::reader::asset_reader::AssetReader;
use crate::reader::asset_writer::AssetWriter;
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write,
    unreal_types::{FName, Guid},
};

#[derive(IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum ViewTargetBlendFunction {
    /** Camera does a simple linear interpolation. */
    VtBlendLinear,
    /** Camera has a slight ease in and ease out, but amount of ease cannot be tweaked. */
    VtBlendCubic,
    /** Camera immediately accelerates, but smoothly decelerates into the target.  Ease amount controlled by BlendExp. */
    VtBlendEaseIn,
    /** Camera smoothly accelerates, but does not decelerate into the target.  Ease amount controlled by BlendExp. */
    VtBlendEaseOut,
    /** Camera smoothly accelerates and decelerates.  Ease amount controlled by BlendExp. */
    VtBlendEaseInOut,
    VtBlendMax,
}

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct ViewTargetBlendParamsProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,

    pub blend_time: OrderedFloat<f32>,
    pub blend_function: ViewTargetBlendFunction,
    pub blend_exp: OrderedFloat<f32>,
    pub lock_outgoing: bool,
}
impl_property_data_trait!(ViewTargetBlendParamsProperty);

impl ViewTargetBlendParamsProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let blend_time = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let blend_function = ViewTargetBlendFunction::try_from(asset.read_u8()?)?;
        let blend_exp = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let lock_outgoing = asset.read_i32::<LittleEndian>()? != 0;

        Ok(ViewTargetBlendParamsProperty {
            name,
            property_guid,
            duplication_index,
            blend_time,
            blend_function,
            blend_exp,
            lock_outgoing,
        })
    }
}

impl PropertyTrait for ViewTargetBlendParamsProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        asset.write_f32::<LittleEndian>(self.blend_time.0)?;
        asset.write_u8(self.blend_function.into())?;
        asset.write_f32::<LittleEndian>(self.blend_exp.0)?;
        asset.write_i32::<LittleEndian>(match self.lock_outgoing {
            true => 1,
            false => 0,
        })?;
        Ok(size_of::<f32>() * 2 + size_of::<u8>() + size_of::<i32>())
    }
}
