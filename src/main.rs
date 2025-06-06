use meltdown_manager::{run_game, voxels::blocks::Blocks};
use strum::IntoEnumIterator;

#[test]
fn gen_block_meta() {
    block_meta::make_block_meta_file(Blocks::iter());
}

fn main() {
    run_game();
}
