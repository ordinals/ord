use {criterion::Criterion, ord::Index};

fn main() {
  let mut criterion = Criterion::default().configure_from_args();
  let index = Index::open(&Default::default()).unwrap();
  let mut i = 0;

  criterion.bench_function("inscription", |b| {
    b.iter(|| {
      Index::inscription_info_benchmark(&index, i);
      i += 1;
    });
  });

  Criterion::default().configure_from_args().final_summary();
}
