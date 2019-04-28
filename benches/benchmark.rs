#[macro_use]
extern crate criterion;

use criterion::Criterion;
use criterion::black_box;

#[path = "../src/bit_array.rs"] 
mod bit_array;


fn bench_bit_array(c: &mut Criterion) {
    let mut pp = bit_array::BitArray::new(40, 11);
    c.bench_function("put BA", move |b| b.iter(|| pp.put(black_box(20u32), 7)));

    let pp = bit_array::BitArray::new(40, 11);
    c.bench_function("get BA", move |b| b.iter(|| pp.get(black_box(20u32))));
}

criterion_group!(benches, bench_bit_array);


criterion_main!(benches);