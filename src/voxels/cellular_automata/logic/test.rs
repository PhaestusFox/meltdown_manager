use block_meta::FixedNum;

use crate::voxels::cellular_automata::CellData;

#[test]
fn water_is_liquid_at_20c() {
    let cell = CellData::at_k(
        crate::voxels::blocks::Blocks::Water,
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
