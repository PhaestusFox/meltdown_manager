pub use highlight::*;
pub use neighbors::*;
pub use redraw::*;
pub use save_load::*;

mod highlight;
mod neighbors;
mod redraw;
mod save_load;

use super::AxisPointer;

pub(super) fn init(app: &mut bevy::app::App) {
    highlight::init(app);
    neighbors::init(app);
    redraw::init(app);
    save_load::init(app);
}
