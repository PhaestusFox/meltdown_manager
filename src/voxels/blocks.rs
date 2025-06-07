use bevy::prelude::*;
use chunk_serde::BinSerializer;
use phoxels::core::BlockId;

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
    strum_macros::AsRefStr,
)]
#[repr(u8)]
pub enum Blocks {
    #[default]
    Air = 0,
    Copper,
    Iron,
    Steel,
    Uranium,
    Water,
    Void,
}

impl std::fmt::Display for Blocks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl Blocks {
    pub const fn properties(&self) -> &'static block_meta::properties::BlockProperties {
        block_meta::block_properties(*self as u8)
    }

    pub const fn meta(&self) -> &'static block_meta::computed::BlockMeta {
        block_meta::block_meta(*self as u8)
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
        !matches!(self, Blocks::Void | Blocks::Air)
    }
    fn is_transparent(&self) -> bool {
        matches!(self, Blocks::Air | Blocks::Void)
    }
}
