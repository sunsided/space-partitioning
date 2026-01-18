use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use space_partitioning::rtree::{BoundingBox, RTree, RTreeEntry};
use std::ops::Range;

type Entry = (u32, BoundingBox<f32, 2>);

struct BenchData {
    entries: Vec<Entry>,
    queries_small: Vec<BoundingBox<f32, 2>>,
    queries_large: Vec<BoundingBox<f32, 2>>,
    points: Vec<[f32; 2]>,
}

fn criterion_benchmark(c: &mut Criterion) {
    let sizes = [1_000usize, 10_000usize];
    let mut datasets = Vec::with_capacity(sizes.len());

    for (index, count) in sizes.iter().enumerate() {
        datasets.push(build_bench_data(0x5eeda11u64 + index as u64, *count));
    }

    let mut insert_group = c.benchmark_group("rtree_insert");
    for data in &datasets {
        bench_insert_m8(&mut insert_group, data);
        bench_insert_m16(&mut insert_group, data);
    }
    insert_group.finish();

    let mut bulk_group = c.benchmark_group("rtree_bulk_load");
    for data in &datasets {
        bench_bulk_load_m8(&mut bulk_group, data);
        bench_bulk_load_m16(&mut bulk_group, data);
        bench_bulk_load_entries_m8(&mut bulk_group, data);
        bench_bulk_load_entries_m16(&mut bulk_group, data);
    }
    bulk_group.finish();

    let mut query_group = c.benchmark_group("rtree_query_intersects");
    for data in &datasets {
        bench_query_intersects_m8(&mut query_group, data, "small", &data.queries_small);
        bench_query_intersects_m8(&mut query_group, data, "large", &data.queries_large);
        bench_query_intersects_m16(&mut query_group, data, "small", &data.queries_small);
        bench_query_intersects_m16(&mut query_group, data, "large", &data.queries_large);
    }
    query_group.finish();

    let mut contains_group = c.benchmark_group("rtree_query_contains");
    for data in &datasets {
        bench_query_contains_m8(&mut contains_group, data, "small", &data.queries_small);
        bench_query_contains_m8(&mut contains_group, data, "large", &data.queries_large);
        bench_query_contains_m16(&mut contains_group, data, "small", &data.queries_small);
        bench_query_contains_m16(&mut contains_group, data, "large", &data.queries_large);
    }
    contains_group.finish();

    let mut nn_group = c.benchmark_group("rtree_nearest_neighbor");
    for data in &datasets {
        bench_nearest_neighbor_m8(&mut nn_group, data);
        bench_nearest_neighbor_m16(&mut nn_group, data);
    }
    nn_group.finish();

    let mut remove_group = c.benchmark_group("rtree_remove");
    for data in &datasets {
        bench_remove_m8(&mut remove_group, data);
        bench_remove_m16(&mut remove_group, data);
    }
    remove_group.finish();
}

fn bench_insert_m8(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    group.bench_with_input(
        BenchmarkId::new("M8", data.entries.len()),
        &data.entries,
        |b, entries| {
            b.iter_batched(
                || entries.clone(),
                |entries| {
                    let mut tree: RTree<f32, 2, 8, u32> = RTree::default();
                    for (id, bb) in entries {
                        tree.insert(id, bb);
                    }
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn bench_insert_m16(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    group.bench_with_input(
        BenchmarkId::new("M16", data.entries.len()),
        &data.entries,
        |b, entries| {
            b.iter_batched(
                || entries.clone(),
                |entries| {
                    let mut tree: RTree<f32, 2, 16, u32> = RTree::default();
                    for (id, bb) in entries {
                        tree.insert(id, bb);
                    }
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn bench_bulk_load_m8(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    group.bench_with_input(
        BenchmarkId::new("M8", data.entries.len()),
        &data.entries,
        |b, entries| {
            b.iter_batched(
                || entries.clone(),
                |entries| {
                    let tree: RTree<f32, 2, 8, u32> = RTree::bulk_load(entries);
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn bench_bulk_load_m16(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    group.bench_with_input(
        BenchmarkId::new("M16", data.entries.len()),
        &data.entries,
        |b, entries| {
            b.iter_batched(
                || entries.clone(),
                |entries| {
                    let tree: RTree<f32, 2, 16, u32> = RTree::bulk_load(entries);
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn bench_bulk_load_entries_m8(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    group.bench_with_input(
        BenchmarkId::new("M8_entries", data.entries.len()),
        &data.entries,
        |b, entries| {
            b.iter_batched(
                || build_entry_records(entries),
                |entries| {
                    let tree: RTree<f32, 2, 8, u32> = RTree::bulk_load_entries(entries);
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn bench_bulk_load_entries_m16(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    group.bench_with_input(
        BenchmarkId::new("M16_entries", data.entries.len()),
        &data.entries,
        |b, entries| {
            b.iter_batched(
                || build_entry_records(entries),
                |entries| {
                    let tree: RTree<f32, 2, 16, u32> = RTree::bulk_load_entries(entries);
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn bench_query_intersects_m8(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
    label: &str,
    queries: &[BoundingBox<f32, 2>],
) {
    let tree = build_tree_m8(&data.entries);
    group.bench_function(
        BenchmarkId::new(format!("M8_{label}"), data.entries.len()),
        |b| {
            b.iter(|| {
                let mut hits = 0usize;
                for query in queries {
                    hits += tree.query_intersects(query).len();
                }
                black_box(hits);
            });
        },
    );
}

fn bench_query_intersects_m16(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
    label: &str,
    queries: &[BoundingBox<f32, 2>],
) {
    let tree = build_tree_m16(&data.entries);
    group.bench_function(
        BenchmarkId::new(format!("M16_{label}"), data.entries.len()),
        |b| {
            b.iter(|| {
                let mut hits = 0usize;
                for query in queries {
                    hits += tree.query_intersects(query).len();
                }
                black_box(hits);
            });
        },
    );
}

fn bench_query_contains_m8(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
    label: &str,
    queries: &[BoundingBox<f32, 2>],
) {
    let tree = build_tree_m8(&data.entries);
    group.bench_function(
        BenchmarkId::new(format!("M8_{label}"), data.entries.len()),
        |b| {
            b.iter(|| {
                let mut hits = 0usize;
                for query in queries {
                    hits += tree.query_contains(query).len();
                }
                black_box(hits);
            });
        },
    );
}

fn bench_query_contains_m16(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
    label: &str,
    queries: &[BoundingBox<f32, 2>],
) {
    let tree = build_tree_m16(&data.entries);
    group.bench_function(
        BenchmarkId::new(format!("M16_{label}"), data.entries.len()),
        |b| {
            b.iter(|| {
                let mut hits = 0usize;
                for query in queries {
                    hits += tree.query_contains(query).len();
                }
                black_box(hits);
            });
        },
    );
}

fn bench_nearest_neighbor_m8(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    let tree = build_tree_m8(&data.entries);
    group.bench_function(BenchmarkId::new("M8", data.entries.len()), |b| {
        b.iter(|| {
            let mut hits = 0usize;
            for point in &data.points {
                if tree.nearest_neighbor(*point).is_some() {
                    hits += 1;
                }
            }
            black_box(hits);
        });
    });
}

fn bench_nearest_neighbor_m16(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    let tree = build_tree_m16(&data.entries);
    group.bench_function(BenchmarkId::new("M16", data.entries.len()), |b| {
        b.iter(|| {
            let mut hits = 0usize;
            for point in &data.points {
                if tree.nearest_neighbor(*point).is_some() {
                    hits += 1;
                }
            }
            black_box(hits);
        });
    });
}

fn bench_remove_m8(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    group.bench_with_input(
        BenchmarkId::new("M8", data.entries.len()),
        &data.entries,
        |b, entries| {
            b.iter_batched(
                || build_tree_m8(entries),
                |mut tree| {
                    for (id, bb) in entries {
                        tree.remove(id, bb);
                    }
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn bench_remove_m16(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    group.bench_with_input(
        BenchmarkId::new("M16", data.entries.len()),
        &data.entries,
        |b, entries| {
            b.iter_batched(
                || build_tree_m16(entries),
                |mut tree| {
                    for (id, bb) in entries {
                        tree.remove(id, bb);
                    }
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn build_tree_m8(entries: &[Entry]) -> RTree<f32, 2, 8, u32> {
    let mut tree: RTree<f32, 2, 8, u32> = RTree::default();
    for (id, bb) in entries {
        tree.insert(*id, bb.clone());
    }
    tree
}

fn build_tree_m16(entries: &[Entry]) -> RTree<f32, 2, 16, u32> {
    let mut tree: RTree<f32, 2, 16, u32> = RTree::default();
    for (id, bb) in entries {
        tree.insert(*id, bb.clone());
    }
    tree
}

fn build_bench_data(seed: u64, count: usize) -> BenchData {
    let space = 0.0..1000.0;
    let entry_size = 1.0..50.0;
    let query_small = 5.0..25.0;
    let query_large = 75.0..250.0;

    let entries = build_entries(seed, count, &space, &entry_size);
    let queries_small = build_queries(seed ^ 0x1234, 256, &space, &query_small);
    let queries_large = build_queries(seed ^ 0x9abc, 256, &space, &query_large);
    let points = build_points(seed ^ 0x55aa, 256, &space);

    BenchData {
        entries,
        queries_small,
        queries_large,
        points,
    }
}

fn build_entries(seed: u64, count: usize, space: &Range<f32>, size: &Range<f32>) -> Vec<Entry> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut entries = Vec::with_capacity(count);
    for id in 0..count {
        let bb = random_bb(&mut rng, space, size);
        entries.push((id as u32, bb));
    }
    entries
}

fn build_queries(
    seed: u64,
    count: usize,
    space: &Range<f32>,
    size: &Range<f32>,
) -> Vec<BoundingBox<f32, 2>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut queries = Vec::with_capacity(count);
    for _ in 0..count {
        queries.push(random_bb(&mut rng, space, size));
    }
    queries
}

fn build_points(seed: u64, count: usize, space: &Range<f32>) -> Vec<[f32; 2]> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut points = Vec::with_capacity(count);
    for _ in 0..count {
        points.push([
            rng.gen_range(space.start..space.end),
            rng.gen_range(space.start..space.end),
        ]);
    }
    points
}

fn random_bb(rng: &mut StdRng, space: &Range<f32>, size: &Range<f32>) -> BoundingBox<f32, 2> {
    let cx = rng.gen_range(space.start..space.end);
    let cy = rng.gen_range(space.start..space.end);
    let sx = rng.gen_range(size.start..size.end);
    let sy = rng.gen_range(size.start..size.end);
    let hx = sx * 0.5;
    let hy = sy * 0.5;

    let min_x = cx - hx;
    let max_x = cx + hx;
    let min_y = cy - hy;
    let max_y = cy + hy;

    BoundingBox::from([min_x..=max_x, min_y..=max_y])
}

fn build_entry_records(entries: &[Entry]) -> Vec<RTreeEntry<f32, 2, u32>> {
    entries
        .iter()
        .map(|(id, bb)| RTreeEntry::new(*id, bb.clone()))
        .collect()
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
