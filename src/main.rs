use std::env;
use nannou::prelude::*;

struct Model {
    integer: u32,
    modulus: u32,
    result: u32,
}

fn main() {
    nannou::app(model)
        .event(event)
        .simple_window(view)
        .run();
}

fn model(_app: &App) -> Model {
    let (integer, modulus) = parse_args();
    // TODO: Add --help flag

    Model {
        integer,
        modulus,
        result: integer % modulus,
    }
}

fn parse_args() -> (u32, u32) {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <integer> <modulus>", args[0]);
        std::process::exit(1);
    }

    let integer: u32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid integer (n >= 0).", args[1]);
            std::process::exit(1);
        }
    };

    let modulus: u32 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid integer (n >= 0).", args[2]);
            std::process::exit(1);
        }
    };

    (integer, modulus)
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);
    

    // TODO: Account for rings
    let _num_rings = model.integer.div_ceil(model.modulus);

    // Determine coordinates for each point
    // FIXME: magic numbers
    let radius = 36.0 / 3.0 * (model.modulus as f32);

    let ring_points  = model.modulus; 
    let stride = 360 / ring_points;
    // Map over an array of integers from 0 to 360 to represent the degrees in a circle.
    let points = (0..360).step_by(stride as usize)
        .map(|i| {
            // Convert each degree to radians.
            let radian = deg_to_rad(i as f32);
            // Get the sine of the radian to find the x co-ordinate of this point of the circle
            // and multiply it by the radius.
            let x = radian.sin() * radius;
            // Do the same with cosine to find the y co-ordinate.
            let y = radian.cos() * radius;
            // Construct and return a point object with a color.
            (x,y)
    });

    let time: f32 = app.time;
    // Show a point every second
    let ring_points = (time as usize).min(points.len());

    // FIXME: Account for ring levels
    let ring_opacity = if ring_points == points.len() { 1.0 } else { 0.0 };

    // TODO: Make ring layers
    // NOTE: nannou layers shapes based on when they are drawn.
    // To have rings display below the integer shapes they must be drawn first.
    draw.ellipse()
        .radius(radius)
        .no_fill()
        .stroke(rgba(0.0, 0.75, 1.0, ring_opacity))
        .stroke_weight(2.0);

    for (i, (x, y)) in points.take(ring_points).enumerate() {
        draw.ellipse()
            .w_h(36.0, 36.0) // FIXME: Make a const
            .x_y(x, y)
            .stroke(BLACK)
            .stroke_weight(1.0);

        // Add number inside circle
        draw.text(&i.to_string())
            .x_y(x, y)
            .color(BLACK)
            .font_size(14);
    }

    draw.to_frame(app, &frame).unwrap();
}
