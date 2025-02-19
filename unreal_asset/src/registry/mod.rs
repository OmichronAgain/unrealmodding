use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    io::{Cursor, SeekFrom},
};

use byteorder::LittleEndian;

use crate::{
    crc,
    custom_version::FAssetRegistryVersionType,
    error::{Error, RegistryError},
    reader::{
        asset_reader::AssetReader, asset_trait::AssetTrait, asset_writer::AssetWriter,
        raw_writer::RawWriter,
    },
    registry::name_table_reader::NameTableReader,
    ue4version::VER_UE4_NAME_HASHES_SERIALIZED,
};

use self::{
    name_table_writer::NameTableWriter,
    objects::{
        asset_data::AssetData, asset_package_data::AssetPackageData, depends_node::DependsNode,
    },
};

pub(crate) mod name_table_reader;
pub(crate) mod name_table_writer;
pub mod objects;

#[derive(Debug)]
pub struct AssetRegistryState {
    pub assets_data: Vec<AssetData>,
    pub depends_nodes: Vec<DependsNode>,
    pub package_data: Vec<AssetPackageData>,

    name_map: Option<Vec<String>>,
    engine_version: i32,
    name_map_lookup: Option<HashMap<u64, i32>>,
    version: FAssetRegistryVersionType,
}

impl AssetRegistryState {
    fn load<Reader: AssetReader>(
        asset: &mut Reader,
        version: FAssetRegistryVersionType,
        assets_data: &mut Vec<AssetData>,
        depends_nodes: &mut Vec<DependsNode>,
        package_data: &mut Vec<AssetPackageData>,
    ) -> Result<(), Error> {
        *assets_data = asset.read_array(|asset: &mut Reader| AssetData::new(asset, version))?;

        if version < FAssetRegistryVersionType::AddedDependencyFlags {
            let local_num_depends_nodes = asset.read_i32::<LittleEndian>()?;
            *depends_nodes = Vec::with_capacity(local_num_depends_nodes as usize);

            for i in 0..local_num_depends_nodes {
                depends_nodes.push(DependsNode::new(i, version));
            }
            let depends_nodes_copy = depends_nodes.clone();

            if local_num_depends_nodes > 0 {
                for depends_node in depends_nodes {
                    depends_node.load_dependencies_before_flags(asset, &depends_nodes_copy)?;
                }
            }
        } else {
            let dependency_section_size = asset.read_i64::<LittleEndian>()?;
            let dependency_section_end = asset.position() + dependency_section_size as u64;
            let local_num_depends_nodes = asset.read_i32::<LittleEndian>()?;

            *depends_nodes = Vec::with_capacity(local_num_depends_nodes as usize);
            for i in 0..local_num_depends_nodes {
                depends_nodes.push(DependsNode::new(i, version));
            }

            let assets_data_copy = depends_nodes.clone();
            if local_num_depends_nodes > 0 {
                for depends_node in depends_nodes {
                    depends_node.load_dependencies(asset, &assets_data_copy)?;
                }
            }

            asset.set_position(dependency_section_end as u64);
        }

        *package_data =
            asset.read_array(|asset: &mut Reader| AssetPackageData::new(asset, version))?;

        Ok(())
    }

    fn write_data<Writer: AssetWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
        writer.write_i32::<LittleEndian>(self.assets_data.len() as i32)?;
        for asset_data in &self.assets_data {
            asset_data.write(writer)?;
        }

        if self.version < FAssetRegistryVersionType::AddedDependencyFlags {
            writer.write_i32::<LittleEndian>(self.depends_nodes.len() as i32)?;

            for depends_node in &self.depends_nodes {
                depends_node.save_dependencies_before_flags(writer)?;
            }
        } else {
            let pos = writer.position();
            writer.write_i64::<LittleEndian>(0)?;
            writer.write_i32::<LittleEndian>(self.depends_nodes.len() as i32)?;

            for depends_node in &self.depends_nodes {
                depends_node.save_dependencies(writer)?;
            }

            let end_pos = writer.position();
            writer.set_position(pos);
            writer.write_i64::<LittleEndian>(end_pos as i64 - pos as i64)?;
            writer.set_position(end_pos);
        }

        writer.write_i32::<LittleEndian>(self.package_data.len() as i32)?;
        for package_data in &self.package_data {
            package_data.write(writer)?;
        }

        Ok(())
    }

    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let version = FAssetRegistryVersionType::new(asset)?;
        let mut assets_data = Vec::new();
        let mut depends_nodes = Vec::new();
        let mut package_data = Vec::new();

        let mut name_map = None;
        let mut name_map_lookup = None;

        if version < FAssetRegistryVersionType::RemovedMD5Hash {
            return Err(Error::invalid_file(format!(
                "Cannot read registry state before {}",
                version as i32
            )));
        } else if version < FAssetRegistryVersionType::FixedTags {
            // name table reader
            let mut name_table_reader = NameTableReader::new(asset)?;
            name_map = Some(name_table_reader.name_map.clone()); // todo: something else instead of cloning?
            name_map_lookup = Some(name_table_reader.name_map_lookup.clone());
            Self::load(
                &mut name_table_reader,
                version,
                &mut assets_data,
                &mut depends_nodes,
                &mut package_data,
            )?;
        } else {
            Self::load(
                asset,
                version,
                &mut assets_data,
                &mut depends_nodes,
                &mut package_data,
            )?;
        }

        Ok(Self {
            version,
            assets_data,
            depends_nodes,
            package_data,

            name_map,
            name_map_lookup,

            engine_version: asset.get_engine_version(),
        })
    }

    pub fn write(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        let mut writer = RawWriter::new(cursor, self.engine_version);
        self.version.write(&mut writer)?;

        if self.version < FAssetRegistryVersionType::RemovedMD5Hash {
            return Err(Error::invalid_file(format!(
                "Cannot write registry state before {}",
                self.version as i32
            )));
        } else if self.version < FAssetRegistryVersionType::FixedTags {
            let pos = writer.position();
            writer.write_i64::<LittleEndian>(0)?;

            let name_map_lookup = self.name_map_lookup.as_ref().ok_or_else(|| {
                RegistryError::version("Name map lookup".to_string(), self.version)
            })?;

            let name_map = self
                .name_map
                .as_ref()
                .ok_or_else(|| RegistryError::version("Name map".to_string(), self.version))?;

            let mut name_table_writer = NameTableWriter::new(&mut writer, name_map_lookup);

            self.write_data(&mut name_table_writer)?;

            let offset = writer.position();
            writer.write_i32::<LittleEndian>(name_map.len() as i32)?;
            for name in name_map {
                writer.write_string(&Some(name.clone()))?; // todo: no cloning

                match writer.get_engine_version() >= VER_UE4_NAME_HASHES_SERIALIZED {
                    true => {
                        let hash = crc::generate_hash(name);
                        writer.write_u32::<LittleEndian>(hash)?;
                    }
                    false => {}
                }
            }

            let end = writer.position();

            writer.seek(SeekFrom::Start(pos))?;
            writer.write_i64::<LittleEndian>(offset as i64)?;
            writer.seek(SeekFrom::Start(end))?;
        } else {
            self.write_data(&mut writer)?;
        }

        Ok(())
    }

    pub fn add_name_reference(&mut self, string: &str, add_duplicates: bool) -> i32 {
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);

        let hash = hasher.finish();

        if let Some(lookup) = self.name_map_lookup.as_mut() {
            if !add_duplicates {
                if let Some(index) = lookup.get(&hash) {
                    return *index;
                }
            }

            let name_map = self.name_map.as_mut().expect("Corrupted memory");
            name_map.push(string.to_string());
            lookup.insert(hash, name_map.len() as i32 - 1);

            return name_map.len() as i32 - 1;
        }

        0
    }

    pub fn add_fname(&mut self, string: &str) -> i32 {
        self.add_name_reference(string, false)
    }

    pub fn get_version(&self) -> FAssetRegistryVersionType {
        self.version
    }
}
