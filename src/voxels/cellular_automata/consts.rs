pub type FixedNum = fixed::types::I25F7;

pub const AIR_AT_20C: (FixedNum, CellFlags) = get_e_at_k(Blocks::Air, FixedNum::lit("293.15"));
pub const ATM_1: FixedNum = FixedNum::lit("101.325");
pub const STD_CHARGE: FixedNum = FixedNum::lit("0");

pub const fn get_e_at_k(block: Blocks, k: FixedNum) -> (FixedNum, CellFlags) {
    let props = block.properties();
    let f = if k.to_bits() > props.boiling_point.to_bits() {
        CellFlags::IS_GAS
    } else if k.to_bits() > props.melting_point.to_bits() {
        CellFlags::IS_LIQUID
    } else {
        CellFlags::empty()
    };
    (k.saturating_mul(props.specific_heat), f)
}

use fixed::traits::Fixed;
use strum::IntoEnumIterator;

use crate::voxels::blocks::Blocks;
use crate::voxels::cellular_automata::cells::CellFlags;

use super::CellData;

// impl BlockProperties {
//     pub const VOID: BlockProperties = BlockProperties {
//         heat: FixedNum::ZERO,
//         conductivity: FixedNum::ZERO,
//         mass: FixedNum::ZERO,
//         melting_point: FixedNum::ZERO,
//         boiling_point: FixedNum::ZERO,
//         heat_capacity: FixedNum::lit("1000.0"),
//         fusion_energy: FixedNum::ZERO,
//         vaporization_energy: FixedNum::ZERO,
//         thermal_conductivity: FixedNum::lit("100.0"),
//     };

//     pub const DEFAULT: BlockProperties = BlockProperties {
//         heat: FixedNum::ZERO,
//         conductivity: FixedNum::lit("1.0"),
//         mass: FixedNum::lit("1.0"),
//         melting_point: FixedNum::lit("1000.0"),
//         boiling_point: FixedNum::lit("2000.0"),
//         heat_capacity: FixedNum::lit("1000.0"),
//         thermal_conductivity: FixedNum::lit("50.0"),
//         fusion_energy: FixedNum::lit("1000.0"),
//         vaporization_energy: FixedNum::lit("1000.0"),
//     };

//     pub const URANIUM: BlockProperties = BlockProperties {
//         heat: FixedNum::lit("10000.0"), // todo! drop to 10
//         melting_point: FixedNum::lit("1405.3"),

//         // Density: Very high, 19.1 g/cm3 --- 19100 kg/m3 --- 19.1 ton
//         mass: FixedNum::lit("19.1"),
//         // Molar heat capacity	27.665 J/(mol·K) --- 116.24 J/(kg.K) --- 2220.184 J/K
//         heat_capacity: FixedNum::lit("2220.184"),
//         // Thermal conductivity 27.5 W/(m⋅K) --- 27.5 W/(m.K) --- 27.5 W/K
//         thermal_conductivity: FixedNum::lit("27.5"),
//         boiling_point: FixedNum::lit("4404.0"),

//         fusion_energy: FixedNum::lit("38.403"), // 11.5 J/g / 1000 to fit
//         vaporization_energy: FixedNum::lit("1752.521"), // 400 J/g / 1000 to fit
//         ..BlockProperties::DEFAULT
//     };

//     pub const COPPER: BlockProperties = BlockProperties {
//         heat: FixedNum::lit("0.0"),
//         // Density: 8.935 g/cm3 --- 8935 kg/m3 --- 8.935 u/vox
//         mass: FixedNum::lit("8.935"),
//         // Molar heat capacity	24.440 J/(mol·K) --- 384.603 J/(kg.K) --- 3436.430 J/K
//         heat_capacity: FixedNum::lit("3436.430"),
//         // Thermal conductivity 401 W/(m⋅K) --- 401 W/(m.K) --- 401 W/K
//         thermal_conductivity: FixedNum::lit("401"),

//         melting_point: FixedNum::lit("1357.77"),
//         boiling_point: FixedNum::lit("2835.0"),

//         fusion_energy: FixedNum::lit("208.667"), // 11.5 J/g / 1000 to fit
//         vaporization_energy: FixedNum::lit("4727.284"), // 400 J/g / 1000 to fit

//         ..BlockProperties::DEFAULT
//     };

//     pub const WATER: BlockProperties = BlockProperties {
//         heat: FixedNum::lit("0.0"),
//         // Density: 1 g/cm3 --- 1000 kg/m3 --- 1 u/vox
//         mass: FixedNum::lit("1.0"),
//         // Molar heat capacity	75.3 J/(mol·K) --- 4184 J/(kg.K) --- 4184 J/K
//         heat_capacity: FixedNum::lit("4184"),
//         // Thermal conductivity 0.606 W/(m⋅K) --- 0.606 W/(m.K) --- 0.606 W/K
//         thermal_conductivity: FixedNum::lit("6.606"),
//         melting_point: FixedNum::lit("273.15"),
//         boiling_point: FixedNum::lit("373.15"),

//         fusion_energy: FixedNum::lit("333000"), // 333 J/g / 1000 to fit
//         vaporization_energy: FixedNum::lit("2257000"), // 2257 J/g / 1000 to fit

//         ..BlockProperties::DEFAULT
//     };

//     pub const AIR: BlockProperties = BlockProperties {
//         heat: FixedNum::ZERO,
//         conductivity: FixedNum::lit("0.0257"),
//         mass: FixedNum::lit("0.001225"),
//         melting_point: FixedNum::ZERO,
//         boiling_point: FixedNum::lit("194.65"),
//         heat_capacity: FixedNum::lit("1005.0"),
//         thermal_conductivity: FixedNum::lit("0.0257"),
//         fusion_energy: FixedNum::ZERO,
//         vaporization_energy: FixedNum::ZERO,
//     };
// }
// {
//     type: Water,
//     Molar Mass: 18
//     Melting Point: 273.15 K ​(0.0 °C,  °F)
//     Electrical Conductivity:
//     Heat of fusion 333 J/g --- 333000000J
//     Heat of vaporization 2257 J/g --- 2257000000J
//     Boiling point 373.13 K ​(100 °C,  °F)
//     Density: 1 g/cm3 --- 1000 kg/m3 --- 1 u/vox
//     Molar heat capacity	75.385 J/(mol·K) ---  4188 J/(kg.K) --- 4188 J/K
//     Thermal conductivity 0.6065 W/(m·K) ---                   --- 0.6065 W/K
// }

#[test]
fn blocks_can_evaperation_energy() {
    for block in Blocks::iter() {
        println!("Block: {:?}", block);
        let props = block.properties();
        let fusion_energy = props.fusion_energy;
        let vaporization_energy = props.vaporization_energy;
        let energy_to_melt = props.melting_point * props.specific_heat + fusion_energy;
        println!("melt energy: {:?}", energy_to_melt);
        let energy_to_evaporate =
            props.boiling_point * props.specific_heat + vaporization_energy + fusion_energy;
        println!("evaporate energy: {:?}", energy_to_evaporate);
    }
}
