use super::*;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (melt_all, boil_all, set_all_20c).run_if(can_fuck_with_next_step),
    );
}

fn melt_all(mut chunks: Query<&mut Cells>, input: Res<ButtonInput<KeyCode>>) {}

fn boil_all() {}

fn set_all_20c() {}
