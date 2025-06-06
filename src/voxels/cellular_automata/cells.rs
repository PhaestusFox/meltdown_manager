use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::voxels::blocks::Blocks;

use super::FixedNum;
use super::*;
use bevy::prelude::*;
use chunk_serde::BinSerializer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellData {
    block: Blocks,
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
        Ok(12)
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        Ok((
            CellData {
                block: Blocks::from_repr(slice[0]).unwrap_or(Blocks::Void),
                energy: FixedNum::from_be_bytes(slice[2..6].try_into().unwrap()),
                charge: FixedNum::from_be_bytes(slice[6..10].try_into().unwrap()),
                presure: FixedNum::from_be_bytes(slice[10..14].try_into().unwrap()),
                flags: CellFlags::from_bits_truncate(slice[1]),
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
            energy: AIR_AT_20C.0,
            presure: ATM_1,
            charge: STD_CHARGE,
            flags: AIR_AT_20C.1,
        }
    }
}

impl CellData {
    pub const fn at_k(block: Blocks, k: FixedNum) -> CellData {
        let at = get_e_at_k(block, k);
        CellData {
            block,
            energy: at.0,
            charge: FixedNum::ZERO,
            presure: ATM_1,
            flags: at.1,
        }
    }

    pub fn set_block(&mut self, block: Blocks) {
        let new = get_e_at_k(block, self.temperature());
        self.energy = new.0;
        self.flags = new.1;
        self.block = block;
    }

    pub fn get_block(&self) -> Blocks {
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

    pub fn lookup_g(&self, block: Blocks) -> FixedNum {
        self.block.meta().conductivity(block as u8)
    }
}

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CellFlags: u8 {
        const IS_LIQUID = 0b00000001;
        const IS_GAS = 0b00000010;
        const SINK = 0b00000100;
        const FLOAT = 0b00001000;
    }
}
impl CellData {
    pub const fn all(val: FixedNum) -> Self {
        CellData {
            block: Blocks::Void,
            energy: val,
            charge: val,
            presure: val,
            flags: CellFlags::all(),
        }
    }

    pub const THE_VOID: CellData = CellData {
        block: Blocks::Void,
        energy: FixedNum::ZERO,
        charge: FixedNum::ZERO,
        presure: FixedNum::ZERO,
        flags: CellFlags::IS_GAS,
    };

    pub const MIN: CellData = CellData {
        block: Blocks::Void,
        energy: FixedNum::MIN,
        charge: FixedNum::MIN,
        presure: FixedNum::MIN,
        flags: CellFlags::empty(),
    };

    pub const MAX: CellData = CellData {
        block: Blocks::Void,
        energy: FixedNum::MAX,
        charge: FixedNum::MAX,
        presure: FixedNum::MAX,
        flags: CellFlags::IS_GAS,
    };

    pub const ZERO: CellData = CellData {
        block: Blocks::Void,
        energy: FixedNum::ZERO,
        charge: FixedNum::ZERO,
        presure: FixedNum::ZERO,
        flags: CellFlags::empty(),
    };
}
// #[test]
// mod test {
//     use chunk_serde::CompressedChunkData;

//     use crate::{
//         utils::BlockIter,
//         voxels::{Chunk, blocks::Blocks, map::CHUNK_VOL},
//     };

//     #[test]
//     fn compress_automita() {
//         use super::cellular_automata::{CellData, FixedNum};
//         let mut comp = CompressedChunkData::Error(69);

//         macro_rules! test {
//             ($generator:expr) => {
//                 comp = $generator.compress();
//                 assert_eq!(Chunk::<CellData>::decompress(&comp), $generator);
//             };
//             ($generator:expr, $expect:expr) => {
//                 comp = $generator.compress();
//                 assert_eq!(comp, $expect);
//             };
//         }

//         let mut chunk = Chunk::<CellData>::empty();

//         // test back and forth
//         test!(chunk);
//         // test empty compress to Solid
//         test!(chunk, CompressedChunkData::Solid(CellData::default()));

//         let dummy = CellData {
//             block: Blocks::Void,
//             energy: FixedNum::from_num(69.42),
//             presure: FixedNum::from_num(420),
//             charge: FixedNum::from_num(4.2),
//             flags: Default::default(),
//         };
//         chunk.set_block(0, 0, 0, dummy);
//         // test changing 0,0,0 compress to RLE
//         test!(chunk);
//         test!(
//             chunk,
//             CompressedChunkData::RunLen(vec![
//                 (dummy, 1),
//                 (CellData::default(), (CHUNK_VOL - 1) as u16)
//             ])
//         );

//         // test worse case for RLE
//         let mut raw = vec![CellData::default(); CHUNK_VOL];
//         // set every other block copper
//         for (x, y, z) in BlockIter::<30, 30, 30>::new().step_by(2) {
//             raw[Chunk::<CellData>::index(x, y, z)] = dummy;
//             chunk.set_block(x, y, z, dummy);
//         }

//         test!(chunk);
//         test!(chunk, CompressedChunkData::Raw(raw));
//     }
//     #[test]
//     fn fuzz_cell_compression() {
//         use super::cellular_automata::{CellData, FixedNum};
//         let mut rng = rand::thread_rng();

//         let mut chunk = Chunk::<CellData>::empty();

//         let div = FixedNum::from_num(100.);
//         for _ in 0..27000 {
//             let x = rng.random_range(0..10000);
//             let y = rng.random_range(0..10000);
//             let z = rng.random_range(0..10000);
//             chunk.set_block(
//                 rng.random_range(0..30),
//                 rng.random_range(0..30),
//                 rng.random_range(0..30),
//                 CellData {
//                     block: Blocks::Void,
//                     energy: FixedNum::from_num(x) / div,
//                     charge: FixedNum::from_num(y) / div,
//                     presure: FixedNum::from_num(z) / div,
//                     flags: Default::default(),
//                 },
//             );
//             let comp = chunk.compress();
//             assert_eq!(Chunk::decompress(&comp), chunk);
//         }
//     }

//     #[test]
//     fn fuzz_cell_serde() {
//         use super::cellular_automata::{CellData, FixedNum};
//         let mut chunk = Chunk::empty();
//         let mut rng = rand::thread_rng();

//         use chunk_serde::Serialize;
//         let mut data = BinSerializer::new();
//         let mut out = CompressedChunkData::Error(69);
//         let mut comp = chunk.compress();
//         let mut len = 0;

//         macro_rules! test {
//             ($generator:expr) => {
//                 data.clear();
//                 comp = $generator.compress();
//                 data.insert(&comp);
//                 (out, len) = CompressedChunkData::extract(data.as_ref()).unwrap();
//                 assert_eq!(len, data.len());
//                 assert_eq!(out, comp);
//             };
//         }
//         let div = FixedNum::from_num(100.);
//         for _ in 0..27000 {
//             let x = rng.random_range(0..10000);
//             let y = rng.random_range(0..10000);
//             let z = rng.random_range(0..10000);
//             chunk.set_block(
//                 rng.random_range(0..30),
//                 rng.random_range(0..30),
//                 rng.random_range(0..30),
//                 CellData {
//                     block: Blocks::Void,
//                     energy: FixedNum::from_num(x) / div,
//                     charge: FixedNum::from_num(y) / div,
//                     presure: FixedNum::from_num(z) / div,
//                     flags: Default::default(),
//                 },
//             );
//             test!(chunk);
//         }
//     }
// }
