/* Analyzes rendering performance based on the metrics defined
in the project proposal. */

use ttf_parser::Face;
use ttf_parser::Rect;
use std::fs;

use crate::segment::Segment;

use crate::outline::extract_outline;

use rand::prelude::*;

// Pixel density of target display
const RESOLUTION: f32 = 144f32;
// Font size for output
const POINT_SIZE: f32 = 192f32;

#[derive(Debug)]
pub struct Metrics {
    pub none: f32,
    pub two_by_two: f32,
    pub three_by_three: f32,
    pub four_by_four: f32,
    pub samples: i32
}

/*
pub fn get_supersample_score(test_character: char, test_samples: i32, test_pixels_samples: i32) -> f32
{
    let file = fs::read(FONT_PATH).unwrap();
    let face = match Face::parse(&file, 0) {
        Ok(f) => f,
        Err(e) => {
            eprint!("Error: {}.", e);
            std::process::exit(1);
        }
    };
    let (segments, bbox) = extract_outline(&face, test_character);
    let units_per_em = face.units_per_em() as f32;

    // Step 2: get bounding box for character
    let bbox_unwrapped: Rect = bbox.unwrap();
    let x_min = bbox_unwrapped.x_min as f32;
    // let x_max = bbox_unwrapped.x_max as f32;
    let y_min = bbox_unwrapped.y_min as f32;
    // let y_max = bbox_unwrapped.y_max as f32;
    let width = bbox_unwrapped.width() as f32;
    let height = bbox_unwrapped.height() as f32;
    let (x_pixels, y_pixels) = get_glyph_size(height, width, units_per_em);

    // Step 3: Take TEST_SAMPLES samples from character
    let mut ratio_error = 0f32;

    for _ in 0..test_samples {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..x_pixels) as f32;
        let y = rng.gen_range(0..y_pixels) as f32;

        let mut color_factor_ratio = 0f32;
        let mut within_samples = 0i32;

        for x_subsample in 0..test_pixels_samples {
            let x_offset: f32 = rng.gen();
            let y_offset: f32 = rng.gen();
            let sample = should_draw_point(x + x_offset, y + y_offset, x_min, y_min, units_per_em, &segments);
            within_samples += if sample {1} else {0};
        }

        let mut sampled_ratio = (within_samples as f32) / (test_pixels_samples as f32);

        let x_baseline = x as f32;
        let y_baseline = y as f32;
        let mut color_factor = 0;
        let division_factor = 2;

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

        color_factor_ratio = (color_factor as f32)/ (division_factor as f32 *division_factor as f32);
        sampled_ratio = (within_samples as f32) / (test_samples as f32);

        ratio_error += f32::abs(color_factor_ratio - sampled_ratio);
    }
    // TODO: calculate results for two_by_two and three_by_three

    return 1f32 - (ratio_error / test_samples as f32);
}
*/

pub fn get_render_score(test_character: char, font_path:&str, test_samples: i32) -> Metrics {
    /* Takes TEST_SAMPLES samples from within the char's bbox and returns
    the percentage of them that are correctly classified */
    // Step 1: extract outline from font file
    let file = fs::read(font_path).unwrap();
    let face = match Face::parse(&file, 0) {
        Ok(f) => f,
        Err(e) => {
            eprint!("Error: {}.", e);
            std::process::exit(1);
        }
    };
    let (segments, bbox) = extract_outline(&face, test_character);
    let units_per_em = face.units_per_em() as f32;

    // Step 2: get bounding box for character
    let bbox_unwrapped: Rect = bbox.unwrap();
    let x_min = bbox_unwrapped.x_min as f32;
    // let x_max = bbox_unwrapped.x_max as f32;
    let y_min = bbox_unwrapped.y_min as f32;
    // let y_max = bbox_unwrapped.y_max as f32;
    let width = bbox_unwrapped.width() as f32;
    let height = bbox_unwrapped.height() as f32;
    let (x_pixels, y_pixels) = get_glyph_size(height, width, units_per_em);

    // Step 3: Take TEST_SAMPLES samples from character
    //let mut valid_samples = 0;
    let mut correct_samples_default = 0;
    let mut correct_samples_2x2 = 0;
    let mut correct_samples_3x3 = 0;
    let mut correct_samples_4x4 = 0;
    let mut rng = rand::thread_rng();
    for _ in 0..test_samples {
        // Step 3a: Get a pixel to test
        let x = rng.gen_range(0..x_pixels) as f32;
        let y = rng.gen_range(0..y_pixels) as f32;
        // Step 3b: Get offset for test
        let x_offset: f32 = rng.gen();
        let y_offset: f32 = rng.gen();

        let (x_offset_2x2, y_offset_2x2) = get_subpixel(x_offset, y_offset, 2);
        let (x_offset_3x3, y_offset_3x3) = get_subpixel(x_offset, y_offset, 3);
        let (x_offset_4x4, y_offset_4x4) = get_subpixel(x_offset, y_offset, 4);
        // Step 3c: Check whether the pixel is filled in at (1) the random sample position
        // and (2) the corresponding sample point with default sampling, 2x2 supersampling,
        // and 3x3 supersampling
        let sample_default = should_draw_point(x + 0.5f32, y + 0.5f32, x_min, y_min, units_per_em, &segments);
        let sample_2x2 = should_draw_point(x + x_offset_2x2, y + y_offset_2x2, x_min, y_min, units_per_em, &segments);
        let sample_3x3 = should_draw_point(x + x_offset_3x3, y + y_offset_3x3, x_min, y_min, units_per_em, &segments);
        let sample_4x4 = should_draw_point(x + x_offset_4x4, y + y_offset_4x4, x_min, y_min, units_per_em, &segments);
        let sample_random = should_draw_point(x + x_offset, y + y_offset, x_min, y_min, units_per_em, &segments);

        correct_samples_default += if sample_default == sample_random {1} else {0};
        correct_samples_2x2 += if sample_2x2 == sample_random {1} else {0};
        correct_samples_3x3 += if sample_3x3 == sample_random {1} else {0};
        correct_samples_4x4 += if sample_4x4 == sample_random {1} else {0};
    }
    // TODO: calculate results for two_by_two and three_by_three
    return Metrics{
        none: correct_samples_default as f32 / test_samples as f32,
        two_by_two: correct_samples_2x2 as f32 / test_samples as f32,
        three_by_three: correct_samples_3x3 as f32 / test_samples as f32,
        four_by_four: correct_samples_4x4 as f32 / test_samples as f32,
        samples: test_samples
    };

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

fn get_ratio(units_per_em: f32) -> f32 {
    // Scaling factor in formula, do not change
    const POINTS_PER_INCH: f32 = 72f32;
    
    // control output size via consts at top of file
    return POINT_SIZE * RESOLUTION / (POINTS_PER_INCH * units_per_em);
}

fn get_subpixel(x_offset: f32, y_offset: f32, n: i32) -> (f32, f32) {
    // Step 1: find which subpixel our point is in
    let factor: f32 = 1f32 / n as f32;
    let x_subpixel: i32 = (x_offset / factor).floor() as i32;
    let y_subpixel: i32 = (y_offset / factor).floor() as i32;

    let x_subpixel_offset = ((x_subpixel as f32)/(n as f32)) + (0.5/(n as f32));
    let y_subpixel_offset = ((y_subpixel as f32)/(n as f32)) + (0.5/(n as f32));
    
    return (x_subpixel_offset, y_subpixel_offset);
}