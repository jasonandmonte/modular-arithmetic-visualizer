use nannou::prelude::*;
use std::env;

const CIRCLE_SIZE: f32 = 32.0;
const RING_SPACING: f32 = 40.0;
const RING_RADIUS_SCALE : f32 = 4.0;

struct Model {
    // Natural numbers { 0, 1, 2, ... }
    natural: u32,
    modulus: u32,
    result: u32,
    points: Vec<Point>,
}

#[derive(Debug, PartialEq)]
struct Point {
    x: f32,
    y: f32,
    label: u32,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Usage: {} <natural> <modulus>", args[0]);
        std::process::exit(0);
    }

    nannou::app(model).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    let (natural, modulus) = parse_args(env::args().collect());

    Model {
        natural,
        modulus,
        result: natural % modulus,
        points: generate_points(natural, modulus),
    }
}

/// Parses command-line arguments into a tuple of a natural number and a modulus.
///
/// # Examples
///
/// ```rust
/// let args = vec!["program".to_string(), "12".to_string(), "5".to_string()];
/// let (natural, modulus) = parse_args(args);
/// assert_eq!(natural, 12);
/// assert_eq!(modulus, 5);
/// ```
fn parse_args(args: Vec<String>) -> (u32, u32) {
    if args.len() < 3 {
        eprintln!("Usage: {} <natural> <modulus>", args[0]);
        std::process::exit(1);
    }

    let natural: u32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid number (n >= 0).", args[1]);
            std::process::exit(1);
        }
    };

    let modulus: u32 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid number (n >= 0).", args[2]);
            std::process::exit(1);
        }
    };

    (natural, modulus)
}

/// Creates points with x,y coordinates and labels to be displayed.
///
/// # Examples
///
/// ```rust
/// let natural = 12
/// let modulus = 5
/// let points = generate_points(natural, modulus);
/// assert_eq!(points[0], Point {x: 0.0, y: 32.0, label: 0})
/// ```
fn generate_points(natural: u32, modulus: u32) -> Vec<Point> {
    // NOTE: When evenly divides we want an extra ring (3 mod 3 should be 2 rings)
    let num_rings = (natural + 1).div_ceil(modulus);
    let ring_radius = CIRCLE_SIZE * (modulus as f32) / RING_RADIUS_SCALE ;
    let stride = 360 / modulus;
    let mut points = vec![];

    let mut number: u32 = 0;
    for nr in 0..num_rings {
        let scale_factor = ring_radius + (nr as f32) * RING_SPACING;
        // Map over an array of naturals from 0 to 360 to represent the degrees
        // in a circle.
        for i in (0..360).step_by(stride as usize) {
            let radian = deg_to_rad(i as f32);
            // Get the sine of the radian to find the x co-ordinate of this
            // point of the circle.
            let x = radian.sin() * scale_factor;
            let y = radian.cos() * scale_factor;
            points.push(Point {
                x,
                y,
                label: number,
            });

            number += 1;
        }
    }
    points
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);

    // Draw a point every second
    let visible_points = (app.time as usize).min(model.points.len());

    // NOTE: nannou layers shapes based on when they are drawn.
    // To have rings display below the number circles they must be drawn first.
    draw_rings(&draw, model, visible_points);
    draw_points(&draw, model, visible_points);

    let time = app.time as usize;
    draw_arrow_reduction(&draw, model, time);

    draw.to_frame(app, &frame).unwrap();
}

/// Draws concentric rings based on the number of visible points.
///
/// The number of rings is determined by `ceil[(natural + 1) / modulus`.
/// Rings are drawn in reverse order to accommodate nannou's overlay,
/// with opacity determined by the number of visible points.
fn draw_rings(draw: &Draw, model: &Model, visible_points: usize) {
    // NOTE: When evenly divides we want an extra ring (3 mod 3 should be 2 rings)
    let num_rings = (model.natural + 1).div_ceil(model.modulus) as usize;
    let ppr = model.points.len() / num_rings;

    // Draw largest ring first so it is the base layer.
    for nr in (0..num_rings).rev() {
        // See the ring if the number of points is over the points per ring
        let ring_opacity = if (visible_points.saturating_sub(1) / ppr) >= nr {
            1.0
        } else {
            0.0
        };
        let dynamic_blue = rgba(0.0, 0.75, 1.0, ring_opacity);
        let scale_factor = CIRCLE_SIZE * (model.modulus as f32) / RING_RADIUS_SCALE  + (nr as f32) * RING_SPACING;
        draw.ellipse()
            .radius(scale_factor)
            .no_fill()
            .stroke(dynamic_blue)
            .stroke_weight(2.0);
    }
}

/// Draw points with their associated number label at the center.
///
/// Each point is represented as a small circle with a number label.
/// Points are drawn progressively based on the visible points constraint.
fn draw_points(draw: &Draw, model: &Model, visible_points: usize) {
    for (i, point) in model.points.iter().enumerate() {
        if i >= visible_points {
            break;
        }

        let Point { x, y, label } = point;

        draw.ellipse()
            .w_h(CIRCLE_SIZE, CIRCLE_SIZE)
            .x_y(*x, *y)
            .stroke(BLACK)
            .stroke_weight(1.5);

        draw.text(label.to_string().as_str())
            .x_y(*x, *y)
            .color(BLACK)
            .font_size(10);
    }
}

/// Draws an arrow pointing from the outermost matching point to the innermost matching point.
///
/// Visualizes the reduction by drawing an arrow from the natural number on the
/// outer ring to its associated value on the innermost ring after being
/// reduced by the modulus.
fn draw_arrow_reduction(draw: &Draw, model: &Model, time: usize) {
    if time <= model.points.len() {
        return;
    }

    let matching_points: Vec<&Point> = model
        .points
        .iter()
        .filter(|point| point.label % model.modulus == model.result)
        .collect();
    let transparent_orange = rgba8(245, 173, 66, 150);

    match (matching_points.first(), matching_points.last()) {
        (Some(inner_point), Some(outer_point)) => {
            draw.arrow()
                .color(transparent_orange) // Transparent orange
                .stroke_weight(8.0)
                .start(pt2(outer_point.x, outer_point.y))
                .end(pt2(inner_point.x, inner_point.y));
        }
        _ => {
            eprintln!("No matching points found for the modulus reduction.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let args = vec!["prog".to_string(), "7".to_string(), "3".to_string()];
        assert!(matches!(parse_args(args), (7, 3)));
    }

    #[test]
    fn test_generate_points() {
        let natural = 7;
        let modulus = 3;
        let points = generate_points(natural, modulus);
        let p1 = Point {
            x: 0.0,
            y: 32.0,
            label: 0,
        };
        assert_eq!(points[0], p1);
        // last point on the outer ring
        assert_eq!(points.last().unwrap().label, 8);
    }
}
