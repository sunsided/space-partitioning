use piston_window::*;
use rand::Rng;
use space_partitioning::quadtree::{NodeInfo, QuadRect, QuadTreeElement, AABB};
use space_partitioning::QuadTree;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;

const TREE_DEPTH: u32 = 6;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0; 4];
const DISK: [f32; 4] = [0.0, 1.0, 0.0, 0.5];
const DISK_WITH_HIT: [f32; 4] = [1.0, 1.0, 0.0, 0.5];
const MOUSE: [f32; 4] = [0.8, 0.8, 1.0, 0.25];

const QUAD_CELL_BORDER: [f32; 4] = [0.0, 0.0, 0.0, 0.75];
const QUAD_CELL_FULL: [f32; 4] = [0.25, 0.25, 0.25, 0.25];
const QUAD_CELL_EMPTY: [f32; 4] = [0.25, 0.25, 0.25, 0.125];

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
        .vsync(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let mut rotation = 0.0;
    let mut window_size = [0.0; 2];
    let mut items_under_mouse = HashSet::default();
    let mut mouse = Mouse::default();

    let (mut tree, mut items) = build_test_data();
    tree.insert(mouse.build_qte(&window_size));

    while let Some(e) = window.next() {
        if let Some(args) = e.button_args() {
            if args.state == ButtonState::Press {
                match args.button {
                    Button::Keyboard(Key::R) => {
                        // Clear intersections.
                        items_under_mouse.clear();

                        // Generate new data points
                        let (new_tree, new_items) = build_test_data();
                        tree.insert(mouse.build_qte(&window_size));
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
            tree.insert(mouse.build_qte(&window_size));
        }

        if let Some(args) = e.resize_args() {
            // Need to remove the mouse from the tree before we change its AABB.
            tree.remove(&mouse.build_qte(&window_size));
            window_size = args.window_size;
            tree.insert(mouse.build_qte(&window_size));
        }

        if let Some(args) = e.update_args() {
            rotation += 3.0 * args.dt;

            // Update the moving element.
            assert!(tree.remove(&QuadTreeElement::new(0, items[0].get_aabb())));
            items[0].cx = -rotation.cos() * 128.0;
            items[0].cy = rotation.sin() * 128.0;
            tree.insert(QuadTreeElement::new(0, items[0].get_aabb()))
                .expect("element was out of bounds");

            // Compact the tree.
            tree.cleanup();

            // Get new intersections.
            items_under_mouse =
                intersect_with_mouse(&mut tree, &mut window_size, mouse.pos, CURSOR_SIZE);
        }

        if let Some(args) = e.render_args() {
            window.draw_2d(&e, |c, g, _| {
                clear([1.0; 4], g);

                {
                    // Move to window center.
                    let mut half_window_size =
                        [args.window_size[0] * 0.5, args.window_size[1] * 0.5];
                    let c = c.trans(half_window_size[0], half_window_size[1]);

                    render_tree_nodes(&tree, g, &c);
                    render_disks(&items, g, &c, &items_under_mouse);
                }

                // Render the mouse cursor.
                let rect = [
                    mouse.pos[0] - CURSOR_SIZE * 0.5,
                    mouse.pos[1] - CURSOR_SIZE * 0.5,
                    CURSOR_SIZE,
                    CURSOR_SIZE,
                ];
                rectangle(MOUSE, rect, c.transform, g);
                Rectangle::new_border(BLACK, 1.0).draw(rect, &c.draw_state, c.transform, g);
            });
        }
    }
}

fn intersect_with_mouse(
    tree: &mut QuadTree,
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

    tree.intersect_aabb(&aabb)
}

fn render_disks(items: &Vec<Disk>, g: &mut G2d, c: &Context, matches: &HashSet<u32>) {
    for disk in items.iter() {
        let rect = [
            disk.cx - disk.radius,
            disk.cy - disk.radius,
            2. * disk.radius,
            2. * disk.radius,
        ];

        let color = if matches.contains(&disk.id) {
            DISK_WITH_HIT
        } else {
            DISK
        };

        ellipse(color, rect, c.transform, g);
        Ellipse::new_border(BLACK, 1.0).draw(rect, &c.draw_state, c.transform, g);
    }
}

fn render_tree_nodes(tree: &QuadTree, g: &mut G2d, c: &Context) {
    tree.visit_leaves(|node| {
        let aabb: AABB = node.get_aabb();
        let rect = [
            aabb.tl.x as f64,
            aabb.tl.y as f64,
            (aabb.br.x - aabb.tl.x) as f64,
            (aabb.br.y - aabb.tl.y) as f64,
        ];
        rectangle(node_color(&node), rect, c.transform, g);
        Rectangle::new_border(BLACK, 1.0).draw(rect, &c.draw_state, c.transform, g);
    });
}

fn node_color(node: &NodeInfo) -> [f32; 4] {
    if node.element_count > 0 {
        QUAD_CELL_FULL
    } else {
        QUAD_CELL_EMPTY
    }
}

fn build_test_data() -> (QuadTree, Vec<Disk>) {
    let mut tree = QuadTree::new(QuadRect::new(-256, -256, 512, 512), TREE_DEPTH);
    let mut items = Vec::new();

    // Build a moving element.
    let item = Disk {
        id: 0,
        cx: -128.0,
        cy: 0.0,
        radius: 32.0,
    };
    tree.insert(QuadTreeElement::new(item.id, item.get_aabb()));
    debug_assert_eq!(items.len(), item.id as usize);
    items.push(item);

    // Build some static elements.
    for i in 0..32 {
        let mut rng = rand::thread_rng();
        let item = Disk {
            id: (i + 1) as _,
            cx: rng.gen_range(-256.0..256.0),
            cy: rng.gen_range(-256.0..256.0),
            radius: rng.gen_range(2.0..16.0),
        };

        tree.insert(QuadTreeElement::new(item.id, item.get_aabb()));
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
