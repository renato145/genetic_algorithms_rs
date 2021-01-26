use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ndarray::Array1;
use ndarray_rand::{rand_distr::Uniform, RandomExt};
use rand::{distributions::Uniform as NDUniform, Rng};

fn plain_rust(ndim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let range = Uniform::new_inclusive(-1.0f32, 1.0f32);
    (&mut rng).sample_iter(&range).take(ndim).collect()
}

fn with_ndarray(ndim: usize) -> Array1<f32> {
    let range = NDUniform::new_inclusive(-1.0f32, 1.0f32);
    Array1::random(ndim, &range)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Create indiviual");

    let ndims: Vec<_> = (1..=10).map(|o| o * 10).collect();

    for ndim in ndims.iter() {
        group.bench_with_input(BenchmarkId::new("Plain Rust", ndim), ndim, |b, ndim| {
            b.iter(|| plain_rust(*ndim))
        });
        group.bench_with_input(BenchmarkId::new("With ndarray", ndim), ndim, |b, ndim| {
            b.iter(|| with_ndarray(*ndim))
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
