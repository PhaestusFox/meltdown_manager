use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use meltdown_manager::voxels::{blocks::Blocks, cellular_automata::*};
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
    for x in 0..30 {
        for y in 0..30 {
            for z in 0..30 {
                chunk.set_block(
                    x,
                    y,
                    z,
                    CellData {
                        block: *r.choose(&mut rng).unwrap_or(&Blocks::Air),
                        energy: FixedNum::from_num(rng.random_range(0..10000000)),
                        charge: FixedNum::from_num(rand::random::<f64>()),
                        presure: FixedNum::from_num(rand::random::<f64>()),
                        flags: CellFlags::empty(),
                    },
                );
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
            step(ChunkIter::new(&mut chunk), ChunkGared::new(dummy));
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
            step(ChunkIter::new(&mut chunk), ChunkGared::new(dummy));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
