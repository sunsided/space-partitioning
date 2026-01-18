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
const Z_WEIGHT: f32 = 0.75;

#[derive(Clone, Copy, Debug)]
struct Sphere {
    id: u32,
    center: [i32; 3],
    radius: i32,
}

struct BenchData {
    spheres: Vec<Sphere>,
    queries: Vec<Sphere>,
    z_range: (i32, i32),
}

fn criterion_benchmark(c: &mut Criterion) {
    let sizes = [1_000usize, 10_000usize];
    let mut datasets = Vec::with_capacity(sizes.len());

    for (index, count) in sizes.iter().enumerate() {
        datasets.push(build_data(0x3d_c0_u64 + index as u64, *count));
    }

    let mut insert_group = c.benchmark_group("compare_3d_insert");
    for data in &datasets {
        bench_insert_rtree(&mut insert_group, data);
        bench_insert_quadtree(&mut insert_group, data);
    }
    insert_group.finish();

    let mut query_group = c.benchmark_group("compare_3d_query_intersects");
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
        BenchmarkId::new("rtree", data.spheres.len()),
        &data.spheres,
        |b, spheres| {
            b.iter_batched(
                || spheres.clone(),
                |spheres| {
                    let mut tree: RTree<f32, 3, 16, u32> = RTree::default();
                    for sphere in spheres {
                        tree.insert(sphere.id, sphere_to_rtree_bb(sphere));
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
        BenchmarkId::new("quadtree", data.spheres.len()),
        &data.spheres,
        |b, spheres| {
            b.iter_batched(
                || spheres.clone(),
                |spheres| {
                    let mut tree = QuadTree::new(
                        quad_bounds,
                        QUADTREE_DEPTH,
                        QUADTREE_BUCKET,
                        QUADTREE_MIN_CELL,
                    );
                    for sphere in spheres {
                        let aabb = sphere_to_quadtree_aabb(sphere, data.z_range);
                        tree.insert(QuadTreeElement::new(sphere.id, aabb))
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
    group.bench_function(BenchmarkId::new("rtree", data.spheres.len()), |b| {
        b.iter(|| {
            let mut hits = 0usize;
            for query in &data.queries {
                hits += tree.query_intersects(&sphere_to_rtree_bb(*query)).len();
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
    group.bench_function(BenchmarkId::new("quadtree", data.spheres.len()), |b| {
        b.iter(|| {
            let mut hits = 0usize;
            for query in &data.queries {
                let aabb = sphere_to_quadtree_aabb(*query, data.z_range);
                hits += tree.intersect_aabb(&aabb).len();
            }
            black_box(hits);
        });
    });
}

fn build_rtree(data: &BenchData) -> RTree<f32, 3, 16, u32> {
    let mut tree: RTree<f32, 3, 16, u32> = RTree::default();
    for sphere in &data.spheres {
        tree.insert(sphere.id, sphere_to_rtree_bb(*sphere));
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
    for sphere in &data.spheres {
        let aabb = sphere_to_quadtree_aabb(*sphere, data.z_range);
        tree.insert(QuadTreeElement::new(sphere.id, aabb))
            .expect("insert should work");
    }
    tree
}

fn build_data(seed: u64, count: usize) -> BenchData {
    let spheres = build_spheres(seed, count, 2..50);
    let z_range = z_range(&spheres);
    let queries = build_spheres(seed ^ 0x99, 256, 25..100);

    BenchData {
        spheres,
        queries,
        z_range,
    }
}

fn build_spheres(seed: u64, count: usize, radius_range: std::ops::Range<i32>) -> Vec<Sphere> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut spheres = Vec::with_capacity(count);
    for id in 0..count {
        let radius = rng.gen_range(radius_range.clone());
        let center = [
            rng.gen_range(SPACE_MIN..SPACE_MAX),
            rng.gen_range(SPACE_MIN..SPACE_MAX),
            rng.gen_range(SPACE_MIN..SPACE_MAX),
        ];
        spheres.push(Sphere {
            id: id as u32,
            center,
            radius,
        });
    }
    spheres
}

fn z_range(spheres: &[Sphere]) -> (i32, i32) {
    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;
    for sphere in spheres {
        min_z = min_z.min(sphere.center[2]);
        max_z = max_z.max(sphere.center[2]);
    }
    (min_z, max_z)
}

fn sphere_to_rtree_bb(sphere: Sphere) -> BoundingBox<f32, 3> {
    let r = sphere.radius as f32;
    let cx = sphere.center[0] as f32;
    let cy = sphere.center[1] as f32;
    let cz = sphere.center[2] as f32;

    BoundingBox::from([
        (cx - r)..=(cx + r),
        (cy - r)..=(cy + r),
        (cz - r)..=(cz + r),
    ])
}

fn sphere_to_quadtree_aabb(sphere: Sphere, z_range: (i32, i32)) -> AABB {
    let z_norm = normalize_z(sphere.center[2], z_range);
    let radius_scaled = sphere.radius as f32 * (1.0 + z_norm * Z_WEIGHT);
    let radius_scaled = radius_scaled.round() as i32;

    let min_x = clamp_i32(sphere.center[0] - radius_scaled);
    let max_x = clamp_i32(sphere.center[0] + radius_scaled);
    let min_y = clamp_i32(sphere.center[1] - radius_scaled);
    let max_y = clamp_i32(sphere.center[1] + radius_scaled);

    AABB::new(min_x, min_y, max_x, max_y)
}

fn normalize_z(z: i32, z_range: (i32, i32)) -> f32 {
    let (min_z, max_z) = z_range;
    if min_z == max_z {
        return 0.0;
    }
    (z - min_z) as f32 / (max_z - min_z) as f32
}

fn clamp_i32(value: i32) -> i32 {
    value.clamp(SPACE_MIN, SPACE_MAX)
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
