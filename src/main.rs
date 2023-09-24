extern crate sdl2;
extern crate roots;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::video::Window;
use sdl2::render::Canvas;

use ttf_parser::Face;
use std::fs;

use crate::segment::Segment;
mod segment;

use crate::outline::extract_outline;
mod outline;

use crate::metrics::get_render_score;
mod metrics;

const WINDOW_WIDTH: u32 = 1300u32;
const WINDOW_HEIGHT: u32 = 800u32;
const CANVAS_MARGIN: u32 = 100u32;

// Pixel density of target display
const RESOLUTION: f32 = 144f32;
// Font size for output
const POINT_SIZE: f32 = 32f32;

// Character we're drawing:
const TEST_CHARACTER: char = 'a';
// Samples for performance testing
const TEST_SAMPLES: i32 = 10000;

// const FONT_PATH: &str = "./fonts/DancingScript-Regular.ttf";
// const FONT_PATH: &str = "./fonts/Chopinscript-gxXE.ttf";
// const FONT_PATH: &str = "./fonts/Roboto-Regular.ttf";
// const FONT_PATH: &str = "./fonts/Pacifico-Regular.ttf";
// const FONT_PATH: &str = "./fonts/NotoSerifJP-Regular.otf";
// const FONT_PATH: &str = "./fonts/DarumadropOne-Regular.ttf";
// const FONT_PATH: &str = "./fonts/wingding.ttf";
 const FONT_PATH: &str = "./fonts/Creepster-Regular.ttf";

const SAMPLE_MODE: Supersampling = Supersampling::ThreeByThree;


#[derive(Debug, PartialEq, Eq)]
enum Supersampling {
    None,
    TwoByTwo,
    ThreeByThree,
}

fn main() {
    // "Oh we can just do the project in Rust; it can't
    // be *that* weird to learn" -- me, an idiot, a week ago
    let metrics = get_render_score(TEST_CHARACTER, FONT_PATH, TEST_SAMPLES);
    println!("Score for default sampling: {:?}", metrics.none);
    println!("Score for 2x2 supersampling: {:?}", metrics.two_by_two);
    println!("Score for 3x3 supersampling: {:?}", metrics.three_by_three);
    println!("Score for 4x4 supersampling: {:?}", metrics.four_by_four);

    match sdl_demo() {
        Ok(()) => {}
        Err(e) => {
            eprint!("Error {}.", e);
            std::process::exit(1);
        }
    };
}

fn draw_text(text: &str, start_x: i32, start_y: i32, canvas: &mut Canvas<Window>, current_sample_mode: &Supersampling, font_path: Option<&str>) -> Point {
    let (canvas_width, canvas_height) = canvas.logical_size();
    let x_spacing = 0; // Hard coded - additional space between charafcters
    let y_spacing = POINT_SIZE as i32 * 4; // Hard coded - vertical space between characters
    let mut last_x = start_x;
    let mut last_y = start_y;
    let mut bottom_right = Point::new(start_x, start_y);
    for line in text.lines() {
        last_x = start_x;
        for c in line.chars() {
            if last_x > WINDOW_WIDTH as i32 - (2 * CANVAS_MARGIN) as i32 {
                last_x = start_x; // Go to new line
                last_y = bottom_right.y + y_spacing;
            }
            bottom_right = draw_character(c, last_x, last_y, canvas, current_sample_mode, font_path);
            last_x = bottom_right.x + x_spacing;
        }
        last_y = bottom_right.y + y_spacing;
    }
    return Point::new(last_x - x_spacing, last_y - y_spacing); // Remove the uncessary last space that is produced by adding x_spacing at the end
}

fn draw_character(character: char, start_x: i32, start_y: i32, canvas: &mut Canvas<Window>, current_sample_mode: &Supersampling, font_path: Option<&str>) -> Point {
    // Draws character to canvas and returns right most x coordiante in canvas coordinates
    
    // STEP 2: extract data from font file
    let file = fs::read(font_path.unwrap_or_else(|| FONT_PATH)).unwrap();
    let face = match Face::parse(&file, 0) {
        Ok(f) => f,
        Err(e) => {
            eprint!("Error: {}.", e);
            std::process::exit(1);
        }
    };
    let units_per_em = face.units_per_em() as f32;

    if character == ' ' { // Handle space character separately since it is not included in the ttf file
        let (segments, bbox) = extract_outline(&face, 'a'); // Use the bounding box of lowercase a
        let bbox_height = bbox.unwrap().height() as f32;
        let bbox_width = bbox.unwrap().width() as f32;
        let (bbox_width_pixel_units, bbox_height_pixel_units) = font_to_pixel_units(bbox.unwrap().x_max.into(), bbox.unwrap().y_max.into(), units_per_em);
        let bottom_right = sdl_to_canvas(start_x + bbox_width_pixel_units as i32, -start_y + bbox_height_pixel_units as i32, canvas);
        return bottom_right;
    }

    //let (canvas_width, canvas_height) = canvas.logical_size();

    // STEP 3: get bounding box for characters 
    let (segments, bbox) = extract_outline(&face, character);
    let bbox_height = bbox.unwrap().height() as f32;
    let bbox_width = bbox.unwrap().width() as f32;
    let (x_pixels, y_pixels) = get_glyph_size(bbox_height, bbox_width, units_per_em);

    // STEP 5: draw pixels corresponding to character

    // NOTE: font coordinates have (0, 0) in the bottom-left
    // and SDL coordinates have (0, 0) in the top-left.
    // Had to trace a letter out manually to figure this out
    // lmao. -- James M
    let mut x = 0;
    let mut y = 0;

    let x_min = bbox.unwrap().x_min as f32;
    let y_min = bbox.unwrap().y_min as f32;

    //let factor = 1;

    while x < x_pixels {
        while y < y_pixels {
            let x_center = x as f32 + 0.5f32;
            let y_center = y as f32 + 0.5f32;
            let x_baseline = x as f32;
            let y_baseline = y as f32;
            let mut color_factor = 0;
            let division_factor: i32;
            if *current_sample_mode == Supersampling::ThreeByThree {
                division_factor = 3;
            } else if *current_sample_mode == Supersampling::TwoByTwo {
                division_factor = 2;
            } else {
                division_factor = 1;
            }

            if *current_sample_mode == Supersampling::TwoByTwo || *current_sample_mode == Supersampling::ThreeByThree {
                for x_subsample in 0..division_factor {
                    for y_subsample in 0..division_factor {
                        let curr_x_coord = x_baseline + ((x_subsample as f32)/(division_factor as f32)) + (0.5/(division_factor as f32));
                        let curr_y_coord = y_baseline + ((y_subsample as f32)/(division_factor as f32)) + (0.5/(division_factor as f32));
                        if should_draw_point(curr_x_coord, curr_y_coord, x_min, y_min, units_per_em, &segments) {
                            color_factor += 1;
                        }
                        // print!("{}", color_factor);
                    }
                }
                
                let color_factor_ratio = (255f32) / (division_factor as f32 * division_factor as f32);
                let actual_color = (255f32 - (color_factor as f32)*color_factor_ratio) as u8;
                
                canvas.set_draw_color(Color::RGBA(actual_color,actual_color, actual_color, 255));
                // match canvas.draw_point(Point::new(100 + x + start_x, 700 - y - start_y)) {
                match canvas.draw_point(sdl_to_canvas(start_x + x, y - start_y, canvas)) {
                
                    Ok(()) => {}
                    Err(e) => {
                        eprint!("Error {}.", e);
                        std::process::exit(1);
                    }
                };
            } else if *current_sample_mode == Supersampling::None {
                if should_draw_point(x_center, y_center, x_min, y_min, units_per_em, &segments) {
                    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
                    match canvas.draw_point(sdl_to_canvas(start_x + x, y - start_y, canvas)) {
                        Ok(()) => {}
                        Err(e) => {
                            eprint!("Error {}.", e);
                            std::process::exit(1);
                        }
                    };
                }
            }
            y += 1;
        }
        y = 0;
        x += 1;
    }

    let (bbox_width_pixel_units, bbox_height_pixel_units) = font_to_pixel_units(bbox.unwrap().x_max.into(), bbox.unwrap().y_max.into(), units_per_em);
    let bottom_right = sdl_to_canvas(start_x + bbox_width_pixel_units as i32, -start_y + bbox_height_pixel_units as i32, canvas);
    return bottom_right;
    // let rightmost_x = bottom_right.x;
    // let bottommost_y = bottom_right.y;
    // return rightmost_x;
}

fn sdl_demo() -> Result<(), String> {
    /* Starts a window render. If this is not working, you need to install SDL2 on
    your system. If you use Mac OS the easiest way to do this is to install Homebrew
    and run `brew install sdl2`. Windows is a little more complicated, but
    instructions are available on the `rust-sdl2` github page. If you use Linux you
    probably don't need my help for this. */
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;


    // STEP 1: set up window render
    let window = video_subsystem
        .window("rust-sdl2 demo: Window", WINDOW_WIDTH, WINDOW_HEIGHT)
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // TODO: ideally we wanna center the character within the window but I gotta
    // check my math on this one first
    // let x_render_offset = ((400f32 - bbox_width) / 2f32).floor() as i32;
    // let y_render_offset = ((600f32 - bbox_height) / 2f32).floor() as i32;

    let mut current_sample_mode = SAMPLE_MODE;

    // STEP 4: start loop for render (the next few lines are unimportant)
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    if current_sample_mode == Supersampling::None {
                        current_sample_mode = Supersampling::TwoByTwo;
                    } else if current_sample_mode == Supersampling::TwoByTwo {
                        current_sample_mode = Supersampling::ThreeByThree
                    } else if current_sample_mode == Supersampling::ThreeByThree {
                        current_sample_mode = Supersampling::None;
                    }
                },
                _ => {}
            }
        }
        canvas.clear();
        
        // STEP 5: draw pixels corresponding to character

         let hello = "pls give us 25/25 \nprof. ng and prof. o'brien <3";
        // let hello = "We the People of the United States, in Order to form a more perfect Union, establish Justice, insure domestic Tranquility, provide for the common defense, promote the general Welfare, and secure the Blessings of Liberty to ourselves and our Posterity, do ordain and establish this Constitution for the United States of America.";
        // let hello = "We the People of the United States, in Order\nto form a more perfect Union, establish\nJustice, insure domestic Tranquility, provide for\nthe common defense, promote the general\nWelfare, and secure the Blessings of Liberty\nto ourselves and our Posterity, do ordain and\nestablish this Constitution for the United\nStates of America.";
         draw_text(hello, (CANVAS_MARGIN + POINT_SIZE as u32) as i32, (CANVAS_MARGIN + POINT_SIZE as u32 * 3) as i32, &mut canvas, &current_sample_mode, Some(FONT_PATH));

        //draw_text("We the People of the United States, in Order", (CANVAS_MARGIN + POINT_SIZE as u32) as i32, (CANVAS_MARGIN + POINT_SIZE as u32 * 3) as i32, &mut canvas, &current_sample_mode, Some(FONT_PATH));
        //draw_text("to form a more perfect Union, establish", (CANVAS_MARGIN + POINT_SIZE as u32) as i32, (CANVAS_MARGIN + POINT_SIZE as u32 * 3) as i32 + 100, &mut canvas, &current_sample_mode, Some("./fonts/Roboto-Regular.ttf"));
        //draw_text("Justice, insure domestic Tranquility, provide for", (CANVAS_MARGIN + POINT_SIZE as u32) as i32, (CANVAS_MARGIN + POINT_SIZE as u32 * 3) as i32 + 200, &mut canvas, &current_sample_mode, Some("./fonts/DancingScript-Regular.ttf"));
        //draw_text("the common defense, promote the", (CANVAS_MARGIN + POINT_SIZE as u32) as i32, (CANVAS_MARGIN + POINT_SIZE as u32 * 3) as i32 + 300, &mut canvas, &current_sample_mode, Some("./fonts/Pacifico-Regular.ttf"));
        //draw_text("Welfare, and secure the Blessings of Liberty", (CANVAS_MARGIN + POINT_SIZE as u32) as i32, (CANVAS_MARGIN + POINT_SIZE as u32 * 3) as i32 + 400, &mut canvas, &current_sample_mode, Some("./fonts/Chopinscript-gxXE.ttf"));
        //draw_text("to ourselves and our Posterity, do ordain and", (CANVAS_MARGIN + POINT_SIZE as u32) as i32, (CANVAS_MARGIN + POINT_SIZE as u32 * 3) as i32 + 500, &mut canvas, &current_sample_mode, Some("./fonts/Creepster-Regular.ttf"));
        //draw_text("establish this Constitution for the United\nStates of America.", (CANVAS_MARGIN + POINT_SIZE as u32) as i32, (CANVAS_MARGIN + POINT_SIZE as u32 * 3) as i32 + 600, &mut canvas, &current_sample_mode, Some("./fonts/ComicSansMS3.ttf"));



        // let world = "World";
        // draw_text(world, 130, 500, &mut canvas, &current_sample_mode);



        // STEP 6: draw border around render area (just looks nicer idk)
        let rect = sdl2::rect::Rect::new(100, 100, WINDOW_WIDTH - CANVAS_MARGIN * 2, WINDOW_HEIGHT - CANVAS_MARGIN * 2);
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        match canvas.draw_rect(rect) {
            Ok(()) => {}
            Err(e) => {
                eprint!("Error {}.", e);
                std::process::exit(1);
            }
        };

        // Fill in any undrawn pixels with white
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        canvas.present();
    }

    return Ok(());
}

fn sdl_to_canvas(x: i32, y: i32, canvas: &Canvas<Window>) -> Point {
    return Point::new(x, (canvas.logical_size().1 as i32) - y);
}

fn should_draw_point(x: f32, y:f32, x_min: f32, y_min: f32, units_per_em:f32, segments: &Vec<Segment>) -> bool {
    // https://developer.apple.com/fonts/TrueType-Reference-Manual/RM02/Chap2.html
    let (mut x_units, mut y_units) = pixels_to_font_units(x, y, units_per_em);
    x_units += x_min;
    y_units += y_min;
    let mut count: i32 = 0;
    // println!("Checking point ({:?}, {:?})", x_units, y_units);

    for segment in segments.into_iter() {
        count = count + segment.intersect(x_units, y_units, 1.0, 0.0);
    }

    return count % 2 == 1;

    // return count != 0;
}

fn get_glyph_size(height: f32, width: f32, units_per_em:f32) -> (i32, i32) {
    // Given the height and width of a glyph in font units, converts to the 
    // number of pixels (x, y) that need to be drawn on the display

    let ratio = get_ratio(units_per_em);

    let x_pixels = (width * ratio).ceil() as i32;
    let y_pixels = (height * ratio).ceil() as i32;
    return (x_pixels, y_pixels);
}

fn pixels_to_font_units(x: f32, y: f32, units_per_em:f32) -> (f32, f32) {
    let ratio = get_ratio(units_per_em);

    return (x / ratio, y / ratio);
}

fn font_to_pixel_units(x: f32, y: f32, units_per_em:f32) -> (f32, f32) {
    // Undoes the conversion done in the above function
    let ratio = get_ratio(units_per_em);

    return (x * ratio, y * ratio);
}

fn get_ratio(units_per_em: f32) -> f32 {
    // Scaling factor in formula, do not change
    const POINTS_PER_INCH: f32 = 72f32;
    
    // control output size via consts at top of file
    return POINT_SIZE * RESOLUTION / (POINTS_PER_INCH * units_per_em);
}