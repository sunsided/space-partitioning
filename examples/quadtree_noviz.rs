use indicatif::{ProgressBar, ProgressStyle};
use rand::{thread_rng, Rng};
use space_partitioning::intersections::IntersectsWith;
use space_partitioning::quadtree::{QuadRect, QuadTreeElement, AABB};
use space_partitioning::QuadTree;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Duration;

const TREE_DEPTH: u8 = 6;
const MAX_NUM_ELEMENTS: u32 = 1;
const NUM_STATIC_ELEMENTS: u32 = 512;
const CURSOR_SIZE: f64 = 64.0;

struct Disk {
    pub id: u32,
    pub cx: f64,
    pub cy: f64,
    pub radius: f64,
}

#[derive(Debug, Default)]
struct Mouse {
    pub pos: [f64; 2],
}

fn main() {
    let mut rotation = 0.0;
    let mut window_size = [0.0; 2];
    let mut mouse = Mouse::default();

    let mut ray = Ray::new(0., -320., 0., -1.);

    let (mut tree, mut items) = build_test_data();
    let _ = tree.insert(mouse.build_qte(&window_size));

    let pb = ProgressBar::new(0);
    pb.set_message("Simulating");
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("[{spinner}] [{elapsed_precise} {per_sec:.cyan/blue}] {msg}"),
    );

    let dt = 0.016666666666666666;
    let dt_dur = Duration::from_millis(16);
    let mut previous = std::time::Instant::now();
    let mut cycles = 0;

    loop {
        cycles += 1;

        // Update
        let now = std::time::Instant::now();
        let delta = now - previous;
        if delta >= dt_dur {
            pb.inc(cycles);
            cycles = 0;

            previous = now;
            rotation += 3.0 * dt;

            // Need to remove the mouse from the tree before we change its AABB.
            tree.remove(&mouse.build_qte(&window_size));
            mouse.pos = [
                thread_rng().gen_range(-256..=256) as f64,
                thread_rng().gen_range(-256..=256) as f64,
            ];
            let _ = tree.insert(mouse.build_qte(&window_size));

            // Update the ray.
            let angle = ((rotation * 0.2f64).cos() * 90.).to_radians();
            ray = Ray::new(ray.x, ray.y, angle.sin() as _, angle.cos() as _);

            // Update the moving element.
            assert!(tree.remove(&QuadTreeElement::new(0, items[0].get_aabb())));
            items[0].cx = -rotation.cos() * 128.0;
            items[0].cy = rotation.sin() * 128.0;
            tree.insert(QuadTreeElement::new(0, items[0].get_aabb()))
                .expect("element was out of bounds");

            // Compact the tree.
            tree.cleanup();
        }

        // Run intersections as quickly as possible.
        let _ = intersect_with_mouse(&mut tree, &mut window_size, mouse.pos, CURSOR_SIZE);
        let _ = intersect_with_ray(&mut tree, &ray);
    }
}

fn intersect_with_mouse(
    tree: &QuadTree,
    window_size: &mut [f64; 2],
    pos: [f64; 2],
    cursor_size: f64,
) -> HashSet<u32> {
    let x = pos[0] - window_size[0] * 0.5 - cursor_size * 0.5;
    let y = pos[1] - window_size[1] * 0.5 - cursor_size * 0.5;

    let aabb = AABB::new(
        x.floor() as _,
        y.floor() as _,
        (x + cursor_size).ceil() as _,
        (y + cursor_size).ceil() as _,
    );

    let vec = tree.intersect_aabb(&aabb);
    let vec_len = vec.len();
    let set = HashSet::from_iter(vec.into_iter());
    debug_assert_eq!(vec_len, set.len());
    set
}

fn intersect_with_ray(tree: &QuadTree, ray: &Ray) -> HashSet<u32> {
    let vec = tree.intersect_generic(ray);
    let vec_len = vec.len();
    let set = HashSet::from_iter(vec.into_iter());
    debug_assert_eq!(vec_len, set.len());
    set
}

fn build_test_data() -> (QuadTree, Vec<Disk>) {
    let mut tree = QuadTree::new(
        QuadRect::new(-256, -256, 512, 512),
        TREE_DEPTH,
        MAX_NUM_ELEMENTS,
        1,
    );
    let mut items = Vec::new();

    // Build a moving element.
    let item = Disk {
        id: 0,
        cx: -128.0,
        cy: 0.0,
        radius: 32.0,
    };
    tree.insert(QuadTreeElement::new(item.id, item.get_aabb()))
        .expect("insert failed");
    debug_assert_eq!(items.len(), item.id as usize);
    items.push(item);

    // Build some static elements.
    for i in 0..NUM_STATIC_ELEMENTS {
        let mut rng = rand::thread_rng();
        let large = rng.gen::<f64>() <= 0.05;
        let size = if large { 24. } else { 8. };

        let item = Disk {
            id: (i + 1) as _,
            cx: rng.gen_range(-256.0..256.0),
            cy: rng.gen_range(-256.0..256.0),
            radius: rng.gen_range(2.0..size),
        };

        tree.insert(QuadTreeElement::new(item.id, item.get_aabb()))
            .expect("insert failed");
        debug_assert_eq!(items.len(), item.id as usize);
        items.push(item);
    }
    (tree, items)
}

impl Disk {
    fn get_aabb(&self) -> AABB {
        AABB::new(
            (self.cx - self.radius).floor() as _,
            (self.cy - self.radius).floor() as _,
            (self.cx + self.radius).ceil() as _,
            (self.cy + self.radius).ceil() as _,
        )
    }
}

impl Mouse {
    fn build_qte(&self, window_size: &[f64; 2]) -> QuadTreeElement {
        QuadTreeElement::new(
            1337,
            AABB::new(
                (self.pos[0] - window_size[0] * 0.5 - CURSOR_SIZE * 0.5).floor() as _,
                (self.pos[1] - window_size[0] * 0.5 - CURSOR_SIZE * 0.5).floor() as _,
                (self.pos[0] - window_size[0] * 0.5 + CURSOR_SIZE * 0.5).ceil() as _,
                (self.pos[1] - window_size[0] * 0.5 + CURSOR_SIZE * 0.5).ceil() as _,
            ),
        )
    }
}

#[derive(Debug)]
struct Ray {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    inv_dx: f32,
    inv_dy: f32,
}

impl Ray {
    fn new(x: f32, y: f32, dx: f32, dy: f32) -> Ray {
        Ray {
            x,
            y,
            dx,
            dy,
            inv_dx: 1.0 / dx,
            inv_dy: 1.0 / dy,
        }
    }
}

impl IntersectsWith<AABB> for Ray {
    fn intersects_with(&self, other: &AABB) -> bool {
        // https://gamedev.stackexchange.com/a/18459/10433

        let t1 = (other.tl.x as f32 - self.x) * self.inv_dx;
        let t2 = (other.br.x as f32 - self.x) * self.inv_dx;
        let t3 = (other.br.y as f32 - self.y) * self.inv_dy;
        let t4 = (other.tl.y as f32 - self.y) * self.inv_dy;

        let tmin = t1.min(t2).max(t3.min(t4));
        let tmax = t1.max(t2).min(t3.max(t4));

        // if tmax < 0, ray (line) is intersecting AABB, but the whole AABB is behind us
        // if tmin > tmax, ray doesn't intersect AABB
        if (tmax < 0.) | (tmin > tmax) {
            return false;
        }

        return true;
    }
}
