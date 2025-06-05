#[cfg(not(target_arch = "wasm32"))]
mod not_wasm {
    use bevy::ecs::world::{FromWorld, World};

    use crate::voxels::cellular_automata::{CellData, CellFlags, FixedNum};

    pub struct MaxValue {
        max: CellData,
        channel: std::sync::mpsc::Receiver<CellData>,
        sender: std::sync::mpsc::Sender<CellData>,
    }

    impl MaxValue {
        pub fn get_sender(&self) -> std::sync::mpsc::Sender<CellData> {
            self.sender.clone()
        }

        pub fn get_max(&self) -> CellData {
            self.max
        }

        pub fn restart(&mut self) {
            self.max = CellData {
                block: crate::voxels::blocks::Blocks::Void,
                energy: FixedNum::ONE,
                presure: FixedNum::ONE,
                charge: FixedNum::ONE,
                flags: CellFlags::empty(),
            };
        }

        pub fn run(&mut self) {
            while let Ok(data) = self.channel.try_recv() {
                self.max.max(&data);
            }
        }
    }

    impl FromWorld for MaxValue {
        fn from_world(_: &mut World) -> Self {
            let (sender, channel) = std::sync::mpsc::channel();
            MaxValue {
                max: CellData {
                    block: crate::voxels::blocks::Blocks::Void,
                    energy: FixedNum::ONE,
                    presure: FixedNum::ONE,
                    charge: FixedNum::ONE,
                    flags: CellFlags::empty(),
                },
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
