use block_meta::FixedNum;

use crate::voxels::cellular_automata::CellData;

#[test]
fn water_is_liquid_at_20c() {
    let cell = CellData::at_k(
        crate::voxels::block::BlockType::Water,
        FixedNum::lit("293.15"),
    );
    assert!(
        cell.flags
            .contains(crate::voxels::cellular_automata::cells::CellFlags::IS_LIQUID)
    );
    assert!(
        !cell
            .flags
            .contains(crate::voxels::cellular_automata::cells::CellFlags::IS_GAS),
    );
}

#[test]
fn water_is_more_dense_than_air() {
    let water = CellData::at_k(
        crate::voxels::block::BlockType::Water,
        FixedNum::lit("293.15"),
    );
    let air = CellData::at_k(
        crate::voxels::block::BlockType::Air,
        FixedNum::lit("293.15"),
    );
    assert!(water.properties().density > air.properties().density);
}
