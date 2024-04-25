#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::time;

use audio::DesktopAudioRecorder;

use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};

use rustfft::{FftPlanner, num_complex::Complex};

// const WIDTH: usize = 768;
// const HEIGHT: usize = 256;

const WIDTH: usize = 16;
const HEIGHT: usize = 16;

const AREA: usize = WIDTH * HEIGHT;

// fn interpolate_i16(value: i16) -> i16 {
    // let input_min = i16::MIN/2;
    // let input_max = i16::MAX/2;
    // let output_min = 0_i16;
    // let output_max = HEIGHT as i16;

    // let value = value as f32;
    // let input_min = input_min as f32;
    // let input_max = input_max as f32;
    // let output_min = output_min as f32;
    // let output_max = output_max as f32;

    // let interpolated = ((value - input_min) / (input_max - input_min)) * (output_max - output_min) + output_min;
    // interpolated.round() as i16
// }

// fn mean(slice: &[u8]) -> u32 {
    // let mut col = 0u32;
    // for item in slice {
        // // if *item > 128 {
            // col += *item as u32;
        // // }
    // }
    // col / slice.len() as u32
// }

#[allow(dead_code)]
fn save_to_file(file_name: &str, data: &[u8]) {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = match File::create(file_name) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("creating {} resulted in {}", file_name, e);
            std::process::exit(1);
        }
    };
    match file.write_all(data) {
        Ok(r) => {
            println!("Success! `{:?}`", r);
        }
        Err(e) => {
            eprintln!("writing to {} returned {}", file_name, e);
        }
    };
}

fn interpolate_f32_to_u8(value: f32) -> u8 {
    let min = 0.0;
    // let max = 8192.0f32;
    // let max = 4096.0f32;
    let max = 2048.0f32;

    if value <= min {
        return 0;
    }
    if value >= max {
        return 255;
    }

    let range = max - min;
    let normalized = (value - min) / range;
    let interpolated = normalized * 255.0;

    interpolated.round() as u8
}

fn interleave_reversed_halves(vec: &mut Vec<Complex<f32>>) -> Vec<Complex<f32>> {
    let len = vec.len();
    let mid = len / 2;
    
    // Split the vector into two halves
    let (first_half, second_half) = vec.split_at_mut(mid);

    // Reverse the order of the second half
    first_half.reverse();

    // Interleave the two halves
    let mut result = Vec::with_capacity(len);
    let mut i = 0;
    let mut j = 0;
    while i < first_half.len() && j < second_half.len() {
        result.push(first_half[i]);
        result.push(second_half[j]);
        i += 1;
        j += 1;
    }

    result
}

// linearly interpolates A's position between B and C to D and E
fn lerp(a: f32, b: f32, c: f32, d: f32, e: f32) -> f32 {
    (a - b) * (e - d) / (c - b) + d
}

struct Rgb {
    r: u32,
    g: u32,
    b: u32,
}


// same as lerp() but the output values are Rgb structs
fn rgb_lerp(x: f32, y: f32, z: f32, color1: &Rgb, color2: &Rgb) -> Rgb {
    Rgb {
        r: lerp(x, y, z, color1.r as f32, color2.r as f32) as u32,
        g: lerp(x, y, z, color1.g as f32, color2.g as f32) as u32,
        b: lerp(x, y, z, color1.b as f32, color2.b as f32) as u32,
    }
}

fn main() {
    let mut recorder = DesktopAudioRecorder::new("Experiment").unwrap();

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: Scale::X32,
            scale_mode: ScaleMode::Stretch,
            topmost: true,
            transparency: false,
            none: false,
        })
        .unwrap_or_else(|e| {
            panic!("{:#?}", e);
        });

    // let frac = (1000*1000)/60;
    // let interval = time::Duration::from_micros(frac);
    // window.limit_update_rate(Some(interval));
    // let interval = time::Duration::from_micros(0);

    let bg: u32 = 0xFF000000;
    // let fg: u32 = 0xFFFFFFFF;

    let mut buf: [u32; AREA] = [bg; AREA];
    // let mut cords: Vec<usize> = Vec::with_capacity(WIDTH);

    // let mut last_pos: usize = 0;

    // let mut total: usize = 0;

    let mut block_ct = 0;
    // let mut block_col = 0u32;
    let mut block: Vec<u8> = Vec::with_capacity(65536);

    // let mut last_update = [0usize; WIDTH];

    let mut update = false;


    let fft_size = 4096;
    // let fft_size = 2048;
    // let fft_size = 1024;
    // let fft_size = 512;
    // let fft_size = 256;
    let new_data_size = 512;
    let line_res = HEIGHT;


    let mut planner: FftPlanner<f32> = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);

    let mut complex_buf: Vec<Complex<f32>> = vec![Complex {re: 0.0, im: 0.0}; fft_size];

    let mut scan_pos = 0;

    let mut scan_ct = 0;

    // let mut profile: [u32; 256] = [0; 256];
    
    let mut max = 0.0;
    let mut max2 = 0.0;

    let mut values = vec![0; fft_size];

    while window.is_open() {
        match recorder.read_frame() {
            Ok(data) => {
                // let len = data.len();
                // println!("Len: {:3} ", len);

                for item in data.iter() {
                    block.push(*item);
                    block_ct += 1;
                }

                // print!("Col: {:5} CT: {:4} ", block.len(), block_ct);
                if block_ct > new_data_size {
                    values = values[new_data_size..].to_vec();
                    assert!(block.len() >= new_data_size);
                    values.extend_from_slice(&block[0..new_data_size]);
                    // dbg!(complex_buf.len());
                    
                    let mut complex_buf = values.iter()
                        .map(|v| Complex {re: *v as f32, im: 0.0})
                        .collect::<Vec<_>>();

                    fft.process(&mut complex_buf);

                    let sum = complex_buf.iter()
                        .map(|v| v.norm())
                        .sum::<f32>();

                    let mean: f32 = sum / complex_buf.len() as f32;

                    // println!("{}", mean);

                    if max < mean {max = mean};
                    let a = (mean / max * 255.0) as u32;
                    println!("{}", a);
                    // let col = (255u32 << 24) | (a << 16);
                    let col = a;

                    let mean2 = values.iter()
                        .zip(values[1..].iter())
                        .map(|(d1, d2)| (*d1 as i32 - *d2 as i32).abs())
                        .sum::<i32>() as f32;
                    if max2 < mean2 {max2 = mean2};
                    let b = (mean2/max2 * 255.0) as u8;

                    for y in 0..HEIGHT/2 {
                        let pos = y * WIDTH;
                        for x in 0..WIDTH {
                            buf[pos + x] = col;
                        }
                    }
                    for y in HEIGHT/2..HEIGHT {
                        let pos = y * WIDTH;
                        for x in 0..WIDTH {
                            buf[pos + x] = (b as u32) << 8;
                        }
                    }

                    complex_buf.clear();

                    block.clear();
                    block_ct = 0;

                    scan_ct += 1;
                    // if scan_ct % 2 == 1 {
                        update = true;
                    // }
                }
            },
            Err(e) => eprintln!("{}", e)
        };
        if update {
            window.update_with_buffer(&buf, WIDTH, HEIGHT).unwrap();
            update = false;
            // println!("------------------------");
            // for chunk in profile.chunks(16) {
                // for item in chunk {
                    // print!("{:5} ", item);
                // }
                // println!();
            // }
        }
        if window.is_key_down(Key::Escape) {std::process::exit(0)}
    }
}

