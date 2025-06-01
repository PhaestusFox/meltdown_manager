// don't ask why I have decided to role my own serde just felt like it ;P

pub trait Serialize: Sized {
    /// Add self to stream
    /// Returns the num bytes Added
    fn into_vec(&self, vec: &mut Vec<u8>) -> usize;
    /// Get self from stream
    /// Returns Self and bytes Used
    fn from_slice(slice: &[u8]) -> (Self, usize);
}

#[derive(PartialEq, Eq, Debug)]
pub enum CompressedChunkData<T: Eq + Serialize> {
    Solid(T),
    RunLen(Vec<(T, u16)>),
    Raw(Vec<T>),
    Error(u8),
}

impl<T: Serialize> Serialize for Vec<T> {
    fn into_vec(&self, vec: &mut Vec<u8>) -> usize {
        let mut len = 4;
        for l in self.len().to_be_bytes() {
            vec.push(l);
        }
        for item in self {
            len += item.into_vec(vec);
        }
        len
    }

    fn from_slice(slice: &[u8]) -> (Self, usize) {
        let mut len = 0usize.to_be_bytes();
        let mut used = 0;
        for byte in len.iter_mut() {
            *byte = slice[used];
            used += 1;
        }
        let len = usize::from_be_bytes(len);
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            let (t, con) = T::from_slice(&slice[used..]);
            out.push(t);
            used += con;
        }
        (out, used)
    }
}

impl<T: Serialize> Serialize for (T, u16) {
    fn into_vec(&self, vec: &mut Vec<u8>) -> usize {
        let len = self.0.into_vec(vec) + 2;
        let be = self.1.to_be_bytes();
        vec.push(be[0]);
        vec.push(be[1]);
        len
    }
    fn from_slice(slice: &[u8]) -> (Self, usize) {
        let (t, at) = T::from_slice(slice);
        ((t, u16::from_be_bytes([slice[at], slice[at + 1]])), at + 2)
    }
}

impl<T: Eq + Serialize> Serialize for CompressedChunkData<T> {
    fn into_vec(&self, vec: &mut Vec<u8>) -> usize {
        match self {
            CompressedChunkData::Solid(other) => {
                vec.push(0);
                other.into_vec(vec) + 1
            }
            CompressedChunkData::RunLen(items) => {
                vec.push(1);
                items.into_vec(vec) + 1
            }
            CompressedChunkData::Raw(items) => {
                vec.push(2);
                items.into_vec(vec) + 1
            }
            CompressedChunkData::Error(i) => {
                debug_assert!(false, "Dont serialize a CompressedChunkData::Error");
                if *i < 3 {
                    vec.push(3);
                } else {
                    vec.push(*i);
                }
                1
            }
        }
    }

    fn from_slice(slice: &[u8]) -> (Self, usize) {
        match slice[0] {
            0 => {
                let (out, used) = T::from_slice(&slice[1..]);
                (CompressedChunkData::Solid(out), used + 1)
            }
            1 => {
                let (out, used) = Vec::from_slice(&slice[1..]);
                (CompressedChunkData::RunLen(out), used + 1)
            }
            2 => {
                let (out, used) = Vec::from_slice(&slice[1..]);
                (CompressedChunkData::Raw(out), used + 1)
            }
            i => (CompressedChunkData::Error(i), 1),
        }
    }
}
