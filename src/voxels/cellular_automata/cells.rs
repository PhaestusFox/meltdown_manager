use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::voxels::blocks::Blocks;

use super::FixedNum;
use super::*;
use bevy::prelude::*;
use chunk_serde::BinSerializer;
const TWO: FixedNum = FixedNum::lit("2.0");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellData {
    pub block: Blocks,
    pub energy: FixedNum,
    pub charge: FixedNum,
    pub presure: FixedNum,
    pub flags: CellFlags,
}

pub struct BlockProperties {
    pub heat: FixedNum,
    pub conductivity: FixedNum,
    pub mass: FixedNum,

    // For heat
    /// units of energy per degree Kelvin
    pub heat_capacity: FixedNum,
    /// units of energy per degree Kelvin per TimeStep
    pub thermal_conductivity: FixedNum,

    pub melting_point: FixedNum,
    pub fusion_energy: FixedNum,
    pub boiling_point: FixedNum,
    pub vaporization_energy: FixedNum,
}

impl chunk_serde::Serialize for CellData {
    fn insert(&self, vec: &mut BinSerializer) -> Result<usize> {
        for byte in self.energy.to_be_bytes() {
            vec.push(byte);
        }
        for byte in self.charge.to_be_bytes() {
            vec.push(byte);
        }
        for byte in self.presure.to_be_bytes() {
            vec.push(byte);
        }
        vec.push(self.block as u8);
        vec.push(self.flags.bits());
        Ok(12)
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        Ok((
            CellData {
                block: Blocks::from_repr(slice[13]).unwrap_or(Blocks::Void),
                energy: FixedNum::from_be_bytes(slice[0..4].try_into().unwrap()),
                charge: FixedNum::from_be_bytes(slice[4..8].try_into().unwrap()),
                presure: FixedNum::from_be_bytes(slice[8..12].try_into().unwrap()),
                flags: CellFlags::from_bits_truncate(slice[14]),
            },
            13,
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
            block: Blocks::Air,
            energy: K_AT_20C,
            presure: ATM_1,
            charge: STD_CHARGE,
            flags: CellFlags::empty(),
        }
    }
}

impl CellData {
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

    pub fn normalize(&self, range: CellData) -> CellData {
        let mut out = *self;
        if range.energy != FixedNum::ZERO {
            out.energy /= range.energy;
            out.energy = out.energy.clamp(FixedNum::ZERO, FixedNum::ONE);
        }
        if range.charge != FixedNum::ZERO {
            out.charge /= range.charge;
            out.charge = out.charge.clamp(FixedNum::ZERO, FixedNum::ONE);
        }
        if range.presure != FixedNum::ZERO {
            out.presure /= range.presure;
            out.presure = out.presure.clamp(FixedNum::ZERO, FixedNum::ONE);
        }
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
        let melting_loss = self.block.block_properties().fusion_energy;
        let boiling_loss = self.block.block_properties().vaporization_energy;
        if self.flags.contains(CellFlags::IS_GAS) {
            (self.energy - melting_loss - boiling_loss)
                / self.block.block_properties().heat_capacity
        } else if self.flags.contains(CellFlags::IS_LIQUID) {
            (self.energy - melting_loss) / self.block.block_properties().heat_capacity
        } else {
            self.energy / self.block.block_properties().heat_capacity
        }
    }

    pub fn set_phase(&mut self) {
        let melting_loss = self.block.block_properties().fusion_energy;
        let boiling_loss = self.block.block_properties().vaporization_energy;

        let temp_after_melting =
            (self.energy - melting_loss) / self.block.block_properties().heat_capacity;
        let temp_after_boiling = (self.energy - boiling_loss - melting_loss)
            / self.block.block_properties().heat_capacity;

        if temp_after_boiling > self.block.block_properties().boiling_point {
            self.flags.insert(CellFlags::IS_GAS);
            self.flags.remove(CellFlags::IS_LIQUID);
        } else if temp_after_melting > self.block.block_properties().melting_point {
            self.flags.insert(CellFlags::IS_LIQUID);
            self.flags.remove(CellFlags::IS_GAS);
        } else {
            self.flags.remove(CellFlags::IS_LIQUID | CellFlags::IS_GAS);
        }
    }

    pub fn lookup_g(&self, block: Blocks) -> FixedNum {
        if block == self.block {
            return self.block.block_properties().thermal_conductivity;
        }
        // turn this into a lookup table
        TWO / (FixedNum::ONE / self.block.block_properties().thermal_conductivity
            + FixedNum::ONE / block.block_properties().thermal_conductivity)
    }
}

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CellFlags: u8 {
        const IS_LIQUID = 0b00000001;
        const IS_GAS = 0b00000010;
    }
}
