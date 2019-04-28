#[macro_use]
extern crate criterion;

use criterion::Criterion;
use criterion::black_box;



#[path = "../src/bitindex.rs"] 
mod bitindex;
use bitindex::*;

fn bench_bit_index(c: &mut Criterion) {
    let mut pp = PackedPage {
      ints: [0; PAGE_SIZE + (READ_SIZE-1)],
    };

    let size : u32 = 11;
    let mask : u64 = (1u64 << size) - 1u64;

    c.bench_function("put BI", move |b| b.iter(|| pp.put(black_box(20u32), 7, size, mask)));


    let pp = PackedPage {
      ints: [0; PAGE_SIZE + (READ_SIZE-1)],
    };

    c.bench_function("get BI", move |b| b.iter(|| pp.get(black_box(20u32), size, mask)));
}


#[path = "../src/bit_array.rs"] 
mod bit_array;


fn bench_bit_array(c: &mut Criterion) {
    let mut pp = bit_array::BitArray::new(40, 11);
    c.bench_function("put BA", move |b| b.iter(|| pp.put(black_box(20u32), 7)));

    let pp = bit_array::BitArray::new(40, 11);
    c.bench_function("get BA", move |b| b.iter(|| pp.get(black_box(20u32))));
}

criterion_group!(benches, bench_bit_index, bench_bit_array);


criterion_main!(benches);