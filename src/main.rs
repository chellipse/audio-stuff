use std::time;
use std::thread::sleep;

use scratchpad::DesktopAudioRecorder;

use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};

const WIDTH: usize = 900;
const HEIGHT: usize = 300;
const AREA: usize = WIDTH * HEIGHT;

fn interpolate_i16(value: i16) -> i16 {
    let input_min = i16::MIN;
    let input_max = i16::MAX;
    let output_min = 0_i16;
    let output_max = HEIGHT as i16;

    let value = value as f32;
    let input_min = input_min as f32;
    let input_max = input_max as f32;
    let output_min = output_min as f32;
    let output_max = output_max as f32;

    let interpolated = ((value - input_min) / (input_max - input_min)) * (output_max - output_min) + output_min;
    interpolated.round() as i16
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

    // let frac = (1000*1000)/60;
    // let interval = time::Duration::from_micros(frac);
    // window.limit_update_rate(Some(interval));
    let interval = time::Duration::from_micros(0);

    let bg: u32 = 0xBB000000;
    let fg: u32 = 0xFFFFFFFF;

    let mut buf: [u32; AREA] = [bg; AREA];
    let mut cords: Vec<usize> = Vec::with_capacity(WIDTH);

    let mut last_pos: usize = 0;

    while window.is_open() {
        match recorder.read_frame() {
            Ok(data) => {
                // println!("{:?}", data);
                for item in data.iter().rev() {
                    let val = interpolate_i16(*item);
                    // print!("{:6}", item);
                    // print!("{} ", val);

                    dbg!(cords.len());
                    for (i, c) in cords.iter_mut().rev().enumerate() {
                        // dbg!(&i, &c);
                        if i > (WIDTH-1) {
                            // dbg!(&i);
                            break
                        }
                        // dbg!(*c);
                        if *c == 0 {break}
                        buf[*c] = bg;
                        buf[*c-1] = fg;
                        *c = *c-1;
                    }

                    let pos = val as usize * (WIDTH-0) + (WIDTH-1) as usize;
                    // dbg!(pos, val);
                    cords.push(pos);
                    buf[pos] = fg;
                }
                // println!();
            },
            Err(e) => eprintln!("{}", e)
        };
        window.update_with_buffer(&buf, WIDTH, HEIGHT).unwrap();
        if window.is_key_down(Key::Escape) {std::process::exit(0)}
        sleep(interval);
    }
}

