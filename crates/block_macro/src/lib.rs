#![feature(slice_as_chunks)]
#![allow(dead_code)]
use std::fs;

pub use crate::properties::BlockProperties;
use crate::{computed::BlockMeta, properties::RawBlockProperties};

pub type FixedNum = fixed::types::I25F7;

pub fn make_block_meta_file<T: Iterator<Item: AsRef<str>>>(iter: T) {
    use std::io::Write;
    use std::path::Path;

    let path = Path::new("assets/blocks/raw.meta");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .expect("can create file");
    // println!(
    //     "Writing block meta from frmat {:#?}",
    //     ron::to_string_prety(&RawBlockProperties::VOID, ron::ser::PrettyConfig::default()).unwrap()
    // );

    for block in iter {
        let path = format!("assets/blocks/{}.block", block.as_ref());
        match fs::read_to_string(path) {
            Ok(b) => match properties::RawBlockProperties::from_str(b.as_str()) {
                Ok(raw) => {
                    assert_eq!(
                        file.write(&raw.to_bytes())
                            .expect("Failed to write block meta"),
                        RAW_SIZE
                    );
                }
                Err(e) => {
                    eprintln!("Failed to parse block meta for {}: {}", block.as_ref(), e);
                    assert_eq!(
                        file.write(&RawBlockProperties::VOID.to_bytes())
                            .expect("Failed to write block meta"),
                        RAW_SIZE
                    );
                }
            },
            Err(e) => {
                eprintln!("Failed to read block meta for {}: {}", block.as_ref(), e);
                assert_eq!(
                    file.write(&RawBlockProperties::VOID.to_bytes())
                        .expect("Failed to write block meta"),
                    RAW_SIZE
                );
            }
        };
    }
}

const BLOCK_META: BlockMetaArray = load_block_meta();

type BlockMetaArray = [BlockMeta; META_LEN];

const META_LEN: usize = RAW_DATA_LEN / size_of::<properties::RawBlockProperties>();

const RAW_DATA_LEN: usize = include_bytes!("../../../assets/blocks/raw.meta").len();

const THERMAL_CONDUCTIVITY: [FixedNum; (META_LEN * (META_LEN + 1)) / 2] =
    generate_thermal_conductivity();

const RAW_SIZE: usize = size_of::<properties::RawBlockProperties>();

pub const fn block_properties(block: u8) -> &'static BlockProperties {
    &BLOCK_META[block as usize].properties
}

pub const fn block_meta(block: u8) -> &'static BlockMeta {
    &BLOCK_META[block as usize]
}

const fn load_block_meta() -> BlockMetaArray {
    let (data, _) = include_bytes!("../../../assets/blocks/raw.meta")
        .as_chunks::<{ size_of::<properties::RawBlockProperties>() }>();
    let mut meta = [BlockMeta::VOID; META_LEN];
    let mut i = 0;
    while i < META_LEN {
        let raw = properties::RawBlockProperties::from_bytes(data[i]);
        let le = (raw.melting_point * raw.specific_heat) / 100 + raw.fusion_energy;
        let ge = if raw.fusion_energy == 0 {
            ((raw.boiling_point - raw.melting_point) * raw.specific_heat) / 100 + raw.fusion_energy
        } else {
            i32::MAX >> 7 // Max gas energy if fusion energy is 0
        };

        let properties = properties::BlockProperties::from_raw(raw);
        meta[i] = BlockMeta {
            id: i as u8,
            properties,
            liquid_energy: FixedNum::const_from_int(le), // Placeholder, should be calculated
            gas_energy: FixedNum::const_from_int(ge),    // Placeholder, should be calculated
        };
        i += 1;
    }
    meta
}

const fn generate_thermal_conductivity() -> [FixedNum; (META_LEN * (META_LEN + 1)) / 2] {
    let mut conductivity = [FixedNum::ZERO; (META_LEN * (META_LEN + 1)) / 2];
    let mut i = 0;
    while i < META_LEN {
        let mut j = i;
        while j < META_LEN {
            // make everything 7.5 million times larger to increase precision
            let r1 = FixedNum::const_from_int(7500000)
                .saturating_div(block_properties(i as u8).thermal_conductivity);
            let r2 = FixedNum::const_from_int(7500000)
                .saturating_div(block_properties(j as u8).thermal_conductivity);
            let divisor = r1.saturating_add(r2);
            let hm = FixedNum::const_from_int(15000000).saturating_div(divisor);
            conductivity[((j * (j + 1)) / 2) + i] = hm;
            j += 1;
        }
        i += 1;
    }
    conductivity
}

const ONETHOUSAND: FixedNum = FixedNum::const_from_int(1000);
const ONEHUNDRED: FixedNum = FixedNum::const_from_int(100);
const TEN: FixedNum = FixedNum::const_from_int(10);
const TWO: FixedNum = FixedNum::const_from_int(10);

pub mod computed;
pub mod properties;
