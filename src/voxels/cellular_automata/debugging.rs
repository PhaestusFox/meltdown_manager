use super::*;
pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (melt_all, boil_all, set_all_300k)
            .run_if(can_fuck_with_next_step)
            .run_if(is_cheat),
    );
}

fn melt_all(mut chunks: Query<&mut Cells>, input: Res<ButtonInput<KeyCode>>) {
    if !input.just_pressed(KeyCode::KeyM) {
        return;
    }
    for mut chunk in &mut chunks {
        for block in chunk.iter_mut() {
            block.flags.set(CellFlags::IS_LIQUID, true);
        }
    }
}

fn boil_all(mut chunks: Query<&mut Cells>, input: Res<ButtonInput<KeyCode>>) {
    if !input.just_pressed(KeyCode::KeyG) {
        return;
    }
    for mut chunk in &mut chunks {
        for block in chunk.iter_mut() {
            block.flags.set(CellFlags::IS_GAS, true);
        }
    }
}

fn set_all_300k(mut chunks: Query<&mut Cells>, input: Res<ButtonInput<KeyCode>>) {
    if !input.just_pressed(KeyCode::KeyF) {
        return;
    }
    for mut chunk in &mut chunks {
        for block in chunk.iter_mut() {
            let props = block.get_block_type().properties();
            let le = props.specific_heat;
            let mp = props.melting_point;
            let re = if mp < FixedNum::lit("300") {
                le * FixedNum::lit("300") + mp
            } else {
                le * FixedNum::lit("300")
            };
            block.energy = re;
        }
    }
}

fn is_cheat(input: Res<ButtonInput<KeyCode>>) -> bool {
    input.pressed(KeyCode::Backquote)
}
