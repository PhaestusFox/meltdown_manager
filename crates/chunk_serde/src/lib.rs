// don't ask why I have decided to role my own serde just felt like it ;P

use std::fmt::Write;

use bevy_ecs::error::BevyError;
use bevy_ecs::error::Result;

pub struct BinSerializer {
    index: usize,
    data: Vec<u8>,
}

pub struct BinDeSerializer<'a> {
    index: usize,
    data: &'a [u8],
}

impl BinDeSerializer<'_> {
    pub fn new(data: &[u8]) -> BinDeSerializer {
        BinDeSerializer { index: 0, data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn extract<T: Serialize>(&mut self) -> Result<T> {
        let (v, used) = T::extract(&self.data[self.index..])?;
        self.index += used;
        Ok(v)
    }
}

impl std::ops::Index<usize> for BinSerializer {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl AsRef<[u8]> for BinSerializer {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl Default for BinSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl BinSerializer {
    pub fn new() -> BinSerializer {
        BinSerializer {
            index: 0,
            data: Vec::new(),
        }
    }

    pub fn push(&mut self, byte: u8) {
        self.data.push(byte);
    }
    pub fn finalize(self) -> Vec<u8> {
        self.data
    }
    pub fn clear(&mut self) {
        self.index = 0;
        self.data.clear();
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn insert<T: Serialize>(&mut self, val: &T) -> Result<usize> {
        match val.insert(self) {
            Ok(v) => {
                self.index += v;
                Ok(v)
            }
            Err(e) => {
                self.data.truncate(self.index);
                Err(e)
            }
        }
    }

    pub fn extract<T: Serialize>(&mut self) -> Result<T> {
        let (v, used) = T::extract(&self.data[self.index..])?;
        self.index += used;
        Ok(v)
    }
}

pub struct StrSerializer {
    index: usize,
    data: String,
}

impl StrSerializer {
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn in_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn insert<T: Serialize>(&mut self, val: &T) -> Result<usize> {
        val.insert_str(self)
    }
    pub fn extract<T: Serialize>(&mut self) -> Result<T> {
        let (v, used) = T::extract_str(&self.data[self.index..])?;
        self.index += used;
        Ok(v)
    }
    pub fn push(&mut self, ch: char) {
        self.data.push(ch);
    }

    pub fn push_str(&mut self, str: &str) {
        self.data.push_str(str);
    }

    pub fn write(&mut self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        self.data.write_fmt(args)
    }
}

pub trait Serialize: Sized {
    /// Add self to binarty serializer
    /// Returns the num bytes Added
    fn insert(&self, serializer: &mut BinSerializer) -> Result<usize>;
    /// Get self from slice
    /// Returns Self and bytes Used
    fn extract(slice: &[u8]) -> Result<(Self, usize)>;
    /// Add self to str serializer
    /// Returns the num char Added
    fn insert_str(&self, _serializer: &mut StrSerializer) -> Result<usize> {
        unimplemented!()
    }

    /// Get self from str
    /// Returns the num char Used
    fn extract_str(_str: &str) -> Result<(Self, usize)> {
        unimplemented!()
    }
}

#[derive(Eq, Debug)]
pub enum CompressedChunkData<T> {
    Solid(T),
    RunLen(Vec<(T, u16)>),
    Raw(Vec<T>),
    Error(u8),
}

impl<T: Eq> PartialEq for CompressedChunkData<T> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            CompressedChunkData::Solid(v) => {
                if let CompressedChunkData::Solid(o) = other {
                    v == o
                } else {
                    false
                }
            }
            CompressedChunkData::RunLen(v) => {
                if let CompressedChunkData::RunLen(o) = other {
                    v == o
                } else {
                    false
                }
            }
            CompressedChunkData::Raw(v) => {
                if let CompressedChunkData::Raw(o) = other {
                    v == o
                } else {
                    false
                }
            }
            CompressedChunkData::Error(_) => false,
        }
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn insert(&self, vec: &mut BinSerializer) -> Result<usize> {
        let mut len = 4;
        for l in self.len().to_be_bytes() {
            vec.push(l);
        }
        for item in self {
            len += item.insert(vec)?;
        }
        Ok(len)
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        let mut len = 0usize.to_be_bytes();
        let mut used = 0;
        for byte in len.iter_mut() {
            *byte = slice[used];
            used += 1;
        }
        let len = usize::from_be_bytes(len);
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            let (t, con) = T::extract(&slice[used..])?;
            out.push(t);
            used += con;
        }
        Ok((out, used))
    }

    fn insert_str(&self, serializer: &mut StrSerializer) -> Result<usize> {
        let mut used = 2;
        serializer.push('[');
        serializer.push('\n');
        let mut trailing = false;
        for val in self.iter() {
            if trailing {
                serializer.push(',');
                serializer.push('\n');
                used += 2;
            } else {
                trailing = true;
            }
            used += val.insert_str(serializer)?;
        }
        used += 2;
        serializer.push_str("\n]");
        Ok(used)
    }

    fn extract_str(str: &str) -> Result<(Self, usize)> {
        let mut used = 0;
        let mut skip = 0;
        let mut out = None;
        let mut exp = '[';
        for char in str.chars() {
            used += 1;
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if char.is_whitespace() {
                continue;
            }
            if char == '[' && out.is_none() {
                let mut var = Vec::new();
                let (res, len) = T::extract_str(&str[used..])?;
                var.push(res);
                skip += len;
                out = Some(var);
                exp = ']';
                continue;
            }
            if char == ']' && out.is_some() {
                break;
            }
            if char == ',' && out.is_none() {
                if let Some(ref mut out) = out {
                    let (res, len) = T::extract_str(&str[used..])?;
                    out.push(res);
                    skip += len;
                    continue;
                }
            }
            Err(StrError::WrongChar(exp, char))?;
        }
        Ok((out.ok_or(StrError::EOF)?, used))
    }
}

impl<T: Serialize> Serialize for (T, u16) {
    fn insert(&self, vec: &mut BinSerializer) -> Result<usize> {
        let len = self.0.insert(vec)? + 2;
        let be = self.1.to_be_bytes();
        vec.push(be[0]);
        vec.push(be[1]);
        Ok(len)
    }
    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        let (t, at) = T::extract(slice)?;
        Ok(((t, u16::from_be_bytes([slice[at], slice[at + 1]])), at + 2))
    }
    fn insert_str(&self, serializer: &mut StrSerializer) -> Result<usize> {
        let start = serializer.len();
        serializer.push('(');
        self.0.insert_str(serializer)?;
        serializer.push(',');
        serializer.write(format_args!("{}", self.1));
        serializer.push(')');
        Ok(serializer.len() - start)
    }
    fn extract_str(str: &str) -> Result<(Self, usize)> {
        let mut used = 0;
        let mut out = None;
        let mut expt = '(';
        let mut len = None;
        let mut skip = 0;
        for char in str.chars() {
            used += 1;
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if char.is_whitespace() {
                continue;
            }
            if char == '(' && out.is_none() {
                let (res, con) = T::extract_str(&str[used..])?;
                out = Some(res);
                skip += con;
                expt = ',';
                continue;
            }
            if char == ',' && out.is_some() && len.is_none() {
                let (res, con) = u16::extract_str(&str[used..])?;
                len = Some(res);
                skip += con;
                expt = ')';
            }
            if char == ')' && len.is_some() {
                break;
            }
            return Err(StrError::WrongChar(expt, char))?;
        }
        match (out, len) {
            (Some(out), Some(len)) => Ok(((out, len), used)),
            (None, None) | (None, Some(_)) => Err(StrError::TupleError(0))?,
            (Some(_), None) => Err(StrError::TupleError(1))?,
        }
    }
}

impl Serialize for u16 {
    fn insert(&self, serializer: &mut BinSerializer) -> Result<usize> {
        let b = self.to_be_bytes();
        serializer.push(b[0]);
        serializer.push(b[1]);
        Ok(2)
    }
    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        if slice.len() < 2 {
            Err(BinError::EOF)?
        }
        Ok((u16::from_be_bytes([slice[0], slice[1]]), 2))
    }
    fn insert_str(&self, serializer: &mut StrSerializer) -> Result<usize> {
        let start = serializer.len();
        serializer.write(format_args!("{}", self))?;
        Ok(serializer.len() - start)
    }
    fn extract_str(str: &str) -> Result<(Self, usize)> {
        let mut used = 0;
        let mut res = String::new();
        for char in str.chars() {
            if char.is_whitespace() && res.is_empty() {
                used += 1;
                continue;
            }
            if char.is_numeric() {
                used += 1;
                res.push(char);
                continue;
            }
            if res.is_empty() {
                Err(StrError::ExpectDigit(char))?;
            }
            break;
        }
        Ok((u16::from_str_radix(&res, 10)?, used))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BinError {
    #[error("EOF")]
    EOF,
}

#[derive(thiserror::Error, Debug)]
pub enum StrError {
    #[error("EOF")]
    EOF,
    #[error("Expected {0} found {1}")]
    WrongChar(char, char),
    #[error("Expected Digtit found {0}")]
    ExpectDigit(char),
    #[error("Touple Missing field {0}")]
    TupleError(u8),
    #[error("Int Parse Error {0}")]
    IntParseError(#[from] std::num::ParseIntError),
    #[error("Ivalid Name: {0}\nMust be one of:\n{1:?}")]
    InValidName(String, &'static [&'static str]),
    #[error("Missing Char: Expected {0} before EOF")]
    ExpectChar(char),
}

impl<T: Eq + Serialize> Serialize for CompressedChunkData<T> {
    fn insert(&self, vec: &mut BinSerializer) -> Result<usize, BevyError> {
        Ok(match self {
            CompressedChunkData::Solid(other) => {
                vec.push(0);
                other.insert(vec)? + 1
            }
            CompressedChunkData::RunLen(items) => {
                vec.push(1);
                items.insert(vec)? + 1
            }
            CompressedChunkData::Raw(items) => {
                vec.push(2);
                items.insert(vec)? + 1
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
        })
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        match slice[0] {
            0 => {
                let (out, used) = T::extract(&slice[1..])?;
                Ok((CompressedChunkData::Solid(out), used + 1))
            }
            1 => {
                let (out, used) = Vec::extract(&slice[1..])?;
                Ok((CompressedChunkData::RunLen(out), used + 1))
            }
            2 => {
                let (out, used) = Vec::extract(&slice[1..])?;
                Ok((CompressedChunkData::Raw(out), used + 1))
            }
            i => Ok((CompressedChunkData::Error(i), 1)),
        }
    }

    fn insert_str(&self, serializer: &mut StrSerializer) -> Result<usize> {
        let mut used = 0;
        match self {
            CompressedChunkData::Solid(v) => {
                used += 7;
                serializer.push_str("Solid(");
                used += v.insert_str(serializer)?;
                serializer.push(')');
            }
            CompressedChunkData::RunLen(items) => {
                used += 8;
                serializer.push_str("RunLen(");
                used += items.insert_str(serializer)?;
                serializer.push(')');
            }
            CompressedChunkData::Raw(items) => {
                used += 5;
                serializer.push_str("Raw(");
                used += items.insert_str(serializer)?;
                serializer.push(')');
            }
            CompressedChunkData::Error(_) => {
                debug_assert!(false, "Don't Serialize Errors");
                used += 7;
                serializer.push_str("Error()");
            }
        }
        Ok(used)
    }

    fn extract_str(str: &str) -> Result<(Self, usize)> {
        let mut used = 0;
        let mut target = String::new();
        for char in str.chars() {
            used += 1;
            if char.is_whitespace() && target.is_empty() {
                continue;
            }
            if char == '(' && !target.is_empty() {
                return match target.as_str() {
                    "Solid" => {
                        let (res, len) = T::extract_str(&str[used..])?;
                        used += len;
                        Ok((CompressedChunkData::Solid(res), used))
                    }
                    "RunLen" => {
                        let (res, len) = Vec::extract_str(&str[used..])?;
                        used += len;
                        Ok((CompressedChunkData::RunLen(res), used))
                    }
                    "Raw" => {
                        let (res, len) = Vec::extract_str(&str[used..])?;
                        used += len;
                        Ok((CompressedChunkData::Raw(res), used))
                    }
                    _ => Err(StrError::InValidName(target, &["Solid", "RunLen", "Raw"]).into()),
                };
            }
            target.push(char);
        }
        Err(StrError::EOF.into())
    }
}

impl Serialize for i32 {
    fn insert(&self, serializer: &mut BinSerializer) -> Result<usize> {
        let bytes = self.to_be_bytes();
        serializer.push(bytes[0]);
        serializer.push(bytes[1]);
        serializer.push(bytes[2]);
        serializer.push(bytes[3]);
        Ok(4)
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        if slice.len() < 4 {
            Err(BinError::EOF)?
        }
        let bytes = [slice[0], slice[1], slice[2], slice[3]];
        Ok((i32::from_be_bytes(bytes), 4))
    }
}

#[test]
fn test_i32_serialize() {
    let mut serializer = BinSerializer::new();
    let value: i32 = 12345678;
    let size = value.insert(&mut serializer).unwrap();
    assert_eq!(size, 4);
    let (extracted_value, used) = i32::extract(serializer.as_ref()).unwrap();
    assert_eq!(extracted_value, value);
    assert_eq!(used, 4);

    let mut serializer = BinSerializer::new();

    let value: i32 = -1;
    let size = value.insert(&mut serializer).unwrap();
    assert_eq!(size, 4);
    let (extracted_value, used) = i32::extract(serializer.as_ref()).unwrap();
    assert_eq!(extracted_value, value);
    assert_eq!(used, 4);

    serializer.insert(&12345678).unwrap();

    let mut de = BinDeSerializer::new(serializer.as_ref());
    let extracted_value = de.extract::<i32>().unwrap();
    assert_eq!(extracted_value, -1);

    let extracted_value = de.extract::<i32>().unwrap();
    assert_eq!(extracted_value, 12345678);
}

impl Serialize for u64 {
    fn insert(&self, serializer: &mut BinSerializer) -> Result<usize> {
        let bytes = self.to_be_bytes();
        for byte in bytes.iter() {
            serializer.push(*byte);
        }
        Ok(8)
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        if slice.len() < 8 {
            Err(BinError::EOF)?
        }
        let mut bytes = [0; 8];
        bytes.copy_from_slice(&slice[..8]);
        Ok((u64::from_be_bytes(bytes), 8))
    }
}

#[test]
fn test_u64_serialize() {
    let mut serializer = BinSerializer::new();
    let value: u64 = 1234567890123456789;
    let size = value.insert(&mut serializer).unwrap();
    assert_eq!(size, 8);
    let (extracted_value, used) = u64::extract(serializer.as_ref()).unwrap();
    assert_eq!(extracted_value, value);
    assert_eq!(used, 8);

    let mut serializer = BinSerializer::new();

    let value: u64 = 0xFFFFFFFFFFFFFFFF;
    let size = value.insert(&mut serializer).unwrap();
    assert_eq!(size, 8);
    let (extracted_value, used) = u64::extract(serializer.as_ref()).unwrap();
    assert_eq!(extracted_value, value);
    assert_eq!(used, 8);

    serializer.insert(&1234567890123456789u64).unwrap();

    let mut de = BinDeSerializer::new(serializer.as_ref());
    let extracted_value = de.extract::<u64>().unwrap();
    assert_eq!(extracted_value, 0xFFFFFFFFFFFFFFFF);
    let extracted_value = de.extract::<u64>().unwrap();
    assert_eq!(extracted_value, 1234567890123456789);
}

impl<const N: usize> Serialize for [u8; N] {
    fn insert(&self, vec: &mut BinSerializer) -> Result<usize> {
        vec.data.extend_from_slice(self);
        Ok(N)
    }

    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        if slice.len() < N {
            Err(BinError::EOF)?
        }
        let mut out = [0; N];
        out.copy_from_slice(&slice[..N]);
        Ok((out, N))
    }
}

#[test]
fn test_array_serialize() {
    let mut serializer = BinSerializer::new();
    let value: [u8; 4] = [1, 2, 3, 4];
    let size = value.insert(&mut serializer).unwrap();
    assert_eq!(size, 4);
    let (extracted_value, used) = <[u8; 4]>::extract(serializer.as_ref()).unwrap();
    assert_eq!(extracted_value, value);
    assert_eq!(used, 4);

    let mut serializer = BinSerializer::new();

    let value: [u8; 8] = [5, 6, 7, 8, 9, 10, 11, 12];
    let size = value.insert(&mut serializer).unwrap();
    assert_eq!(size, 8);
    let (extracted_value, used) = <[u8; 8]>::extract(serializer.as_ref()).unwrap();
    assert_eq!(extracted_value, value);
    assert_eq!(used, 8);

    serializer.insert(&[1, 2, 3, 4]).unwrap();
    let mut de = BinDeSerializer::new(serializer.as_ref());
    let extracted_value = de.extract::<[u8; 4]>().unwrap();
    assert_eq!(extracted_value, [5, 6, 7, 8]);
    let extracted_value = de.extract::<[u8; 8]>().unwrap();
    assert_eq!(extracted_value, [9, 10, 11, 12, 1, 2, 3, 4]); // Padding for the array size
}
