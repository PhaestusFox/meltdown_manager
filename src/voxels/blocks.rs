use bevy::prelude::*;
use chunk_serde::BinSerializer;
use phoxels::core::BlockId;

use super::cellular_automata::BlockProperties;

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    strum_macros::EnumIter,
    strum_macros::FromRepr,
    Default,
    strum_macros::EnumCount,
)]
#[repr(u8)]
pub enum Blocks {
    #[default]
    Void = 0,
    Copper,
    Iron,
    Steel,
    Uranium,
}

impl Blocks {
    pub const fn block_properties(&self) -> BlockProperties {
        match self {
            Blocks::Void => BlockProperties::VOID,
            Blocks::Copper => BlockProperties {
                ..BlockProperties::DEFAULT
            },
            Blocks::Iron => BlockProperties::DEFAULT,
            Blocks::Steel => BlockProperties::DEFAULT,
            Blocks::Uranium => BlockProperties::URANIUM,
        }
    }
}

impl From<BlockId> for Blocks {
    fn from(id: BlockId) -> Self {
        Blocks::from_repr(id.0).unwrap_or(Blocks::Void)
    }
}

impl chunk_serde::Serialize for Blocks {
    fn insert(&self, vec: &mut BinSerializer) -> Result<usize> {
        vec.push(*self as u8);
        Ok(1)
    }
    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        #[cfg(debug_assertions)]
        return Ok((Blocks::from_repr(slice[0]).unwrap(), 1));
        #[cfg(not(debug_assertions))]
        return Ok((Blocks::from_repr(slice[0]).unwrap_or(Blocks::Void), 1));
    }
}

impl phoxels::prelude::Block for Blocks {
    fn id(&self) -> u8 {
        *self as u8
    }
    fn is_solid(&self) -> bool {
        true
    }
    fn is_transparent(&self) -> bool {
        false
    }
}
