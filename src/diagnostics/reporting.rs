#[cfg(not(target_arch = "wasm32"))]
mod not_wasm {
    use bevy::ecs::world::{FromWorld, World};
    use fixed::traits::Fixed;

    use crate::voxels::cellular_automata::{CellData, CellFlags, FixedNum};

    pub struct MaxValue {
        max_temp: FixedNum,
        channel: std::sync::mpsc::Receiver<CellData>,
        sender: std::sync::mpsc::Sender<CellData>,
    }

    impl MaxValue {
        pub fn get_sender(&self) -> std::sync::mpsc::Sender<CellData> {
            self.sender.clone()
        }

        pub fn get_max(&self) -> FixedNum {
            self.max_temp
        }

        pub fn restart(&mut self) {
            self.max_temp = FixedNum::ONE;
        }

        pub fn run(&mut self) {
            while let Ok(data) = self.channel.try_recv() {
                self.max_temp = self.max_temp.max(data.temperature());
            }
        }
    }

    impl FromWorld for MaxValue {
        fn from_world(_: &mut World) -> Self {
            let (sender, channel) = std::sync::mpsc::channel();
            MaxValue {
                max_temp: FixedNum::ZERO,
                channel,
                sender,
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::voxels::cellular_automata::CellData;
    #[derive(Default)]
    pub struct MaxValue;

    impl MaxValue {
        pub fn get_sender(&self) -> FakeSender {
            FakeSender
        }

        pub fn get_max(&self) -> CellData {
            CellData::MAX
        }

        pub fn restart(&mut self) {}

        pub fn run(&mut self) {}
    }

    #[derive(Clone)]
    pub struct FakeSender;

    impl FakeSender {
        pub fn send(&self, _data: CellData) {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use not_wasm::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
