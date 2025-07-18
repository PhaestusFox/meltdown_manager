use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use meltdown_manager::{
    BlockIter,
    voxels::{CHUNK_SIZE, ChunkId, block::BlockType, cellular_automata::*},
};
use rand::{Rng, SeedableRng, seq::IndexedRandom};
use std::{hint::black_box, time::Duration};
use strum::IntoEnumIterator;

fn gen_chunk() -> Cells {
    Cells::empty()
}

fn gen_random_chunk() -> Cells {
    let mut chunk = Cells::empty();
    let mut rng = rand::rngs::StdRng::from_seed([0; 32]);
    let r = BlockType::iter().collect::<Vec<_>>();
    let mut block = CellData::default();
    for (x, y, z) in BlockIter::new() {
        block.energy = FixedNum::from_num(rng.random_range(0..10000000));
        block.set_block_type(*r.choose(&mut rng).unwrap_or(&BlockType::Air));
        chunk.set_cell(x, y, z, block);
    }
    chunk
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut chunks = [
        gen_chunk(),
        gen_chunk(),
        gen_chunk(),
        gen_chunk(),
        gen_chunk(),
        gen_chunk(),
        gen_chunk(),
    ];
    let dummy = [
        Some(&chunks[0]),
        Some(&chunks[1]),
        Some(&chunks[2]),
        Some(&chunks[3]),
        Some(&chunks[4]),
        Some(&chunks[5]),
        Some(&chunks[6]),
    ];
    let blocks = ChunkData::empty();
    c.bench_function("gen_empty", |b| b.iter(|| black_box(gen_chunk())));
    let mut c = c.benchmark_group("Step");
    c.measurement_time(Duration::from_secs(30));
    c.bench_function("step empty", |b| {
        b.iter(|| {
            let mut chunk = gen_chunk();
            for x in 0..16 {
                #[cfg(debug_assertions)]
                step(
                    ChunkIter::new(&mut chunk),
                    ChunkGared::new(dummy, ChunkId::new(0, 0, 0)),
                    0,
                );
                #[cfg(not(debug_assertions))]
                step(ChunkIter::new(&mut chunk), ChunkGared::new(dummy), 0);
            }
        })
    });

    chunks[0] = gen_random_chunk();

    let dummy = [
        Some(&chunks[0]),
        Some(&chunks[1]),
        Some(&chunks[2]),
        Some(&chunks[3]),
        Some(&chunks[4]),
        Some(&chunks[5]),
        Some(&chunks[6]),
    ];
    c.bench_function("step random", |b| {
        b.iter(|| {
            let mut chunk = gen_chunk();
            for x in 0..16 {
                #[cfg(debug_assertions)]
                step(
                    ChunkIter::new(&mut chunk),
                    ChunkGared::new(dummy, ChunkId::new(0, 0, 0)),
                    0,
                );
                #[cfg(not(debug_assertions))]
                step(ChunkIter::new(&mut chunk), ChunkGared::new(dummy), 0);
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
