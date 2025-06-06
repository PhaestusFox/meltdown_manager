use rand::{
    Rng,
    seq::{IndexedRandom, SliceRandom},
};
use strum::IntoEnumIterator;

use crate::{
    utils::BlockIter,
    voxels::{CHUNK_VOL, blocks::Blocks, voxel_chunk::chunk::Chunk},
};
use chunk_serde::{BinSerializer, CompressedChunkData};

#[test]
fn chunk_compression() {
    let empty = Chunk::empty();
    let solid = Chunk::solid(Blocks::Copper);

    // check empty is solid air
    assert_eq!(CompressedChunkData::Solid(Blocks::Void), empty.compress());
    // check solid is solid block
    assert_eq!(CompressedChunkData::Solid(Blocks::Copper), solid.compress());

    let empty_cop = empty.compress();
    let solid_cop = solid.compress();
    // check empty.comp == empty.decomp
    assert_eq!(Chunk::decompress(&empty_cop), empty);
    // check solid.comp == solid.decomp
    assert_eq!(Chunk::decompress(&solid_cop), solid);

    // steal empty because why not
    let mut chunk = empty;
    // set 0,0,0 to something else
    chunk.set_block(0, 0, 0, Blocks::Copper);

    // check RLE is chosen since 2 runs = 6 bytes < 27000 bytes of raw
    assert_eq!(
        CompressedChunkData::RunLen(vec![
            (Blocks::Copper, 1),
            (Blocks::Void, (CHUNK_VOL - 1) as u16)
        ]),
        chunk.compress()
    );

    let comp = chunk.compress();
    assert_eq!(Chunk::decompress(&comp), chunk);

    let mut raw = vec![Blocks::Void; CHUNK_VOL];
    // set every other block copper
    for (x, y, z) in BlockIter::<30, 30, 30>::new().step_by(2) {
        raw[Chunk::<Blocks>::index(x, y, z)] = Blocks::Copper;
        chunk.set_block(x, y, z, Blocks::Copper);
    }

    // check that is worse case for RLE is uses raw, 27000 runs = 81000 bytes > 27000 bytes of raw
    assert_eq!(CompressedChunkData::Raw(raw), chunk.compress());

    let comp = chunk.compress();

    // check raw decompress back to chunk
    assert_eq!(Chunk::decompress(&comp), chunk);

    // reuse solid because why not
    let mut chunk = solid;
    // todo add transiton test for RLE
    let mut r = 1;
    for (x, y, z) in BlockIter::<30, 30, 30>::new().step_by(2) {
        chunk.set_block(x, y, z, Blocks::Iron);
        r += 2;
        if r >= 9000 {
            break;
        }
    }

    if let CompressedChunkData::Raw(_) = chunk.compress() {
        panic!("Should be RLE");
    }

    chunk.set_block(20, 20, 20, Blocks::Uranium);

    if let CompressedChunkData::RunLen(_) = chunk.compress() {
        panic!("Should be Raw");
    }
}

#[test]
fn chunk_serde() {
    use chunk_serde::Serialize;
    let mut chunk = Chunk::empty();
    let mut data = BinSerializer::new();
    let mut out = CompressedChunkData::Error(69);
    let mut comp = chunk.compress();
    let mut len = 0;

    macro_rules! test {
        ($generator:expr) => {
            data.clear();
            comp = $generator.compress();
            data.insert(&comp);
            (out, len) = CompressedChunkData::extract(data.as_ref()).unwrap();
            assert_eq!(len, data.len());
            assert_eq!(out, comp);
        };
    }

    test!(Chunk::empty());
    for block in Blocks::iter() {
        test!(Chunk::solid(block));
    }
    chunk.set_block(0, 0, 0, Blocks::Copper);
    test!(chunk);

    let mut raw = vec![Blocks::Void; CHUNK_VOL];
    // set every other block copper
    for (x, y, z) in BlockIter::<30, 30, 30>::new().step_by(2) {
        raw[Chunk::<Blocks>::index(x, y, z)] = Blocks::Copper;
        chunk.set_block(x, y, z, Blocks::Copper);
    }

    test!(chunk);

    let raw = CompressedChunkData::Raw(raw);
    test!(Chunk::decompress(&raw));
    let raw_len = len;
    assert_eq!(raw_len, 27000 + 1 + size_of::<usize>()); // 1 byte for compression type + 8 for Vec len + 27000 for blocks

    chunk = Chunk::empty();

    let mut r = 1;
    for (x, y, z) in BlockIter::<30, 30, 30>::new().step_by(2) {
        chunk.set_block(x, y, z, Blocks::Iron);
        r += 2;
        if r >= 9000 {
            break;
        }
    }

    test!(chunk);
    let w_rle = len;
    assert!(data[0] == 1);

    chunk.set_block(20, 20, 20, Blocks::Uranium);

    test!(chunk);
    assert!(w_rle <= raw_len);
}

#[test]
fn fuzz_compression() {
    let mut chunk = Chunk::empty();
    let mut rng = rand::thread_rng();
    let block: Vec<_> = Blocks::iter().collect();
    for _ in 0..27000 {
        chunk.set_block(
            rng.random_range(0..30),
            rng.random_range(0..30),
            rng.random_range(0..30),
            *block.choose(&mut rng).unwrap(),
        );
        let comp = chunk.compress();
        assert_eq!(Chunk::decompress(&comp), chunk);
    }
}

#[test]
fn fuzz_serde() {
    let mut chunk = Chunk::empty();
    let mut rng = rand::thread_rng();
    let block: Vec<_> = Blocks::iter().collect();

    use chunk_serde::Serialize;
    let mut data = BinSerializer::new();
    let mut out = CompressedChunkData::Error(69);
    let mut comp = chunk.compress();
    let mut len = 0;

    macro_rules! test {
        ($generator:expr) => {
            data.clear();
            comp = $generator.compress();
            data.insert(&comp);
            (out, len) = CompressedChunkData::extract(data.as_ref()).unwrap();
            assert_eq!(len, data.len());
            assert_eq!(out, comp);
        };
    }
    for _ in 0..27000 {
        chunk.set_block(
            rng.random_range(0..30),
            rng.random_range(0..30),
            rng.random_range(0..30),
            *block.choose(&mut rng).unwrap(),
        );
        test!(chunk);
    }
}
