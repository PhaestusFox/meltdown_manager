use bevy::prelude::*;
use chunk_serde::{BinSerializer, StrError};

const K_AT_20C: FixedNum = FixedNum::lit("293.15");
const ATM_1: FixedNum = FixedNum::lit("101.325");
const STD_CHARGE: FixedNum = FixedNum::lit("0");

pub type FixedNum = fixed::types::I22F10;
use super::{voxel_chunk::Chunk, *};

pub struct PrevioseChunk(Chunk<CellData>);

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
