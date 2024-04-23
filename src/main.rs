#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// use std::time;

use audio::DesktopAudioRecorder;

use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};

use rustfft::{FftPlanner, num_complex::Complex};

// const WIDTH: usize = 768;
// const HEIGHT: usize = 256;

const WIDTH: usize = 1536;
const HEIGHT: usize = 1024;

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
            scale: Scale::X1,
            scale_mode: ScaleMode::Center,
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
    let line_res = HEIGHT;


    let mut planner: FftPlanner<f32> = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);

    let mut complex_buf: Vec<Complex<f32>> = Vec::with_capacity(fft_size);

    let mut scan_pos = 0;

    let mut scan_ct = 0;

    // let mut profile: [u32; 256] = [0; 256];

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
                if block_ct > fft_size {

                    for i in 0..fft_size {
                        let x = block[i] as f32;
                        // print!("{} ", x);
                        complex_buf.push(Complex {re: x, im: 0.0});
                        if i == fft_size-1 {break};
                    }
                    // dbg!(complex_buf.len());

                    fft.process(&mut complex_buf);
                    complex_buf[0].re = 0.0;
                    // dbg!(complex_buf[0].re);
                    // dbg!(complex_buf[1].re);
                    // dbg!(complex_buf[2].re);

                    let rev_inter_cb = interleave_reversed_halves(&mut complex_buf);
                    // dbg!(rev_inter_cb.len());

                    let chunk_size = (fft_size/line_res)+0;
                    // dbg!(chunk_size);

                    let mut vals: Vec<u8> = Vec::with_capacity(line_res);

                    for (i, big_chunk) in rev_inter_cb.chunks_exact(fft_size/32).enumerate() {
                        let local_chunk_size = match i {
                            0..=7       => 8,
                            8..=15      => 7,
                            16..=17     => 6,
                            18..=19     => 5,
                            20..=21     => 4,
                            22..=26     => 3,
                            27..=30     => 2,
                            31          => 1,
                            o => {
                                panic!("Whattttt?????!? {}", o);
                            },
                        };
                        // dbg!(local_chunk_size);
                        for chunk in big_chunk.chunks_exact(local_chunk_size) {
                            let mut a = 0f32;
                            for n in chunk.iter() {
                                a += n.im;
                                a += n.re;
                            }
                            let b = a / chunk_size as f32;
                            let c = interpolate_f32_to_u8(b);
                            vals.push(c);
                        }
                        // println!("{:?}", vals.len());
                    }
                    let printable_vals = {
                        let len = vals.len();
                        if len < line_res {len} else {line_res}
                    };

                    // const COLOR1: Rgb = Rgb {r: 000, g: 000, b: 000};
                    // const COLOR2: Rgb = Rgb {r: 127, g: 000, b: 127};
                    // const COLOR3: Rgb = Rgb {r: 255, g: 000, b: 000};
                    // const COLOR4: Rgb = Rgb {r: 255, g: 230, b: 000};
                    // const COLOR5: Rgb = Rgb {r: 255, g: 255, b: 255};

                    const COLOR1: Rgb = Rgb {r: 000, g: 000, b: 000};
                    const COLOR2: Rgb = Rgb {r: 080, g: 080, b: 120};
                    const COLOR3: Rgb = Rgb {r: 155, g: 155, b: 200};
                    const COLOR4: Rgb = Rgb {r: 200, g: 200, b: 255};
                    const COLOR5: Rgb = Rgb {r: 255, g: 255, b: 255};

                    for (i, v) in vals[..printable_vals].iter().enumerate() {
                        // let lc = match *v {
                            // 000..=063  => {rgb_lerp(*v as f32, 0.0, 63.0, &COLOR1, &COLOR2)},
                            // 064..=127 => {rgb_lerp(*v as f32, 0.0, 63.0, &COLOR2, &COLOR3)},
                            // 128..=195 => {rgb_lerp(*v as f32, 0.0, 63.0, &COLOR3, &COLOR4)},
                            // 196..=255 => {rgb_lerp(*v as f32, 0.0, 63.0, &COLOR4, &COLOR5)},
                        // };
                        // let col = (255u32 << 24) | (lc.r << 16) | (lc.g << 8) | lc.b;

                        let a = *v as u32;
                        // // let col = (255u32 << 24) | (a << 16) | (a << 8) | a;
                        let col = (255u32 << 24) | (a << 16);

                        let pos = i * WIDTH + scan_pos;
                        buf[pos] = col;

                        // profile[*v as usize] += 1;
                    }

                    scan_pos += 1;
                    if scan_pos == WIDTH {scan_pos = 0}

                    complex_buf.clear();

                    block.clear();
                    block_ct = 0;

                    scan_ct += 1;
                    if scan_ct % 2 == 1 {
                        update = true;
                    }
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

