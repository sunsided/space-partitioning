use piston_window::*;
use rand::Rng;
use space_partitioning::quadtree::{NodeInfo, QuadRect, QuadTreeElement, AABB};
use space_partitioning::QuadTree;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0; 4];
const DISK: [f32; 4] = [0.0, 1.0, 0.0, 0.5];
const DISK_WITH_HIT: [f32; 4] = [1.0, 1.0, 0.0, 0.5];
const MOUSE: [f32; 4] = [0.8, 0.8, 1.0, 0.25];

const QUAD_CELL_BORDER: [f32; 4] = [0.0, 0.0, 0.0, 0.75];
const QUAD_CELL_FULL: [f32; 4] = [0.25, 0.25, 0.25, 0.25];
const QUAD_CELL_EMPTY: [f32; 4] = [0.25, 0.25, 0.25, 0.125];

struct Disk {
    pub id: u32,
    pub cx: f64,
    pub cy: f64,
    pub radius: f64,
}

fn main() {
    let opengl = OpenGL::V4_5;
    let mut window: PistonWindow = WindowSettings::new("QuadTree", [800, 800])
        .exit_on_esc(true)
        .vsync(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let (mut tree, mut items) = build_test_data();

    let mut rotation = 0.0;
    let mut mouse_pos = [0.0; 2];
    let mut window_size = [0.0; 2];
    let mut items_under_mouse = HashSet::default();
    const CURSOR_SIZE: f64 = 64.0;

    while let Some(e) = window.next() {
        if let Some(args) = e.button_args() {
            if args.state == ButtonState::Press {
                match args.button {
                    Button::Keyboard(Key::R) => {
                        // Clear intersections.
                        items_under_mouse.clear();

                        // Generate new data points
                        let (new_tree, new_items) = build_test_data();
                        tree = new_tree;
                        items = new_items;
                    }
                    _ => {}
                }
            }
        }

        if let Some(pos) = e.mouse_cursor_args() {
            mouse_pos = pos;
        }

        if let Some(args) = e.resize_args() {
            window_size = args.window_size;
        }

        if let Some(args) = e.update_args() {
            rotation += 3.0 * args.dt;

            // Update the element.
            assert!(tree.remove(&QuadTreeElement::new(0, items[0].get_aabb())));
            items[0].cx = -rotation.cos() * 128.0;
            items[0].cy = rotation.sin() * 128.0;
            tree.insert(QuadTreeElement::new(0, items[0].get_aabb()));

            // Compact the tree.
            tree.cleanup();

            // Get new intersections.
            items_under_mouse =
                intersect_with_mouse(&mut tree, &mut window_size, mouse_pos, CURSOR_SIZE);
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
                    mouse_pos[0] - CURSOR_SIZE * 0.5,
                    mouse_pos[1] - CURSOR_SIZE * 0.5,
                    CURSOR_SIZE,
                    CURSOR_SIZE,
                ];
                rectangle(MOUSE, rect, c.transform, g);
                Rectangle::new_border(BLACK, 1.0).draw(rect, &c.draw_state, c.transform, g);

                /*
                for i in 0..5 {
                    let c = c.trans(0.0, i as f64 * 100.0);

                    let black = [0.0, 0.0, 0.0, 1.0];
                    let red = [1.0, 0.0, 0.0, 1.0];
                    let rect = math::margin_rectangle([20.0, 20.0, 60.0, 60.0], i as f64 * 5.0);
                    rectangle(red, rect, c.transform, g);
                    Rectangle::new_border(black, 2.0).draw(rect, &c.draw_state, c.transform, g);

                    let green = [0.0, 1.0, 0.0, 1.0];
                    let h = 60.0 * (1.0 - i as f64 / 5.0);
                    let rect = [120.0, 50.0 - h / 2.0, 60.0, h];
                    ellipse(green, rect, c.transform, g);
                    Ellipse::new_border(black, 2.0).draw(rect, &c.draw_state, c.transform, g);

                    let blue = [0.0, 0.0, 1.0, 1.0];
                    circle_arc(
                        blue,
                        10.0,
                        0.0,
                        f64::_360() - i as f64 * 1.2 - rotation,
                        [230.0, 30.0, 40.0, 40.0],
                        c.transform,
                        g,
                    );

                    let orange = [1.0, 0.5, 0.0, 1.0];
                    line(
                        orange,
                        5.0,
                        [320.0 + i as f64 * 15.0, 20.0, 380.0 - i as f64 * 15.0, 80.0],
                        c.transform,
                        g,
                    );

                    let magenta = [1.0, 0.0, 0.5, 1.0];
                    polygon(
                        magenta,
                        &[
                            [420.0, 20.0],
                            [480.0, 20.0],
                            [480.0 - i as f64 * 15.0, 80.0],
                        ],
                        c.transform,
                        g,
                    );
                }
                */
            });
        }
    }

    // TODO: let candidates = tree.intersect_aabb(&AABB::new(128, 128, 256, 256));
    // TODO: assert!(candidates.len() > 0);
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

fn build_test_data() -> (QuadTree, Vec<Disk>) {
    let mut tree = QuadTree::new(QuadRect::new(-256, -256, 512, 512), 4);
    let mut items = Vec::new();

    let item = Disk {
        id: 0,
        cx: -128.0,
        cy: 0.0,
        radius: 32.0,
    };
    tree.insert(QuadTreeElement::new(item.id, item.get_aabb()));
    debug_assert_eq!(items.len(), item.id as usize);
    items.push(item);

    for i in 1..=32 {
        let mut rng = rand::thread_rng();
        let item = Disk {
            id: i as _,
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
