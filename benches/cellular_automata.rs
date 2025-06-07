use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use meltdown_manager::voxels::{ChunkId, blocks::Blocks, cellular_automata::*};
use rand::{Rng, SeedableRng, seq::IndexedRandom};
use std::hint::black_box;
use strum::IntoEnumIterator;

fn gen_chunk() -> Cells {
    Cells::empty()
}

fn gen_random_chunk() -> Cells {
    let mut chunk = Cells::empty();
    let mut rng = rand::rngs::StdRng::from_seed([0; 32]);
    let r = Blocks::iter().collect::<Vec<_>>();
    let mut block = CellData::default();
    for x in 0..30 {
        for y in 0..30 {
            for z in 0..30 {
                block.energy = FixedNum::from_num(rng.random_range(0..10000000));
                block.charge = FixedNum::from_num(rand::random::<f64>());
                block.presure = FixedNum::from_num(rand::random::<f64>());
                block.set_block(*r.choose(&mut rng).unwrap_or(&Blocks::Air));
                chunk.set_block(x, y, z, block);
            }
        }
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
    c.bench_function("step empty", |b| {
        b.iter(|| {
            let mut chunk = gen_chunk();
            #[cfg(debug_assertions)]
            step(
                ChunkIter::new(&mut chunk),
                ChunkGared::new(dummy, ChunkId::new(0, 0, 0)),
                0,
            );
            #[cfg(not(debug_assertions))]
            step(ChunkIter::new(&mut chunk), ChunkGared::new(dummy), 0);
        })
    });
    for i in [0, 1, 3, 7] {
        c.bench_with_input(
            BenchmarkId::new("Step Empty", StepMode::from_bits_retain(i)),
            &i,
            |b, i| {
                b.iter(|| {
                    let mut chunk = gen_chunk();
                    #[cfg(debug_assertions)]
                    step(
                        ChunkIter::new(&mut chunk),
                        ChunkGared::new(dummy, ChunkId::new(0, 0, 0)),
                        *i,
                    );
                    #[cfg(not(debug_assertions))]
                    step(ChunkIter::new(&mut chunk), ChunkGared::new(dummy), *i);
                })
            },
        );
    }

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
            #[cfg(debug_assertions)]
            step(
                ChunkIter::new(&mut chunk),
                ChunkGared::new(dummy, ChunkId::new(0, 0, 0)),
                0,
            );
            #[cfg(not(debug_assertions))]
            step(ChunkIter::new(&mut chunk), ChunkGared::new(dummy), 0);
        })
    });
    c.bench_function("step random phase change", |b| {
        b.iter(|| {
            let mut chunk = gen_chunk();
            #[cfg(debug_assertions)]
            step(
                ChunkIter::new(&mut chunk),
                ChunkGared::new(dummy, ChunkId::new(0, 0, 0)),
                1,
            );
            #[cfg(not(debug_assertions))]
            step(ChunkIter::new(&mut chunk), ChunkGared::new(dummy), 1);
        })
    });

    c.bench_function("step random gravity", |b| {
        b.iter(|| {
            let mut chunk = gen_chunk();
            #[cfg(debug_assertions)]
            step(
                ChunkIter::new(&mut chunk),
                ChunkGared::new(dummy, ChunkId::new(0, 0, 0)),
                3,
            );
            #[cfg(not(debug_assertions))]
            step(ChunkIter::new(&mut chunk), ChunkGared::new(dummy), 1);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
