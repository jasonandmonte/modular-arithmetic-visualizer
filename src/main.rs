use nannou::prelude::*;
use nannou_egui::{egui, Egui};

const CIRCLE_SIZE: f32 = 32.0;
const RING_SPACING: f32 = 40.0;
const RING_RADIUS_SCALE: f32 = 4.0;

/// Holds geometry for the number representation.
#[derive(Debug, Copy, Clone, PartialEq)]
struct Point {
    /// X coordinate.
    x: f32,
    /// Y coordinate.
    y: f32,
    /// Number to be displayed within the point.
    label: u32,
}

/// State of the application for drawing modular arithmetic.
struct Model {
    /// Show cycles within mod group.
    cycle: bool,
    /// Natural numbers. { 0, 1, 2, ... }
    natural: u32,
    /// Base of modular arithmetic.
    modulus: u32,
    /// Result of the mod operation.
    result: u32,
    /// Points with coordinates.
    points: Vec<Point>,
    /// User interface component.
    egui: Egui,
    /// Counter for drawing animations.
    time: f32,
    /// Update natural from UI.
    new_natural: u32,
    /// Update modulus from UI.
    new_modulus: u32,
    /// Update cycle from UI.
    new_cycle: bool,
}

/// Points to determine the an arrow's direction
struct ArrowPoints {
    /// Starting point.
    start: Point2,
    /// Ending point.
    end: Point2,
}

fn main() {
    nannou::app(model).update(update).run();
}

/// Configures the starting state for the drawing.
fn model(app: &App) -> Model {
    // Create window
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    // Start with a basic drawing
    Model {
        egui,
        cycle: false,
        natural: 7,
        modulus: 3,
        result: 7 % 3,
        points: generate_points(false, 7, 3),
        time: 0.0,
        new_natural: 7,
        new_modulus: 3,
        new_cycle: false,
    }
}

/// Handles changes from the user interface.
fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    model.time += 0.04;

    egui::Window::new("Configuration").show(&ctx, |ui| {
        ui.label("Operand:");
        ui.add(egui::Slider::new(&mut model.new_natural, 1..=40));

        ui.label("Modulus:");
        ui.add(egui::Slider::new(&mut model.new_modulus, 1..=40));

        if ui
            .add(egui::RadioButton::new(!model.new_cycle, "Calculation"))
            .clicked()
        {
            model.new_cycle = false;
        }
        if ui
            .add(egui::RadioButton::new(model.new_cycle, "Cycles"))
            .clicked()
        {
            model.new_cycle = true;
        }

        // Regenerate the drawing
        let clicked = ui.button("Generate").clicked();
        if clicked {
            model.cycle = model.new_cycle;
            model.natural = model.new_natural;
            model.modulus = model.new_modulus;
            model.result = model.natural % model.modulus;
            model.points = generate_points(model.cycle, model.natural, model.modulus);
            // Reset time
            model.time = 0.0;
        }
    });
}

/// Presents drawing to the window.
fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);

    if model.cycle {
        draw_points(&draw, model);
        draw_cycle_arrows(&draw, model);
    } else {
        // NOTE: nannou layers shapes based on when they are drawn.
        // To have rings display below the number circles they must be drawn first.
        draw_rings(&draw, model);
        draw_points(&draw, model);
        draw_arrow_reduction(&draw, model);
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

//------------------------------------------------------------------------------
// Model Helpers
//------------------------------------------------------------------------------

/// Handles window events and forwards them to the egui instance.
fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

//------------------------------------------------------------------------------
// Update Helpers
//------------------------------------------------------------------------------

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
            let current_ring_max = modulus * (nr + 1);
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

//------------------------------------------------------------------------------
// View Helpers
//------------------------------------------------------------------------------

/// Draws concentric rings based on the number of visible points.
///
/// The number of rings is determined by `ceil[(natural + 1) / modulus`.
/// Rings are drawn in reverse order to accommodate nannou's overlay,
/// with opacity determined by the number of visible points.
fn draw_rings(draw: &Draw, model: &Model) {
    let rings = compute_rings(
        model.natural,
        model.modulus,
        model.points.len(),
        model.time as usize,
    );

    for (scale_factor, opacity) in rings {
        let dynamic_blue = rgba(0.0, 0.75, 1.0, opacity);
        draw.ellipse()
            .radius(scale_factor)
            .no_fill()
            .stroke(dynamic_blue)
            .stroke_weight(2.0);
    }
}

/// Auxiliary function to compute rings and properties.
fn compute_rings(natural: u32, modulus: u32, points_len: usize, time: usize) -> Vec<(f32, f32)> {
    // NOTE: When evenly divides we want an extra ring (3 mod 3 should be 2 rings)
    let num_rings = (natural + 1).div_ceil(modulus) as usize;
    let ppr = points_len / num_rings;
    let mut rings = vec![];

    // Draw largest ring first so it is the base layer.
    for nr in (0..num_rings).rev() {
        // See the ring if the number of points is over the points per ring
        let ring_opacity = if (time.saturating_sub(1) / ppr) >= nr {
            1.0
        } else {
            0.0
        };
        let scale_factor =
            CIRCLE_SIZE * (modulus as f32) / RING_RADIUS_SCALE + (nr as f32) * RING_SPACING;
        rings.push((scale_factor, ring_opacity));
    }
    rings
}

/// Draw points with their associated number label at the center.
///
/// Each point is represented as a small circle with a number label.
/// Points are drawn progressively based on the visible points constraint.
fn draw_points(draw: &Draw, model: &Model) {
    for (i, point) in model.points.iter().enumerate() {
        // We want all points to be drawn for a cycle
        if !model.cycle && i >= model.time as usize {
            return;
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
fn draw_arrow_reduction(draw: &Draw, model: &Model) {
    if model.time as usize <= model.points.len() {
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
            let arrow_points = shrink_arrow(outer_point, inner_point);

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
fn draw_cycle_arrows(draw: &Draw, model: &Model) {
    for (i, start_point) in model.points.iter().enumerate() {
        if i >= model.time as usize {
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
        end: pt2(x2, y2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_points() {
        let test_cases = vec![
            (false, 7, 3, 9),
            (false, 3, 3, 6),
            (false, 17, 12, 24),
            // Should be a single ring with a modulus number of points
            (true, 1, 3, 3),
            (true, 2, 7, 7),
            (true, 5, 12, 12),
        ];

        for (cycle, natural, modulus, expect_num_points) in test_cases {
            let points = generate_points(cycle, natural, modulus);
            assert_eq!(
                points.len(),
                expect_num_points,
                "Should have same number of points. Failed for natural={}, modulus={}",
                natural,
                modulus
            );
            assert_eq!(
                points.last().unwrap().label,
                (expect_num_points as u32) - 1,
                "Last point should match the expected last number"
            );
        }
    }

    #[test]
    fn test_shrink_arrow() {
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

    #[test]
    fn test_compute_rings() {
        let natural = 7;
        let modulus = 3;
        let points_len = 10;
        let time = 5;

        let rings = compute_rings(natural, modulus, points_len, time);

        let expected_num_rings = (natural + 1).div_ceil(modulus) as usize;
        assert_eq!(
            rings.len(),
            expected_num_rings,
            "Should have same number of rings"
        );
        assert_eq!(rings[0].0, 104.0, "Should have ring scaling");
        assert_eq!(rings[0].1, 0.0, "Should be opaque");
    }
}
