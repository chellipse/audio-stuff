use scratchpad::DesktopAudioRecorder;
use std::time;
use std::thread::sleep;

fn convert_s16le_to_f32(data: &[u8], num_channels: u8) -> Vec<f32> {
    let num_samples = data.len() / (num_channels as usize * 2);
    let mut samples = Vec::with_capacity(num_samples * num_channels as usize);

    for i in 0..num_samples {
        for channel in 0..num_channels {
            let offset = (i * num_channels as usize + channel as usize) * 2;
            let sample = i16::from_le_bytes([data[offset], data[offset + 1]]);
            samples.push(sample as f32 / std::i16::MAX as f32);
        }
    }

    samples
}

fn main() {
    let mut recorder = DesktopAudioRecorder::new("Experiment").unwrap();

    // let start = time::Instant::now();

    let ten_millis = time::Duration::from_millis(0);

    loop {
        match recorder.read_frame() {
            Ok(data) => {
                println!("{:?}", convert_s16le_to_f32(&data, 1));
                // for item in data.iter() {
                    // print!("{:4}", item);
                // }
                // println!();
            },
            Err(e) => eprintln!("{}", e)
        };
        sleep(ten_millis);

        // if Instant::now().duration_since(start).as_millis() > 5000 {
            // break;
        // }
    }
}
