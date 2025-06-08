use meltdown_manager::{run_game, voxels::block::BlockType};
use strum::IntoEnumIterator;

#[test]
fn gen_block_meta() {
    block_meta::make_block_meta_file(BlockType::iter());
}

fn main() {
    run_game();
}
