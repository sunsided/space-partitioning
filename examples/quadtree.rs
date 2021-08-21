use indicatif::{ProgressBar, ProgressStyle};
use piston_window::*;
use rand::Rng;
use space_partitioning::intersections::IntersectsWith;
use space_partitioning::quadtree::{NodeInfo, QuadRect, QuadTreeElement, AABB};
use space_partitioning::types::HashSet;
use space_partitioning::QuadTree;
use std::iter::FromIterator;

const TREE_DEPTH: u8 = 8;
const MAX_NUM_ELEMENTS: u32 = 1;
const NUM_STATIC_ELEMENTS: u32 = 256;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

const DISK: [f32; 4] = [0.0, 1.0, 0.0, 0.5];
const DISK_WITH_MOUSE_HIT: [f32; 4] = [1.0, 1.0, 0.0, 0.5];
const DISK_WITH_RAY_HIT: [f32; 4] = [0.2, 0.0, 1.0, 0.5];
const DISK_WITH_MANY_HITS: [f32; 4] = [1.0, 0.0, 0.0, 0.5];
const MOUSE: [f32; 4] = [0.8, 0.8, 1.0, 0.25];
const RAY: [f32; 4] = [1.0, 0.5, 0.0, 1.0];

const QUAD_CELL_BORDER: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const QUAD_CELL_FULL: [f32; 4] = [0.25, 0.25, 0.25, 0.25];
const QUAD_CELL_EMPTY: [f32; 4] = [0.25, 0.25, 0.25, 0.05];

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
    let opengl = OpenGL::V4_5;
    let mut window: PistonWindow = WindowSettings::new("QuadTree", [800, 800])
        .exit_on_esc(true)
        .vsync(false)
        .graphics_api(opengl)
        .build()
        .unwrap();

    window.set_ups(120);
    window.set_swap_buffers(true);

    let ellipse = Ellipse::new(BLACK).resolution(16).border(
        Ellipse::new_border(BLACK, 1.0)
            .resolution(16)
            .border
            .unwrap(),
    );

    let rectangle =
        Rectangle::new(BLACK).border(Rectangle::new_border(QUAD_CELL_BORDER, 1.0).border.unwrap());

    let mut rotation = 0.0;
    let mut window_size = [0.0; 2];
    let mut items_under_mouse = HashSet::default();
    let mut items_under_ray = HashSet::default();
    let mut mouse = Mouse::default();

    let mut ray = Ray::new(0., -320., 0., -1.);

    let (mut tree, mut items) = build_test_data();
    let _ = tree.insert(mouse.build_qte(&window_size));
    let mut tree_structure = collect_tree_structure(&tree);

    let pb = ProgressBar::new_spinner();
    pb.set_draw_rate(5);
    pb.set_message("Simulating");
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("[{spinner}] [{elapsed_precise} {per_sec:.cyan/blue}] {msg}"),
    );

    while let Some(e) = window.next() {
        if let Some(args) = e.button_args() {
            if args.state == ButtonState::Press {
                match args.button {
                    Button::Keyboard(Key::R) => {
                        // Clear intersections.
                        items_under_mouse.clear();
                        items_under_ray.clear();

                        // Generate new data points
                        let (new_tree, new_items) = build_test_data();
                        let _ = tree.insert(mouse.build_qte(&window_size));
                        tree_structure = collect_tree_structure(&tree);
                        tree = new_tree;
                        items = new_items;
                    }
                    _ => {}
                }
            }
        }

        if let Some(pos) = e.mouse_cursor_args() {
            // Need to remove the mouse from the tree before we change its AABB.
            tree.remove(&mouse.build_qte(&window_size));
            mouse.pos = pos;
            let _ = tree.insert(mouse.build_qte(&window_size));
        }

        if let Some(args) = e.resize_args() {
            // Need to remove the mouse from the tree before we change its AABB.
            tree.remove(&mouse.build_qte(&window_size));
            window_size = args.window_size;
            let _ = tree.insert(mouse.build_qte(&window_size));
        }

        if let Some(args) = e.update_args() {
            pb.inc(1);
            rotation += 3.0 * args.dt;

            // Update the ray.
            let angle = ((rotation * 0.2).cos() * 90.).deg_to_rad();
            ray = Ray::new(ray.x, ray.y, angle.sin() as _, angle.cos() as _);

            // Update the moving element.
            assert!(tree.remove(&QuadTreeElement::new(0, items[0].get_aabb())));
            items[0].cx = -rotation.cos() * 128.0;
            items[0].cy = rotation.sin() * 128.0;
            tree.insert(QuadTreeElement::new(0, items[0].get_aabb()))
                .expect("element was out of bounds");

            // Compact the tree.
            tree.cleanup();

            // Update the visualization.
            tree_structure = collect_tree_structure(&tree);

            // Get new intersections.
            items_under_mouse =
                intersect_with_mouse(&mut tree, &mut window_size, mouse.pos, CURSOR_SIZE);
            items_under_ray = intersect_with_ray(&mut tree, &ray);
        }

        window.draw_2d(&e, |c, g, _| {
            clear([1.0; 4], g);

            let half_window_size = [window_size[0] * 0.5, window_size[1] * 0.5];

            // Render the tree and its elements.
            {
                let c = c.trans(half_window_size[0], half_window_size[1]);
                render_tree_nodes(&tree_structure, g, &c, &rectangle);
                render_disks(
                    &items,
                    g,
                    &c,
                    &items_under_mouse,
                    &items_under_ray,
                    &ellipse,
                );
            }

            // Render the mouse cursor.
            {
                let rect = [
                    mouse.pos[0] - CURSOR_SIZE * 0.5,
                    mouse.pos[1] - CURSOR_SIZE * 0.5,
                    CURSOR_SIZE,
                    CURSOR_SIZE,
                ];
                rectangle
                    .color(MOUSE)
                    .draw(rect, &c.draw_state, c.transform, g);
            }

            // Render the ray.
            {
                let c = c.trans(half_window_size[0], half_window_size[1]);
                render_ray(&ray, g, c);
            }
        });
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

    HashSet::from_iter(tree.intersect_aabb(&aabb).into_iter())
}

fn intersect_with_ray(tree: &QuadTree, ray: &Ray) -> HashSet<u32> {
    HashSet::from_iter(tree.intersect_generic(ray).into_iter())
}

fn render_disks(
    items: &Vec<Disk>,
    g: &mut G2d,
    c: &Context,
    mouse_matches: &HashSet<u32>,
    ray_matches: &HashSet<u32>,
    ellipse: &Ellipse,
) {
    for disk in items.iter() {
        let rect = [
            disk.cx - disk.radius,
            disk.cy - disk.radius,
            2. * disk.radius,
            2. * disk.radius,
        ];

        let mouse_hit = mouse_matches.contains(&disk.id);
        let ray_hit = ray_matches.contains(&disk.id);

        let color = if mouse_hit & ray_hit {
            DISK_WITH_MANY_HITS
        } else if mouse_hit {
            DISK_WITH_MOUSE_HIT
        } else if ray_hit {
            DISK_WITH_RAY_HIT
        } else {
            DISK
        };

        ellipse
            .color(color)
            .draw(rect, &c.draw_state, c.transform, g);
    }
}

fn render_tree_nodes(
    tree_rects: &Vec<([f64; 4], [f32; 4])>,
    g: &mut G2d,
    c: &Context,
    rectangle: &Rectangle,
) {
    for (rect, color) in tree_rects.iter() {
        rectangle
            .color(*color)
            .draw(*rect, &c.draw_state, c.transform, g);
    }
}

fn collect_tree_structure(tree: &QuadTree) -> Vec<([f64; 4], [f32; 4])> {
    let mut rects = Vec::with_capacity(64);
    tree.visit_leaves(|node| {
        let aabb: AABB = node.get_aabb();
        let rect = [
            aabb.tl.x as f64,
            aabb.tl.y as f64,
            (aabb.br.x - aabb.tl.x) as f64,
            (aabb.br.y - aabb.tl.y) as f64,
        ];
        let color = node_color(&node);
        rects.push((rect, color));
    });
    rects
}

fn render_ray(ray: &Ray, g: &mut G2d, c: Context) {
    line(
        RAY,
        1.0,
        [
            ray.x as f64,
            ray.y as _,
            (ray.x + ray.dx * 1024.0) as _,
            (ray.y + ray.dy * 1024.0) as _,
        ],
        c.transform,
        g,
    );
}

fn node_color(node: &NodeInfo) -> [f32; 4] {
    if node.element_count > 0 {
        QUAD_CELL_FULL
    } else {
        QUAD_CELL_EMPTY
    }
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
