use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::ThreadRng;
use rand::{thread_rng, Rng};
use space_partitioning::quadtree::{QuadRect, QuadTreeElement, AABB};
use space_partitioning::QuadTree;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("insert tree(w=256, h=256, depth=4)", |b| {
        let mut tree = QuadTree::new(QuadRect::new(0, 0, 256, 256), 4);
        let mut rng = thread_rng();
        let mut id = 0;

        b.iter(|| {
            let aabb = random_aabb(&mut rng, 1..256, 1..256, 1..32, 1..32);
            tree.insert(QuadTreeElement::new(id, aabb))
                .expect("insert should work");
            id += 1;
        })
    });

    c.bench_function("insert tree(w=256, h=256, depth=8)", |b| {
        let mut tree = QuadTree::new(QuadRect::new(0, 0, 256, 256), 8);
        let mut rng = thread_rng();
        let mut id = 0;

        b.iter(|| {
            let aabb = random_aabb(&mut rng, 1..256, 1..256, 1..32, 1..32);
            tree.insert(QuadTreeElement::new(id, aabb))
                .expect("insert should work");
            id += 1;
        })
    });

    c.bench_function("intersect_aabb tree(n=1024, w=256, h=256, depth=4)", |b| {
        let mut rng = thread_rng();
        let tree = build_random_tree(&mut rng, 1024, 256, 256, 4);
        b.iter(|| {
            let aabb = random_aabb(&mut rng, 1..256, 1..256, 1..64, 1..64);
            tree.intersect_aabb(&aabb)
        })
    });

    c.bench_function(
        "intersect_generic tree(n=1024, w=256, h=256, depth=4)",
        |b| {
            let mut rng = thread_rng();
            let tree = build_random_tree(&mut rng, 1024, 256, 256, 4);
            b.iter(|| {
                let aabb = random_aabb(&mut rng, 1..256, 1..256, 1..64, 1..64);
                tree.intersect_generic(&aabb)
            })
        },
    );

    c.bench_function(
        "intersect_generic tree(n=1024, w=256, h=256, depth=8)",
        |b| {
            let mut rng = thread_rng();
            let tree = build_random_tree(&mut rng, 1024, 256, 256, 8);
            b.iter(|| {
                let aabb = random_aabb(&mut rng, 1..256, 1..256, 1..64, 1..64);
                tree.intersect_generic(&aabb)
            })
        },
    );
}

fn build_random_tree(
    mut rng: &mut ThreadRng,
    num_elements: u32,
    width: i32,
    height: i32,
    depth: u32,
) -> QuadTree {
    let mut tree = QuadTree::new(QuadRect::new(0, 0, width, height), depth);
    for id in 0..num_elements {
        let aabb = random_aabb(&mut rng, 1..256, 1..256, 1..32, 1..32);
        tree.insert(QuadTreeElement::new(id, aabb))
            .expect("insert should work");
    }
    tree
}

#[inline]
fn random_aabb(
    rng: &mut ThreadRng,
    x: std::ops::Range<i32>,
    y: std::ops::Range<i32>,
    w: std::ops::Range<i32>,
    h: std::ops::Range<i32>,
) -> AABB {
    let x = rng.gen_range(x);
    let y = rng.gen_range(y);
    let hx = rng.gen_range(w) >> 1;
    let hy = rng.gen_range(h) >> 1;
    AABB::new(x - hx, y - hy, x + hx, y + hy)
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
