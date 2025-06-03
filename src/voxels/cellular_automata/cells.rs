use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use super::FixedNum;
use super::*;
use bevy::prelude::*;
use chunk_serde::BinSerializer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellData {
    pub temperature: FixedNum,
    pub charge: FixedNum,
    pub presure: FixedNum,
}

pub struct BlockProperties {
    pub heat: FixedNum,
    pub conductivity: FixedNum,
    pub density: FixedNum,
    pub melting_point: FixedNum,
}

impl BlockProperties {
    pub const VOID: BlockProperties = BlockProperties {
        heat: FixedNum::ZERO,
        conductivity: FixedNum::ZERO,
        density: FixedNum::ZERO,
        melting_point: FixedNum::ZERO,
    };

    pub const DEFAULT: BlockProperties = BlockProperties {
        heat: FixedNum::lit("0.0"),
        conductivity: FixedNum::lit("1.00"),
        density: FixedNum::lit("1.0"),
        melting_point: FixedNum::lit("500.0"),
    };

    pub const URANIUM: BlockProperties = BlockProperties {
        heat: FixedNum::lit("5.0"),
        conductivity: FixedNum::lit("1.00"),
        density: FixedNum::lit("19.1"),
        melting_point: FixedNum::lit("500.0"),
    };
}

impl chunk_serde::Serialize for CellData {
    fn insert(&self, vec: &mut BinSerializer) -> Result<usize> {
        for byte in self.temperature.to_be_bytes() {
            vec.push(byte);
        }
        for byte in self.charge.to_be_bytes() {
            vec.push(byte);
        }
        for byte in self.presure.to_be_bytes() {
            vec.push(byte);
        }
        Ok(12)
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        Ok((
            CellData {
                temperature: FixedNum::from_be_bytes(slice[0..4].try_into().unwrap()),
                charge: FixedNum::from_be_bytes(slice[4..8].try_into().unwrap()),
                presure: FixedNum::from_be_bytes(slice[8..12].try_into().unwrap()),
            },
            12,
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
            temperature: K_AT_20C,
            presure: ATM_1,
            charge: STD_CHARGE,
        }
    }
}

impl CellData {
    pub const THE_VOID: CellData = CellData {
        temperature: FixedNum::ZERO,
        charge: FixedNum::ZERO,
        presure: FixedNum::ZERO,
    };

    pub const MIN: CellData = CellData {
        temperature: FixedNum::MIN,
        charge: FixedNum::MIN,
        presure: FixedNum::MIN,
    };

    pub const MAX: CellData = CellData {
        temperature: FixedNum::MAX,
        charge: FixedNum::MAX,
        presure: FixedNum::MAX,
    };

    pub const ZERO: CellData = CellData {
        temperature: FixedNum::ZERO,
        charge: FixedNum::ZERO,
        presure: FixedNum::ZERO,
    };

    pub fn min(&mut self, other: &Self) {
        self.temperature = self.temperature.min(other.temperature);
        self.charge = self.charge.min(other.charge);
        self.presure = self.presure.min(other.presure);
    }

    pub fn max(&mut self, other: &Self) {
        self.temperature = self.temperature.max(other.temperature);
        self.charge = self.charge.max(other.charge);
        self.presure = self.presure.max(other.presure);
    }

    pub fn any_zero(&self) -> bool {
        self.temperature.is_zero() | self.charge.is_zero() | self.presure.is_zero()
    }

    pub fn normalize(&self, range: CellData) -> CellData {
        let mut out = *self;
        if range.temperature != FixedNum::ZERO {
            out.temperature /= range.temperature;
            out.temperature = out.temperature.clamp(FixedNum::ZERO, FixedNum::ONE);
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
        self.temperature -= rhs.temperature;
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
        self.temperature /= rhs.temperature;
        self.charge /= rhs.charge;
        self.presure /= rhs.presure;
    }
}

impl MulAssign for CellData {
    fn mul_assign(&mut self, rhs: Self) {
        self.temperature *= rhs.temperature;
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
        self.temperature *= rhs;
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
        self.temperature = self.temperature.clamp(min, max);
        self.charge = self.charge.clamp(min, max);
        self.presure = self.presure.clamp(min, max);
    }
}

impl AddAssign for CellData {
    fn add_assign(&mut self, rhs: Self) {
        self.temperature += rhs.temperature;
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
        self.temperature /= rhs;
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
