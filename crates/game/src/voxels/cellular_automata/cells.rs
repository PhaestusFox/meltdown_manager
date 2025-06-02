use std::ops::{Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

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
                temperature: FixedNum::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]),
                charge: FixedNum::from_be_bytes([slice[4], slice[5], slice[6], slice[7]]),
                presure: FixedNum::from_be_bytes([slice[8], slice[9], slice[10], slice[11]]),
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
        let num = FixedNum::from_num(rhs);
        self.temperature *= num;
        self.presure *= num;
        self.charge *= num;
    }
}

impl<T: fixed::traits::ToFixed> Mul<T> for CellData {
    type Output = Self;
    fn mul(mut self, rhs: T) -> Self::Output {
        self *= rhs;
        self
    }
}
