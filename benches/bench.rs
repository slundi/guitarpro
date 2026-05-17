use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn example_benchmark(c: &mut Criterion) {
    // Replace with a real benchmark of your code.
    c.bench_function("example", |b| b.iter(|| black_box(1 + 1)));
}

criterion_group!(benches, example_benchmark);
criterion_main!(benches);
