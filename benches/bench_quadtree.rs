use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};
use space_partitioning::quadtree::{QuadRect, QuadTreeElement, AABB};
use space_partitioning::QuadTree;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("insert w=256,h=256,depth=4", |b| {
        let mut tree = QuadTree::new(QuadRect::new(0, 0, 256, 256), 4);
        let mut rng = thread_rng();
        let mut id = 0;

        b.iter(|| {
            let x = rng.gen_range(1..256);
            let y = rng.gen_range(1..256);
            let hx = rng.gen_range(1..32) >> 1;
            let hy = rng.gen_range(1..32) >> 1;
            tree.insert(QuadTreeElement::new(
                id,
                AABB::new(x - hx, y - hy, x + hx, y + hy),
            ))
            .expect("insert should work");

            id += 1;
        })
    });

    c.bench_function("insert w=256,h=256,depth=8", |b| {
        let mut tree = QuadTree::new(QuadRect::new(0, 0, 256, 256), 8);
        let mut rng = thread_rng();
        let mut id = 0;

        b.iter(|| {
            let x = rng.gen_range(1..256);
            let y = rng.gen_range(1..256);
            let hx = rng.gen_range(1..32) >> 1;
            let hy = rng.gen_range(1..32) >> 1;
            tree.insert(QuadTreeElement::new(
                id,
                AABB::new(x - hx, y - hy, x + hx, y + hy),
            ))
            .expect("insert should work");

            id += 1;
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
