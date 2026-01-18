use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use space_partitioning::quadtree::{AABB, QuadRect, QuadTreeElement};
use space_partitioning::rtree::{BoundingBox, RTree};
use space_partitioning::QuadTree;

const SPACE_MIN: i32 = 0;
const SPACE_MAX: i32 = 1000;
const QUADTREE_DEPTH: u8 = 8;
const QUADTREE_BUCKET: u32 = 16;
const QUADTREE_MIN_CELL: u32 = 1;

#[derive(Clone, Copy, Debug)]
struct Rect2D {
    id: u32,
    center: [i32; 2],
    half_size: [i32; 2],
}

struct BenchData {
    rects: Vec<Rect2D>,
    queries: Vec<Rect2D>,
}

fn criterion_benchmark(c: &mut Criterion) {
    let sizes = [1_000usize, 10_000usize];
    let mut datasets = Vec::with_capacity(sizes.len());

    for (index, count) in sizes.iter().enumerate() {
        datasets.push(build_data(0x2d_c0_u64 + index as u64, *count));
    }

    let mut insert_group = c.benchmark_group("compare_2d_insert");
    for data in &datasets {
        bench_insert_rtree(&mut insert_group, data);
        bench_insert_quadtree(&mut insert_group, data);
    }
    insert_group.finish();

    let mut query_group = c.benchmark_group("compare_2d_query_intersects");
    for data in &datasets {
        bench_query_rtree(&mut query_group, data);
        bench_query_quadtree(&mut query_group, data);
    }
    query_group.finish();
}

fn bench_insert_rtree(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    group.bench_with_input(
        BenchmarkId::new("rtree", data.rects.len()),
        &data.rects,
        |b, rects| {
            b.iter_batched(
                || rects.clone(),
                |rects| {
                    let mut tree: RTree<f32, 2, 16, u32> = RTree::default();
                    for rect in rects {
                        tree.insert(rect.id, rect_to_rtree_bb(rect));
                    }
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn bench_insert_quadtree(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    let quad_bounds = QuadRect::new(SPACE_MIN, SPACE_MIN, SPACE_MAX, SPACE_MAX);
    group.bench_with_input(
        BenchmarkId::new("quadtree", data.rects.len()),
        &data.rects,
        |b, rects| {
            b.iter_batched(
                || rects.clone(),
                |rects| {
                    let mut tree = QuadTree::new(
                        quad_bounds,
                        QUADTREE_DEPTH,
                        QUADTREE_BUCKET,
                        QUADTREE_MIN_CELL,
                    );
                    for rect in rects {
                        let aabb = rect_to_quadtree_aabb(rect);
                        tree.insert(QuadTreeElement::new(rect.id, aabb))
                            .expect("insert should work");
                    }
                    black_box(tree);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

fn bench_query_rtree(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    let tree = build_rtree(data);
    group.bench_function(BenchmarkId::new("rtree", data.rects.len()), |b| {
        b.iter(|| {
            let mut hits = 0usize;
            for query in &data.queries {
                hits += tree.query_intersects(&rect_to_rtree_bb(*query)).len();
            }
            black_box(hits);
        });
    });
}

fn bench_query_quadtree(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &BenchData,
) {
    let tree = build_quadtree(data);
    group.bench_function(BenchmarkId::new("quadtree", data.rects.len()), |b| {
        b.iter(|| {
            let mut hits = 0usize;
            for query in &data.queries {
                let aabb = rect_to_quadtree_aabb(*query);
                hits += tree.intersect_aabb(&aabb).len();
            }
            black_box(hits);
        });
    });
}

fn build_rtree(data: &BenchData) -> RTree<f32, 2, 16, u32> {
    let mut tree: RTree<f32, 2, 16, u32> = RTree::default();
    for rect in &data.rects {
        tree.insert(rect.id, rect_to_rtree_bb(*rect));
    }
    tree
}

fn build_quadtree(data: &BenchData) -> QuadTree {
    let mut tree = QuadTree::new(
        QuadRect::new(SPACE_MIN, SPACE_MIN, SPACE_MAX, SPACE_MAX),
        QUADTREE_DEPTH,
        QUADTREE_BUCKET,
        QUADTREE_MIN_CELL,
    );
    for rect in &data.rects {
        let aabb = rect_to_quadtree_aabb(*rect);
        tree.insert(QuadTreeElement::new(rect.id, aabb))
            .expect("insert should work");
    }
    tree
}

fn build_data(seed: u64, count: usize) -> BenchData {
    let rects = build_rects(seed, count, 2..50);
    let queries = build_rects(seed ^ 0x77, 256, 25..120);

    BenchData { rects, queries }
}

fn build_rects(seed: u64, count: usize, size_range: std::ops::Range<i32>) -> Vec<Rect2D> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut rects = Vec::with_capacity(count);
    for id in 0..count {
        let half_x = rng.gen_range(size_range.clone());
        let half_y = rng.gen_range(size_range.clone());
        let center = [
            rng.gen_range(SPACE_MIN..SPACE_MAX),
            rng.gen_range(SPACE_MIN..SPACE_MAX),
        ];
        rects.push(Rect2D {
            id: id as u32,
            center,
            half_size: [half_x, half_y],
        });
    }
    rects
}

fn rect_to_rtree_bb(rect: Rect2D) -> BoundingBox<f32, 2> {
    let min_x = (rect.center[0] - rect.half_size[0]) as f32;
    let max_x = (rect.center[0] + rect.half_size[0]) as f32;
    let min_y = (rect.center[1] - rect.half_size[1]) as f32;
    let max_y = (rect.center[1] + rect.half_size[1]) as f32;
    BoundingBox::from([min_x..=max_x, min_y..=max_y])
}

fn rect_to_quadtree_aabb(rect: Rect2D) -> AABB {
    let min_x = clamp_i32(rect.center[0] - rect.half_size[0]);
    let max_x = clamp_i32(rect.center[0] + rect.half_size[0]);
    let min_y = clamp_i32(rect.center[1] - rect.half_size[1]);
    let max_y = clamp_i32(rect.center[1] + rect.half_size[1]);
    AABB::new(min_x, min_y, max_x, max_y)
}

fn clamp_i32(value: i32) -> i32 {
    value.clamp(SPACE_MIN, SPACE_MAX)
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
