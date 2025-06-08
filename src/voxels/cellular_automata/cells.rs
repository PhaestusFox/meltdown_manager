use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::voxels::block::BlockType;

use super::FixedNum;
use super::*;
use bevy::prelude::*;
use block_meta::BlockProperties;
use chunk_serde::BinSerializer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellData {
    block: BlockType,
    pub energy: FixedNum,
    tempreture: FixedNum,
    density: FixedNum,
    pub flags: CellFlags,
}

// pub struct BlockProperties {
//     pub heat: FixedNum,
//     pub conductivity: FixedNum,
//     pub mass: FixedNum,

//     // For heat
//     /// units of energy per degree Kelvin
//     pub heat_capacity: FixedNum,
//     /// units of energy per degree Kelvin per TimeStep
//     pub thermal_conductivity: FixedNum,

//     pub melting_point: FixedNum,
//     pub fusion_energy: FixedNum,
//     pub boiling_point: FixedNum,
//     pub vaporization_energy: FixedNum,
// }

impl chunk_serde::Serialize for CellData {
    fn insert(&self, vec: &mut BinSerializer) -> Result<usize> {
        vec.push(self.block as u8);
        for byte in self.energy.to_be_bytes() {
            vec.push(byte);
        }
        Ok(5)
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        let mut out = CellData {
            block: BlockType::from_repr(slice[0]).unwrap_or(BlockType::Void),
            energy: FixedNum::from_be_bytes(slice[1..5].try_into().unwrap()),
            tempreture: FixedNum::ONE, // Will be set later
            density: FixedNum::ONE,    // Will be set later
            flags: CellFlags::empty(),
        };
        out.set_tempreture();
        out.set_phase();
        out.set_density();
        Ok((out, 5))
    }

    // fn insert_str(&self, serializer: &mut chunk_serde::StrSerializer) -> Result<usize> {
    //     let len = serializer.len();
    //     serializer.write(format_args!(
    //         "(temperature: {}, charge: {}, presure: {})",
    //         self.temperature, self.charge, self.presure
    //     ));
    //     Ok(serializer.len() - len)
    // }

    // fn extract_str(str: &str) -> Result<(Self, usize)> {
    //     let mut used = 0;
    //     let mut skip = 0;
    //     let mut temp = None;
    //     let mut charge = None;
    //     let mut presure = None;
    //     for char in str.chars() {
    //         used += 1;
    //         if skip > 0 {
    //             skip -= 1;
    //             continue;
    //         }
    //         if char.is_whitespace() {
    //             continue;
    //         }
    //         if char == '(' {
    //             let word = str[used..]
    //                 .split(':')
    //                 .next()
    //                 .ok_or(StrError::ExpectChar(':'))?;
    //             used += word.len() + 1;
    //             let (res, len) = FixedNum::e
    //             match word {
    //                 "temperature" => {
    //                     used += 11;
    //                 }
    //             }
    //         }
    //     }
    // }
}

impl Default for CellData {
    fn default() -> Self {
        CellData {
            block: BlockType::Air,
            energy: AIR_AT_20C.0,
            tempreture: FixedNum::lit("293.15"), // 20C in Kelvin
            density: FixedNum::lit("1.0"),       // Default density
            flags: AIR_AT_20C.1,
        }
    }
}

impl CellData {
    pub const fn at_k(block: BlockType, k: FixedNum) -> CellData {
        let at = get_e_at_k(block, k);
        let d = if at.1.contains(CellFlags::IS_GAS) {
            FixedNum::lit("0.33")
        } else if at.1.contains(CellFlags::IS_LIQUID) {
            FixedNum::lit("0.90")
        } else {
            FixedNum::ONE
        };
        CellData {
            block,
            energy: at.0,
            density: block.properties().density.saturating_mul(d),
            tempreture: k,
            flags: at.1,
        }
    }

    pub fn set_block_type(&mut self, block: BlockType) {
        let new = get_e_at_k(block, self.temperature());
        self.energy = new.0;
        self.flags = new.1;
        self.block = block;
        self.set_tempreture();
        self.set_phase();
        self.set_density();
    }

    pub fn get_block_type(&self) -> BlockType {
        self.block
    }

    pub fn min(&mut self, other: &Self) {
        self.energy = self.energy.min(other.energy);
    }

    pub fn max(&mut self, other: &Self) {
        self.energy = self.energy.max(other.energy);
    }

    pub fn any_zero(&self) -> bool {
        self.energy.is_zero()
    }

    pub fn normalize(&self, range: FixedNum) -> CellData {
        let mut out = *self;
        out.energy /= range;
        out.energy = out.energy.clamp(FixedNum::ZERO, FixedNum::ONE);
        out
    }
}

impl Sub for CellData {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign for CellData {
    fn sub_assign(&mut self, rhs: Self) {
        self.energy -= rhs.energy;
    }
}

impl Div for CellData {
    type Output = Self;
    fn div(mut self, rhs: Self) -> Self::Output {
        self /= rhs;
        self
    }
}

impl DivAssign for CellData {
    fn div_assign(&mut self, rhs: Self) {
        self.energy /= rhs.energy;
    }
}

impl MulAssign for CellData {
    fn mul_assign(&mut self, rhs: Self) {
        self.energy *= rhs.energy;
    }
}

impl Mul for CellData {
    type Output = Self;
    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<T: fixed::traits::ToFixed> MulAssign<T> for CellData {
    fn mul_assign(&mut self, rhs: T) {
        let rhs = FixedNum::from_num(rhs);
        self.energy *= rhs;
    }
}

impl<T: fixed::traits::ToFixed> Mul<T> for CellData {
    type Output = Self;
    fn mul(mut self, rhs: T) -> Self::Output {
        self *= rhs;
        self
    }
}

impl CellData {
    pub fn clamp(&mut self, min: FixedNum, max: FixedNum) {
        self.energy = self.energy.clamp(min, max);
    }
}

impl AddAssign for CellData {
    fn add_assign(&mut self, rhs: Self) {
        self.energy += rhs.energy;
    }
}

impl Add for CellData {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<T: fixed::traits::ToFixed> DivAssign<T> for CellData {
    fn div_assign(&mut self, rhs: T) {
        let rhs = FixedNum::from_num(rhs);
        self.energy /= rhs;
    }
}

impl<T: fixed::traits::ToFixed> Div<T> for CellData {
    type Output = Self;
    fn div(mut self, rhs: T) -> Self::Output {
        self /= rhs;
        self
    }
}

impl CellData {
    pub const fn temperature(&self) -> FixedNum {
        self.tempreture
    }

    pub fn set_tempreture(&mut self) {
        let meta = self.block.meta();
        self.tempreture = self.energy / meta.properties.specific_heat;
        if self.temperature() <= FixedNum::ZERO {
            self.energy = FixedNum::ONE;
            self.tempreture = FixedNum::lit("0.15");
        }
    }

    pub fn set_phase(&mut self) {
        if self.temperature() > self.block.properties().boiling_point {
            self.flags.set(CellFlags::IS_GAS, true);
            self.flags.set(CellFlags::IS_LIQUID, false);
        } else if self.temperature() > self.block.properties().melting_point {
            self.flags.set(CellFlags::IS_LIQUID, true);
            self.flags.set(CellFlags::IS_GAS, false);
        } else {
            self.flags
                .set(CellFlags::IS_GAS | CellFlags::IS_LIQUID, false);
        }
    }

    pub const fn lookup_g(&self, block: BlockType) -> FixedNum {
        self.block.meta().conductivity(block as u8)
    }

    pub const fn properties(&self) -> &'static BlockProperties {
        self.block.properties()
    }

    #[inline(always)]
    pub const fn is_liquid(&self) -> bool {
        self.flags.contains(CellFlags::IS_LIQUID)
    }

    #[inline(always)]
    pub const fn is_gas(&self) -> bool {
        self.flags.contains(CellFlags::IS_GAS)
    }

    #[inline(always)]
    pub const fn can_move(&self) -> bool {
        self.flags.intersects(CellFlags::CAN_MOVE)
    }
}

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CellFlags: u8 {
        const IS_LIQUID = 1 << 0;
        const IS_GAS = 1 << 1;
        const MOVE_UP = 1 << 2;
        const MOVE_DOWN = 2 << 2;
        const MOVE_LEFT = 3 << 2;
        const MOVE_RIGHT = 4 << 2;
        const MOVE_FORWARD = 5 << 2;
        const MOVE_BACK = 6 << 2;
        const MOVE_ALL = 7 << 2;
        const CAN_MOVE = 3;
    }
}
impl CellData {
    pub const fn all(val: FixedNum) -> Self {
        CellData {
            block: BlockType::Void,
            energy: val,
            ..CellData::THE_VOID
        }
    }

    pub const THE_VOID: CellData = CellData {
        block: BlockType::Void,
        energy: FixedNum::ONE,
        flags: CellFlags::IS_GAS,
        density: FixedNum::ONE,
        tempreture: FixedNum::lit("271.15"),
    };

    pub const MIN: CellData = CellData {
        block: BlockType::Void,
        energy: FixedNum::MIN,
        flags: CellFlags::empty(),
        ..CellData::THE_VOID
    };

    pub const MAX: CellData = CellData {
        block: BlockType::Void,
        energy: FixedNum::MAX,
        flags: CellFlags::IS_GAS,
        ..CellData::THE_VOID
    };

    pub const ZERO: CellData = CellData {
        block: BlockType::Void,
        energy: FixedNum::ZERO,
        flags: CellFlags::empty(),
        ..CellData::THE_VOID
    };
}

impl CellData {
    pub const fn density(&self) -> FixedNum {
        self.density
    }

    pub fn set_density(&mut self) {
        if self.is_gas() {
            self.density = FixedNum::lit("1.");
        } else if self.is_liquid() {
            let factor = self
                .properties()
                .melting_point
                .saturating_div(self.temperature());
            self.density = self
                .properties()
                .density
                .saturating_mul(FixedNum::lit("0.33"))
                .saturating_mul(factor);
        } else {
            self.density = self.properties().density;
        }
    }
}
