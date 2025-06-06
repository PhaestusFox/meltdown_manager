use ron::error::SpannedResult;

use crate::{FixedNum, ONEHUNDRED, ONETHOUSAND, TEN};

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub(super) struct RawBlockProperties {
    /// Mass of the Voxel in kg
    /// kg / Voxel
    pub density: i32,
    /// The Energy required to heat the Voxel by 1 Kelvin
    /// J / Voxel / Kelvin
    pub specific_heat: i32,
    /// The Energy Transfer per Kelvin per TimeStep
    /// J / Kelvin / Kelvin
    pub thermal_conductivity: i32,
    /// The Energy required to melt the Voxel
    /// J / Voxel
    pub fusion_energy: i32,
    /// The Temperature at which the Voxel melts
    /// Kelvin
    pub melting_point: i32,
    /// The Energy required to vaporize the Voxel
    /// J / Voxel
    pub vaporization_energy: i32,
    /// The Temperature at which the Voxel vaporizes
    /// Kelvin
    pub boiling_point: i32,
}

impl RawBlockProperties {
    pub const fn from_bytes(bytes: [u8; size_of::<RawBlockProperties>()]) -> Self {
        // should only be byte representation of the RawBlockProperties
        unsafe { std::mem::transmute(bytes) }
    }

    pub const fn to_bytes(self) -> [u8; size_of::<RawBlockProperties>()] {
        // should only be byte representation of the RawBlockProperties
        unsafe { std::mem::transmute(self) }
    }

    pub fn from_str(s: &str) -> SpannedResult<Self> {
        ron::from_str(s)
    }

    pub const VOID: Self = RawBlockProperties {
        density: 0,
        specific_heat: 1000,
        thermal_conductivity: 10,
        fusion_energy: 0,
        melting_point: 0,
        vaporization_energy: 0,
        boiling_point: 0,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct BlockProperties {
    /// Mass of the Voxel in kg
    /// kg / Voxel
    pub density: FixedNum,
    /// The Energy required to heat the Voxel by 1 Kelvin
    /// J / Voxel / Kelvin
    pub specific_heat: FixedNum,
    /// The Energy Transfer per Kelvin per TimeStep
    /// J / Kelvin / Kelvin
    pub thermal_conductivity: FixedNum,
    /// The Energy required to melt the Voxel
    /// J / Voxel
    pub fusion_energy: FixedNum,
    /// The Temperature at which the Voxel melts
    /// Kelvin
    pub melting_point: FixedNum,
    /// The Energy required to vaporize the Voxel
    /// J / Voxel
    /// None if energy is too high for simulation
    pub vaporization_energy: FixedNum,
    /// The Temperature at which the Voxel vaporizes
    /// Kelvin
    pub boiling_point: FixedNum,
}

impl BlockProperties {
    pub(crate) const fn from_raw(raw: RawBlockProperties) -> Self {
        let ve = if raw.vaporization_energy == 0 {
            FixedNum::MAX
        } else {
            FixedNum::const_from_int(raw.vaporization_energy)
        };

        BlockProperties {
            density: FixedNum::const_from_int(raw.density),
            specific_heat: FixedNum::const_from_int(raw.specific_heat),
            thermal_conductivity: FixedNum::const_from_int(raw.thermal_conductivity)
                .saturating_div(ONETHOUSAND),
            fusion_energy: FixedNum::const_from_int(raw.fusion_energy),
            melting_point: FixedNum::const_from_int(raw.melting_point).saturating_div(ONEHUNDRED),
            vaporization_energy: ve,
            boiling_point: FixedNum::const_from_int(raw.boiling_point).saturating_div(ONEHUNDRED),
        }
    }

    pub const VOID: BlockProperties = BlockProperties {
        density: FixedNum::ZERO,
        specific_heat: FixedNum::const_from_int(1000),
        thermal_conductivity: FixedNum::ONE,
        fusion_energy: FixedNum::ZERO,
        melting_point: FixedNum::ZERO,
        vaporization_energy: FixedNum::ZERO,
        boiling_point: FixedNum::ZERO,
    };
}
