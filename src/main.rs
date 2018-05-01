extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate vecmath;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};

mod ray_tracer;

use ray_tracer::{RayTracer, line_segments_to_line_strips};

pub struct App {
    gl: GlGraphics,     // OpenGL drawing backend.
    time: f64,
}

fn rect_to_lines(rect: &[f64; 4]) -> Vec<[f64; 4]> {
    let x = rect[0];
    let y = rect[1];
    let w = rect[2];
    let h = rect[3];

    [
        [x, y, x + w, y],
        [x + w, y, x + w, y + h],
        [x + w, y + h, x, y + h],
        [x, y + h, x, y]
    ].to_vec()
}

fn polygon_to_lines(points: &[[f64; 2]]) -> Vec<[f64; 4]> {
    let first_point = points.get(0).unwrap();
    let mut last_point = [first_point[0], first_point[1]];

    let mut lines = Vec::with_capacity(points.len());

    for point in points.iter().skip(1) {
        let x0 = last_point[0];
        let y0 = last_point[1];
        let x = point[0];
        let y = point[1];
        last_point = [point[0], point[1]];

        lines.push([x0, y0, x, y]);
    }

    lines.push([first_point[0], first_point[1], last_point[0], last_point[1]]);
    return lines;
}

//fn rect_to_points(rect: &[f64; 4]) -> [[f64; 2]; 4] {
//    let x = rect[0];
//    let y = rect[1];
//    let w = rect[2];
//    let h = rect[3];
//
//    [
//        [x, y],
//        [x + w, y],
//        [x + w, y + h],
//        [x, y + h]
//    ]
//}

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GRAY: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 0.3];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 0.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const PURPLE: [f32; 4] = [1.0, 0.2, 1.0, 1.0];

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let line_segments = [
            vec!([100.0, 100.0, 100.0, 200.0], [100.0, 200.0, 200.0, 100.0], [200.0, 100.0, 100.0, 100.0]),
            vec!([700.0, 300.0, 700.0, 400.0]),
//            vec!([300.0, 100.0,500.0, 100.0]),
//            vec!([300.0, 700.0, 500.0, 700.0]),
//            vec!([350.0, 600.0, 500.0, 600.0]),
            vec!([500.0, 300.0, 500.0, 400.0], [500.0, 400.0, 600.0, 300.0]),
            vec!([100.0, 400.0, 200.0, 400.0], [200.0, 400.0, 200.0, 500.0]),
            vec!([100.0, 700.0, 200.0, 600.0], [200.0, 600.0, 100.0, 600.0]),
            vec!([400.0, 100.0, 400.0, 500.0]),

//                rect_to_lines(&rectangle::square(500.0, 200.0, 100.0)),
            rect_to_lines(&rectangle::square(50.0, 50.0, 700.0)),
            rect_to_lines(&rectangle::square(300.0, 300.0, 100.0)),
        ].to_vec();
        let time = self.time;
        let ray_tracer = RayTracer {};
        let mut light_source = [400.0, 400.0];

        self.gl.draw(args.viewport(), |c, gl| {
//            light_source[1] = 400.0 - (time / 100.0).sin() * 50.0;
//            light_source[0] = 400.0 + (time / 66.0).sin() * 50.0;
            light_source[0] = 410.0 + (time / 100.0).sin().powi(3) * 400.0;
            light_source[1] = 410.0 - (time / 100.0).sin() * 400.0;

            let transform = c.transform;

            clear(WHITE, gl);

            let line_strips = line_segments_to_line_strips(&line_segments);
            let hit_points = ray_tracer.trace(&light_source, &line_strips);

            for hit in &hit_points {
                let target  = hit.target_point;
                let target_ray = [light_source[0], light_source[1], target[0], target[1]];
                line(BLUE, 1.0, target_ray, transform, gl);

                if hit.ray_segment == 0 {
                    let hit_ray = [light_source[0], light_source[1], hit.point[0], hit.point[1]];
                    line(GREEN, 1.0, hit_ray, transform, gl);
                } else {
                    let hit_ray = [target[0], target[1], hit.point[0], hit.point[1]];
                    line(RED, 1.0, hit_ray, transform, gl);
                }
            }

            for line_strip in line_segments {
                for segment in line_strip {
                    line(BLACK, 1.0, segment, transform, gl);
                }
            }

            let sorted_hit_points = ray_tracer.sort_hits(&light_source, &hit_points);
            let polygon_points = sorted_hit_points.iter().map(|hit| hit.point).collect::<Vec<[f64; 2]>>();

//            let first_point = polygon_points.first().unwrap();
//            let mut last_point = first_point;
//            for point in polygon_points.iter().skip(1) {
//                let triangle = [
//                    light_source,
//                    [last_point[0], last_point[1]],
//                    [point[0], point[1]],
//                ];
//                last_point = &point;
//                polygon(GRAY, &triangle, transform, gl);
//            }
//            let triangle = [
//                light_source,
//                [last_point[0], last_point[1]],
//                [first_point[0], first_point[1]],
//            ];
//            polygon(GRAY, &triangle, transform, gl);

//            for segment in polygon_to_lines(&polygon_points[..]) {
//                line(PURPLE, 1.0, segment, transform, gl);
//            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.time += 1.0;
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
        "spinning-square",
        [800, 800],
    )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        time: 0.0,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}