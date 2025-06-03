use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use meltdown_manager::voxels::cellular_automata::*;
use std::hint::black_box;

fn gen_chunk() -> Cells {
    Cells::empty()
}

fn gen_random_chunk() -> Cells {
    let mut chunk = Cells::empty();
    for x in 0..30 {
        for y in 0..30 {
            for z in 0..30 {
                chunk.set_block(
                    x,
                    y,
                    z,
                    CellData {
                        temperature: FixedNum::from_num(rand::random::<f64>()),
                        charge: FixedNum::from_num(rand::random::<f64>()),
                        presure: FixedNum::from_num(rand::random::<f64>()),
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
            step(ChunkIter::new(&mut chunk, &blocks), ChunkGared::new(dummy));
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
            step(ChunkIter::new(&mut chunk, &blocks), ChunkGared::new(dummy));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
