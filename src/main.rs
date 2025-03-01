use std::env;
use nannou::prelude::*;

const CIRCLE_SIZE: f32 = 36.0;
const RING_SPACING: f32 = 40.0;

struct Model {
    integer: u32,
    modulus: u32,
    result: u32,
    points: Vec<Point>,
}

#[derive(Debug)]
struct Point {
    x: f32,
    y: f32,
    label: String,
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

    
    // TODO: Create a function
    let num_rings = integer.div_ceil(modulus);

    // Determine coordinates for each point
    // FIXME: magic numbers
    let ring_radius = CIRCLE_SIZE / 3.0 * (modulus as f32);
    let ring_points  = modulus; 
    let stride = 360 / ring_points;
    let mut points = vec![];

    let mut count: u32 = 0;
    for nr in 0..num_rings {
        let scale_factor = ring_radius + (nr as f32) * RING_SPACING;
        // Map over an array of integers from 0 to 360 to represent the degrees in a circle.
        for i in (0..360).step_by(stride as usize) {
            let radian = deg_to_rad(i as f32);
            // Get the sine of the radian to find the x co-ordinate of this point of the circle
            // and multiply it by the ring radius.
            let x = radian.sin() * scale_factor;
            // Do the same with cosine to find the y co-ordinate.
            let y = radian.cos() * scale_factor;
            // Construct and return a point object with a color.
            // (x,y)
            points.push(Point {
                x,
                y,
                label: count.to_string(),
            });
            
            count += 1;
        }
    }

    Model {
        integer,
        modulus,
        result: integer % modulus,
        points,
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
   

    // TODO: draw_rings function
    let time: f32 = app.time;
    // Show a point every second
    let timed_draw = (time as usize).min(model.points.len());

    // FIXME: Account for ring levels
    let ring_opacity = 1.0; //if timed_draw == points.len() { 1.0 } else { 0.0 };

    // NOTE: nannou layers shapes based on when they are drawn.
    // To have rings display below the integer shapes they must be drawn first.
    // Draw largest first so it is the base layer
    for nr in (0..model.integer.div_ceil(model.modulus)).rev() {
        draw.ellipse()
            .radius(CIRCLE_SIZE / 3.0 * (model.modulus as f32) + (nr as f32) * RING_SPACING)
            .no_fill()
            .stroke(rgba(0.0, 0.75, 1.0, ring_opacity))
            .stroke_weight(2.0);
    }


    // TODO: draw_points function
    for (i, point) in model.points.iter().enumerate() {
        if i >= timed_draw {
            break;
        }
    
        let Point { x, y, label } = point;
        println!("{}:({},{})", label, x, y);
    
        draw.ellipse()
            .w_h(CIRCLE_SIZE, CIRCLE_SIZE)
            .x_y(*x, *y)
            .stroke(BLACK)
            .stroke_weight(1.0);
    
        draw.text(label.as_str())
            .x_y(*x, *y)
            .color(BLACK)
            .font_size(14);
    }

    draw.to_frame(app, &frame).unwrap();
}
