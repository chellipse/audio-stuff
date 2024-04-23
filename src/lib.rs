use std::cell::RefCell;
use std::rc::Rc;

use libpulse_binding as pulse;
use pulse::callbacks::ListResult;
use pulse::context::{Context, FlagSet as ContextFlagSet, State as ContextState};
use pulse::def::BufferAttr;
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::operation::State as OperationState;
use pulse::sample::Spec;
use pulse::stream::{FlagSet as StreamFlagSet, PeekResult, Stream};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetMonitorError {
    #[error("Mainloop terminated unexpectedly")]
    MainLoopExited,

    #[error("No default output found")]
    NoDefaultSink,

    #[error("No monitor found for default output")]
    NoDefaultMonitor,
}

fn get_default_sink_monitor(
    context: &mut Context,
    mainloop: &mut Mainloop,
) -> Result<String, GetMonitorError> {
    use GetMonitorError::*;
    let default_sink_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    {
        let default_sink_name = Rc::clone(&default_sink_name);
        context.introspect().get_server_info(move |server_info| {
            let name = &*match server_info.default_sink_name.clone() {
                Some(name) => name,
                None => return,
            };
            default_sink_name.replace(Some(name.into()));
        });
    }

    // Wait for default_sink_name to be set.
    loop {
        match mainloop.iterate(true) {
            IterateResult::Success(..) => {}
            _ => return Err(MainLoopExited),
        }

        if default_sink_name.borrow().is_some() {
            break;
        }
    }

    let default_sink_name = match default_sink_name.borrow().clone() {
        Some(name) => name,
        None => return Err(NoDefaultSink),
    };

    let default_sink_monitor_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let default_sink_monitor_name_op;
    {
        let default_sink_monitor_name = Rc::clone(&default_sink_monitor_name);
        default_sink_monitor_name_op =
            context
                .introspect()
                .get_sink_info_by_name(&default_sink_name, move |sink_info| {
                    match sink_info {
                        ListResult::Item(sink_info) => {
                            if default_sink_monitor_name.borrow().is_some() {
                                // Ignore subsequent results.
                                return;
                            }

                            let name = match sink_info.monitor_source_name.clone() {
                                Some(name) => name.to_string(),
                                None => return,
                            };

                            default_sink_monitor_name.replace(Some(name));
                        }
                        _ => (),
                    }
                });
    }

    loop {
        match mainloop.iterate(true) {
            IterateResult::Success(..) => {}
            _ => return Err(MainLoopExited),
        }

        if default_sink_monitor_name.borrow().is_some() {
            break;
        }

        if default_sink_monitor_name_op.get_state() == OperationState::Done {
            // Callback errored
            return Err(NoDefaultMonitor);
        }
    }

    // Unwrap here is okay because we asserted the existance in the loop above.
    let default_sink_monitor_name = default_sink_monitor_name.borrow().clone().unwrap();
    Ok(default_sink_monitor_name)
}

/// The primary struct responsible for capturing audio data.
pub struct DesktopAudioRecorder {
    mainloop: Mainloop,
    context: Context,
    stream: Stream
}

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("Failed to create main loop")]
    MainLoopCreationFail,

    #[error("Failed to create context")]
    ContextCreateFail,

    #[error("Failed to initiate context connection")]
    ConnectionInitFail(#[from] pulse::error::PAErr),

    #[error("Failed to connect context")]
    ConnectionFail,

    #[error("Failed to get monitor for default output")]
    MonitorFail(#[from] GetMonitorError),

    #[error("Main loop exited unexpectedly")]
    MainLoopExited,
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Main loop exited unexpectedly")]
    MainLoopExited,

    #[error("Error reading stream")]
    StreamReadError(#[from] pulse::error::PAErr)
}

impl DesktopAudioRecorder {
    /// Create a new recorder.
    pub fn new(application_name: &str) -> Result<Self, CreateError> {
        use CreateError::*;

        let mut mainloop = Mainloop::new().ok_or(MainLoopCreationFail)?;
        let mut context = Context::new(&mainloop, application_name).ok_or(ContextCreateFail)?;
        context.connect(None, ContextFlagSet::NOFLAGS, None)?;
        // context.connect(Some("3"), ContextFlagSet::NOFLAGS, None)?;
        // context.connect(Some("alsa_output.usb-Framework_Audio_Expansion_Card-00.analog-stereo"), ContextFlagSet::NOFLAGS, None)?;

        loop {
            match mainloop.iterate(true) {
                IterateResult::Err(_) | IterateResult::Quit(_) => {
                    eprintln!("Loop exited");
                    return Err(MainLoopExited);
                }
                IterateResult::Success(_) => {}
            }

            match context.get_state() {
                ContextState::Ready => {
                    println!("Ready");
                    break;
                }
                ContextState::Failed | ContextState::Terminated => {
                    eprintln!("Failed to connect");
                    return Err(ConnectionFail);
                }
                _ => {}
            }
        }

        // println!(".");
        let monitor_source_name = get_default_sink_monitor(&mut context, &mut mainloop)?;
        // dbg!(&monitor_source_name);
        let sample_spec = Spec {
            channels: 1,
            format: pulse::sample::Format::U8,
            rate: 48000
            // rate: 41000
            // rate: 24000
            // rate: 4096
            // rate: 1800
            // rate: 120
        };

        assert!(sample_spec.is_valid());

        let mut stream = Stream::new(
            &mut context,
            "Epic experiment stream",
            &sample_spec,
            None
        ).unwrap();

        let val = u32::max_value();
        // let val = 0u32;
        // let val = u16::max_value() as u32;
        // let val = 4096;
        // let val = 768;

        let flags = StreamFlagSet::ADJUST_LATENCY;

        stream.connect_record(
            Some(&monitor_source_name),
            Some(&BufferAttr {
                maxlength: val,
                tlength: val,
                prebuf: 0,
                minreq: val,
                fragsize: val
            }),
            flags
        ).unwrap();

        // println!(".");
        // stream.connect_playback(
            // Some(&monitor_source_name),
            // Some(&BufferAttr {
                // maxlength: val,
                // tlength: val,
                // prebuf: 0,
                // minreq: val,
                // fragsize: val
            // }),
            // StreamFlagSet::NOFLAGS,
            // None,
            // None
        // ).unwrap();

        // println!(".");
        Ok(DesktopAudioRecorder { mainloop, context, stream })
    }

    /// Read some data from the stream, make sure to call this in a loop.
    pub fn read_frame(&mut self) -> Result<&[u8], ReadError> {
        // println!(".");
        use ReadError::*;

        // if let Some(size) = self.stream.readable_size() {
            // if size > 4096 {
                // print!("R: {:3} ", size)
            // }
        // };

        loop {
            match self.mainloop.iterate(true) {
                IterateResult::Success(..) => {},
                _ => return Err(MainLoopExited)
            };

            match self.stream.get_state() {
                pulse::stream::State::Ready => {},
                _o => {
                    dbg!(_o);
                    continue;
                }
            }
            // loop {
                // match self.stream.readable_size()
            // }
            let peek_result = self.stream.peek()?;
            match peek_result {
                PeekResult::Data(data) => {
                    // println!("DL: {}", data.len());

                    // There is probably a nicer way to do this.
                    // let parsed_data: Vec<i16> = data.into_iter()
                    // .step_by(4)
                    // .enumerate()
                    // .map(|(i, _)| {
                    // i16::from_le_bytes(data[i*2..(i+1)*2].try_into().unwrap())
                    // })
                    // .collect();

                    // let parsed_data: Vec<u8> = Vec::from(data);

                    // let parsed_data: Vec<i16> = unsafe { std::mem::transmute(data)};

                    // let parsed_data = data as *const [i16];

                    self.stream.discard().unwrap();
                    // self.stream.flush(None);
                    // self.stream.flush(None);

                    return Ok(data);
                },
                PeekResult::Empty => {
                    // println!("Empty!");
                },
                PeekResult::Hole(..) => {
                    // println!("Hole!");
                    self.stream.discard().unwrap();
                }
            }
        };
    }

    /// Do cleanup so that the program doesn't segfault. Automatically called when self goes out of
    /// scope
    pub fn quit(&mut self) {
        self.mainloop.quit(pulse::def::Retval(0));
        self.context.disconnect();
        let _ = self.stream.disconnect();
    }
}

impl Drop for DesktopAudioRecorder {
    fn drop(&mut self) {
        self.quit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn record_5_seconds() {
        use std::time::Instant;
        let mut recorder = DesktopAudioRecorder::new("Experiment").unwrap();

        let start = Instant::now();

        loop {
            match recorder.read_frame() {
                Ok(data) => println!("{:?}", data),
                Err(e) => eprintln!("{}", e)
            };

            if Instant::now().duration_since(start).as_millis() > 5000 {
                break;
            }
        }
    }
}
