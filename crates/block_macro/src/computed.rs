use crate::THERMAL_CONDUCTIVITY;
use crate::{BlockProperties, FixedNum};

#[derive(Debug, Clone, Copy)]
pub struct BlockMeta {
    pub id: u8,
    /// Physical properties of the block
    pub properties: BlockProperties,
    /// The absolute energy level a block becomes a liquid
    /// J / Voxel
    /// Calculated: `melting_point * specific_heat + fusion_energy`
    pub liquid_energy: FixedNum,
    /// The absolute energy level a block becomes a gas
    /// J / Voxel
    /// Calculated: `boiling_point * specific_heat + fusion_energy + vaporization_energy`
    pub gas_energy: FixedNum,
}

impl BlockMeta {
    pub const VOID: BlockMeta = BlockMeta {
        id: 255,
        properties: BlockProperties::VOID,
        liquid_energy: FixedNum::ZERO,
        gas_energy: FixedNum::ZERO,
    };

    pub const fn properties(&self) -> &BlockProperties {
        &self.properties
    }

    pub const fn conductivity(&self, other: u8) -> FixedNum {
        let i = max(self.id, other) as usize;
        let j = min(self.id, other) as usize;
        let index = (i * (i + 1)) / 2 + j;
        if other == 255 {
            return FixedNum::ONE;
        }
        THERMAL_CONDUCTIVITY[index]
    }
}

const fn min(a: u8, b: u8) -> u8 {
    if a < b { a } else { b }
}

const fn max(a: u8, b: u8) -> u8 {
    if a > b { a } else { b }
}
