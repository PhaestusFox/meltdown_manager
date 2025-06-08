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
    pub charge: FixedNum,
    pub presure: FixedNum,
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
        vec.push(self.flags.bits());
        for byte in self.energy.to_be_bytes() {
            vec.push(byte);
        }
        for byte in self.charge.to_be_bytes() {
            vec.push(byte);
        }
        for byte in self.presure.to_be_bytes() {
            vec.push(byte);
        }
        Ok(14)
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        Ok((
            CellData {
                block: BlockType::from_repr(slice[0]).unwrap_or(BlockType::Void),
                energy: FixedNum::from_be_bytes(slice[2..6].try_into().unwrap()),
                charge: FixedNum::from_be_bytes(slice[6..10].try_into().unwrap()),
                presure: FixedNum::from_be_bytes(slice[10..14].try_into().unwrap()),
                flags: CellFlags::from_bits_truncate(slice[1]),
            },
            14,
        ))
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
            presure: ATM_1,
            charge: STD_CHARGE,
            flags: AIR_AT_20C.1,
        }
    }
}

impl CellData {
    pub const fn at_k(block: BlockType, k: FixedNum) -> CellData {
        let at = get_e_at_k(block, k);
        CellData {
            block,
            energy: at.0,
            charge: FixedNum::ZERO,
            presure: ATM_1,
            flags: at.1,
        }
    }

    pub fn set_block_type(&mut self, block: BlockType) {
        let new = get_e_at_k(block, self.temperature());
        self.energy = new.0;
        self.flags = new.1;
        self.block = block;
    }

    pub fn get_block_type(&self) -> BlockType {
        self.block
    }

    pub fn min(&mut self, other: &Self) {
        self.energy = self.energy.min(other.energy);
        self.charge = self.charge.min(other.charge);
        self.presure = self.presure.min(other.presure);
    }

    pub fn max(&mut self, other: &Self) {
        self.energy = self.energy.max(other.energy);
        self.charge = self.charge.max(other.charge);
        self.presure = self.presure.max(other.presure);
    }

    pub fn any_zero(&self) -> bool {
        self.energy.is_zero() | self.charge.is_zero() | self.presure.is_zero()
    }

    pub fn normalize(&self, range: FixedNum) -> CellData {
        let mut out = *self;
        out.energy /= range;
        out.energy = out.energy.clamp(FixedNum::ZERO, FixedNum::ONE);
        out.charge /= range;
        out.charge = out.charge.clamp(FixedNum::ZERO, FixedNum::ONE);
        out.presure /= range;
        out.presure = out.presure.clamp(FixedNum::ZERO, FixedNum::ONE);
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
        self.presure -= rhs.presure;
        self.charge -= rhs.charge;
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
        self.charge /= rhs.charge;
        self.presure /= rhs.presure;
    }
}

impl MulAssign for CellData {
    fn mul_assign(&mut self, rhs: Self) {
        self.energy *= rhs.energy;
        self.presure *= rhs.presure;
        self.charge *= rhs.charge;
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
        self.presure *= rhs;
        self.charge *= rhs;
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
        self.charge = self.charge.clamp(min, max);
        self.presure = self.presure.clamp(min, max);
    }
}

impl AddAssign for CellData {
    fn add_assign(&mut self, rhs: Self) {
        self.energy += rhs.energy;
        self.presure += rhs.presure;
        self.charge += rhs.charge;
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
        self.presure /= rhs;
        self.charge /= rhs;
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
    pub fn temperature(&self) -> FixedNum {
        let meta = self.block.meta();
        self.energy / meta.properties.specific_heat
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
            charge: val,
            presure: val,
            flags: CellFlags::all(),
        }
    }

    pub const THE_VOID: CellData = CellData {
        block: BlockType::Void,
        energy: FixedNum::ZERO,
        charge: FixedNum::ZERO,
        presure: FixedNum::ZERO,
        flags: CellFlags::IS_GAS,
    };

    pub const MIN: CellData = CellData {
        block: BlockType::Void,
        energy: FixedNum::MIN,
        charge: FixedNum::MIN,
        presure: FixedNum::MIN,
        flags: CellFlags::empty(),
    };

    pub const MAX: CellData = CellData {
        block: BlockType::Void,
        energy: FixedNum::MAX,
        charge: FixedNum::MAX,
        presure: FixedNum::MAX,
        flags: CellFlags::IS_GAS,
    };

    pub const ZERO: CellData = CellData {
        block: BlockType::Void,
        energy: FixedNum::ZERO,
        charge: FixedNum::ZERO,
        presure: FixedNum::ZERO,
        flags: CellFlags::empty(),
    };
}
