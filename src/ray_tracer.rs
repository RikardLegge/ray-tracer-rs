use vecmath::{Vector2, vec2_normalized};
use std::f64::consts::SQRT_2;
use std::f64::consts::PI;
//use std::f64::INFINITY;

pub struct RayTracer {}

pub type Point = Vector2<f64>;
type LineSegment = [f64; 4];

pub struct Hit {
    pub point: Point,
    pub strip_id: u32,
    segment_id: usize,
    relative_distance: f64,
}

impl Hit {
    pub fn copy(&self) -> Hit {
        let point = [self.point[0], self.point[1]];
        let strip_id = self.strip_id;
        let segment_id = self.segment_id;
        let relative_distance = self.relative_distance;
        Hit { point, strip_id, segment_id, relative_distance }
    }
}

pub struct LineStrip {
    points: Vec<Point>,
    is_closed: bool,
    id: u32,
}

const EPSILON: f64 = 1e-8;

fn line_segments_to_line_strips(line_segment_lists: &Vec<Vec<LineSegment>>) -> Vec<LineStrip> {
    let mut id = 0;
    line_segment_lists.iter().map(|line_segment_list| {
        let mut points = Vec::with_capacity(line_segment_list.len() + 1);
        for line_segment in line_segment_list {
            let point = [line_segment[0], line_segment[1]];
            points.push(point);
        }
        if let Some(line_segment) = line_segment_list.last() {
            let point = [line_segment[2], line_segment[3]];
            points.push(point);
        }

        let is_closed = {
            let pf = points.first().unwrap();
            let pl = points.last().unwrap();
            let dist = (pf[0] - pl[0]).powi(2) + (pf[1] - pl[1]).powi(2);
            dist < EPSILON
        };

        id += 1;
        LineStrip { points, id, is_closed }
    }).collect()
}

fn angle_between(a: f64, b: f64) -> f64 {
    let mut angle = (a - b).abs();
    if angle > PI {
        angle = PI * 2.0 - angle;
    }
    angle
}

impl RayTracer {
    fn trace_ray(&self, ray: &LineSegment, line_strips: &Vec<LineStrip>) -> Vec<Hit> {
        let r_px = ray[0];
        let r_py = ray[1];
        let r_dx = ray[2] - r_px;
        let r_dy = ray[3] - r_py;

        let mut hits = Vec::new();

        for line_strip in line_strips {
            let id = line_strip.id;
            let points = &line_strip.points;
            let first_point = points[0];
            let mut last_point = [first_point[0], first_point[1]];

            for (i, segment) in points.iter().skip(1).enumerate() {
                let s_px = last_point[0];
                let s_py = last_point[1];
                let s_dx = segment[0] - s_px;
                let s_dy = segment[1] - s_py;
                last_point = [segment[0], segment[1]];

                let t2 = (r_dx * (s_py - r_py) + r_dy * (r_px - s_px)) / (s_dx * r_dy - s_dy * r_dx);
                let t1 = (s_px + s_dx * t2 - r_px) / r_dx;

                if t1 < 0.0 || !(t1 >= 0.0) || t2 < -EPSILON || t2 > 1.0 + EPSILON {
                    continue;
                }

                let x = s_px + s_dx * t2;
                let y = s_py + s_dy * t2;
                hits.push(Hit { point: [x, y], relative_distance: t1, strip_id: id, segment_id: i });
            }
        }

        hits.sort_by(|a, b| a.relative_distance.partial_cmp(&b.relative_distance).unwrap());
        hits
    }

    pub fn trace(&self, source: &Point, line_segment_lists: &Vec<Vec<LineSegment>>) -> Vec<(Hit, Point, bool)> {
        let line_strips = line_segments_to_line_strips(line_segment_lists);
        let mut hit_points = Vec::new();

        for line_strip in line_strips.iter() {
            let points = &line_strip.points;
            let skip = line_strip.is_closed;
            for (i, point) in points.iter().enumerate().filter(|&(i, _)| !(skip && i == points.len()-1)) {
                let point_cpy = [point[0], point[1]];

                let ray = [source[0], source[1], point[0], point[1]];
                let hits = self.trace_ray(&ray, &line_strips);
                let hit = hits.first().unwrap();
                let hit_point = [hit.point[0], hit.point[1]];

                if hit.relative_distance > 1.0 - EPSILON && hits.len() > 1 {
                    let cast_another_ray = {
                        if i != 0 && i != points.len() - 1 {
                            let pp = points[i - 1];
                            let pn = points[i + 1];
                            let vec_p = vec2_normalized([point[0] - pp[0], point[1] - pp[1]]);
                            let vec_n = vec2_normalized([point[0] - pn[0], point[1] - pn[1]]);
                            let vec_s = vec2_normalized([point[0] - source[0], point[1] - source[1]]);

                            let dot = vec_p[0] * vec_n[0] + vec_p[1] * vec_n[1];
                            let dot_p = vec_p[0] * vec_s[0] + vec_p[1] * vec_s[1];
                            let dot_n = vec_n[0] * vec_s[0] + vec_n[1] * vec_s[1];

                            dot_n * dot_p < dot.abs()
                        } else {
//                            let pf = points.first().unwrap();
//                            let pl = points.last().unwrap();
//                            let dist = (pf[0] - pl[0]).powi(2) + (pf[1] - pl[1]).powi(2);
//                            dist > EPSILON
                            true
                        }
                    };

                    if cast_another_ray {
                        for i in 1..hits.len() {
                            let hit_2 = &hits[i];
                            let point = [hit_2.point[0], hit_2.point[1]];
                            let dist = (hit_point[0] - point[0]).powi(2) + (hit_point[1] - point[1]).powi(2);
                            if dist < EPSILON && hit_2.strip_id != hit.strip_id {
                                break;
                            }
                            if dist > EPSILON {
                                hit_points.push((hit_2.copy(), point_cpy, false));
                                break;
                            }
                        }
                    }
                }
                hit_points.push((hit.copy(), point_cpy, true));
            }
        }
        hit_points
    }

    pub fn sort_hits<'a>(&self, source: &Point, hit_points: &'a Vec<(Hit, Point, bool)>) -> Vec<&'a (Hit, Point, bool)> {
        let mut hit_points_angle = hit_points.iter().map(|point| {
            let dx = point.0.point[0] - source[0];
            let dy = point.0.point[1] - source[1];
            let angle: f64 = dy.atan2(dx) + PI;
            return (point, angle);
        }).collect::<Vec<(&(Hit, [f64; 2], bool), f64)>>();
        hit_points_angle.sort_by(|a, b| {
            let diff = (a.1 - b.1).abs();
            if diff < EPSILON {
                (b.0).0.segment_id.cmp(&(a.0).0.segment_id)
            } else {
                b.1.partial_cmp(&a.1).unwrap()
            }
        });

        let len = hit_points_angle.len();
        for i in 0..len {
            let i0 = if i >= len { i - len } else { i };
            let i1 = if i + 1 >= len { i + 1 - len } else { i + 1 };
            let i2 = if i + 2 >= len { i + 2 - len } else { i + 2 };

            let should_swap = {
                let point = hit_points_angle[i0];
                let point_1 = hit_points_angle[i1];
                let point_2 = hit_points_angle[i2];

                let hit = &(point.0).0;
                let hit_1 = &(point_1.0).0;
                let hit_2 = &(point_2.0).0;

                let dist = angle_between(point_1.1, point_2.1).abs();
                if dist < EPSILON {
                    if hit_1.strip_id == hit_2.strip_id {
                        let seg_0 = hit.segment_id as i32;
                        let seg_1 = hit_1.segment_id as i32;
                        let seg_2 = hit_2.segment_id as i32;

                        if seg_0 == seg_2 && seg_1 != seg_2 {
                            true
                        } else {
                            false
                        }
                    } else {
                        hit.strip_id == hit_2.strip_id
                    }
                } else {
                    false
                }
            };

            if should_swap {
                hit_points_angle.swap(i1, i2);
            }
        }

        hit_points_angle.into_iter()
            .map(|point| point.0).collect()
    }
}