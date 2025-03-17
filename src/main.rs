use clap::Parser;
use nannou::prelude::*;

const CIRCLE_SIZE: f32 = 32.0;
const RING_SPACING: f32 = 40.0;
const RING_RADIUS_SCALE: f32 = 4.0;

struct Model {
    // Natural numbers { 0, 1, 2, ... }
    cycle: bool,
    natural: u32,
    modulus: u32,
    result: u32,
    points: Vec<Point>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point {
    x: f32,
    y: f32,
    label: u32,
}

struct ArrowPoints {
    start: Point2,
    end: Point2,
}

#[derive(Debug, Parser)]
struct Args {
    /// Enable cycle mode
    #[arg(short = 'c', long = "cycle")]
    cycle: bool,
    /// Operand (reduction or cycle addition)
    #[arg(allow_hyphen_values = true, value_parser = validate_number)]
    natural: u32,
    /// The modulus value
    #[arg(allow_hyphen_values = true, value_parser = validate_number)]
    modulus: u32,
}

fn main() {
    // --help side effect
    Args::parse();
    nannou::app(model).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    // let (cycle, natural, modulus) = parse_args(Args::parse());
    let args = Args::parse();
    let (cycle, natural, modulus) = (args.cycle, args.natural, args.modulus);

    Model {
        cycle,
        natural,
        modulus,
        result: natural % modulus,
        points: generate_points(cycle, natural, modulus),
    }
}

/// TODO:
fn validate_number(s: &str) -> Result<u32, String> {
    match s.parse::<u32>() {
        Ok(n) => Ok(n),
        Err(_) => Err(format!("error: '{}' must be a 0+ number", s)),
    }
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
fn generate_points(cycle: bool, natural: u32, modulus: u32) -> Vec<Point> {
    // NOTE: When evenly divides we want an extra ring (3 mod 3 should be 2 rings)
    let num_rings = if cycle {
        1
    } else {
        (natural + 1).div_ceil(modulus)
    };
    let ring_radius = if cycle {
        320.0
    } else {
        CIRCLE_SIZE * (modulus as f32) / RING_RADIUS_SCALE
    };
    let stride = 360 / modulus as usize;
    let mut points = vec![];

    let mut number: u32 = 0;
    for nr in 0..num_rings {
        let scale_factor = ring_radius + (nr as f32) * RING_SPACING;
        // Map over an array of naturals from 0 to 360 to represent the degrees
        // in a circle.
        for i in (0..360).step_by(stride) {
            // For some numbers the stride can fit an extra erroneous point
            let current_ring_max = modulus * (nr+1);
            if number >= current_ring_max {
                break;
            }
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

    if model.cycle {
        draw_points(&draw, model, model.points.len());
        draw_cycle_arrows(&draw, model, visible_points);
    } else {
        // NOTE: nannou layers shapes based on when they are drawn.
        // To have rings display below the number circles they must be drawn first.
        draw_rings(&draw, model, visible_points);
        draw_points(&draw, model, visible_points);

        let time = app.time as usize;
        draw_arrow_reduction(&draw, model, time);
    }

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
        let scale_factor =
            CIRCLE_SIZE * (model.modulus as f32) / RING_RADIUS_SCALE + (nr as f32) * RING_SPACING;
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
            .font_size(12);
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
            let arrow_points= shrink_arrow(outer_point, inner_point);

            draw.arrow()
                .color(transparent_orange) // Transparent orange
                .stroke_weight(8.0)
                .points(arrow_points.start, arrow_points.end);
        }
        _ => {
            eprintln!("No matching points found for the modulus reduction.");
        }
    }
}

/// Draws arrows between points based on the addition of a natural number.
///
/// Iterates over the points in the model and draws an arrow per second and
/// shows the cycles in a modulo.
fn draw_cycle_arrows(draw: &Draw, model: &Model, time: usize) {
    for (i, start_point) in model.points.iter().enumerate() {
        // Draw one arrow per second
        if i >= time {
            break;
        }

        let end = (i + model.natural as usize) % model.modulus as usize;
        let end_point = &model.points[end];

        let arrow_points = shrink_arrow(start_point, end_point);

        draw.arrow()
            .color(ORANGE)
            .stroke_weight(4.0)
            .head_width(12.0)
            .points(arrow_points.start, arrow_points.end);
    }
}

/// Shortens an arrow between two points by adjusting moving the start and end
/// points.
///
/// This function applies linear interpolation (LERP) to move the start and end
/// points closer to each other.
fn shrink_arrow(start_point: &Point, end_point: &Point) -> ArrowPoints {
    // lerp: https://youtu.be/jvPPXbo87ds?si=eQEpr14Vs_9zIeH3&t=140
    // We want to reduce the arrow to minimize overlap with points
    let t_factor = 0.05;
    let x1 = start_point.x + t_factor * (end_point.x - start_point.x);
    let y1 = start_point.y + t_factor * (end_point.y - start_point.y);
    let x2 = end_point.x - t_factor * (end_point.x - start_point.x);
    let y2 = end_point.y - t_factor * (end_point.y - start_point.y);

    ArrowPoints {
        start: pt2(x1, y1),
        end: pt2(x2, y2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_points() {
        let cycle = false;
        let natural = 7;
        let modulus = 3;
        let points = generate_points(cycle, natural, modulus);
        let p1 = Point {
            x: 0.0,
            y: 24.0,
            label: 0,
        };
        assert_eq!(points[0], p1);
        // last point on the outer ring
        assert_eq!(points.last().unwrap().label, 8);
    }

    #[test]
    fn test_scale_down_arrow_points() {
        let p1 = Point {
            x: 0.0,
            y: 32.0,
            label: 0,
        };
        let p2 = Point {
            x: 0.0,
            y: -32.0,
            label: 1,
        };
        let arrow_points = shrink_arrow(&p1, &p2);

        assert!(arrow_points.start.y < p1.y, "Should move start point down");
        assert!(arrow_points.end.y > p2.y, "Should move end point up");
    }
}
