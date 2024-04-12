#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::time;
use std::thread::sleep;

use audio::DesktopAudioRecorder;

use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};

const WIDTH: usize = 768;
const HEIGHT: usize = 256;
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

fn mean(slice: &[u8]) -> u32 {
    let mut col = 0u32;
    for item in slice {
        // if *item > 128 {
            col += *item as u32;
        // }
    }
    col / slice.len() as u32
}

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

fn main() {
    let mut recorder = DesktopAudioRecorder::new("Experiment").unwrap();

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: true,
            title: true,
            resize: false,
            scale: Scale::X2,
            scale_mode: ScaleMode::AspectRatioStretch,
            topmost: true,
            transparency: true,
            none: false,
        })
        .unwrap_or_else(|e| {
            panic!("{:#?}", e);
        });

    let frac = (1000*1000)/60;
    let interval = time::Duration::from_micros(frac);
    // window.limit_update_rate(Some(interval));
    // let interval = time::Duration::from_micros(0);

    let bg: u32 = 0xFF000000;
    let fg: u32 = 0xFFFFFFFF;

    let mut buf: [u32; AREA] = [bg; AREA];
    // let mut cords: Vec<usize> = Vec::with_capacity(WIDTH);

    // let mut last_pos: usize = 0;

    // let mut total: usize = 0;

    let mut block_ct = 0;
    // let mut block_col = 0u32;
    let mut block: Vec<u8> = Vec::with_capacity(65536);

    let mut last_update = [0usize; WIDTH];

    let mut update = false;



    while window.is_open() {
        // sleep(interval);
        match recorder.read_frame() {
            Ok(data) => {
                let len = data.len();
                // total += len;
                // print!("\nTotal: {:7} ", total);
                println!("Len: {:3} ", len);

                for item in data.iter() {
                    // if *item > 126 {
                    // block_col += *item as u32;
                    block.push(*item);
                    block_ct += 1;
                    // }
                }

                print!("Col: {:5} CT: {:4} ", block.len(), block_ct);
                let mut scan_pos = WIDTH-1;
                // sleep(interval);
                // if block_ct > WIDTH*3 {
                if block_ct > 1_230_000 {



                    // dbg!(block.len());
                    // for item in block.iter().rev() {
                    for chunk in block.chunks(3) {
                        let val = mean(chunk);
                        // print!("{} ", val);

                        let pos = val as usize * (WIDTH - scan_pos) as usize;
                        buf[pos] = fg;
                        buf[last_update[scan_pos]] = bg;
                        last_update[scan_pos] = pos;

                        if scan_pos == 0 {
                            break
                        }
                        scan_pos -= 1;
                    }
                    // println!("{:?}", last_update);

                    update = true;
                    block.clear();
                    block_ct = 0;
                }
                // println!();
            },
            Err(e) => eprintln!("{}", e)
        };
        if update {
            // println!(".");
            window.update_with_buffer(&buf, WIDTH, HEIGHT).unwrap();
            update = false;
        }
        if window.is_key_down(Key::Escape) {std::process::exit(0)}
        // sleep(interval);
        // println!(".");
    }
}

