extern crate failure;
extern crate floating_duration;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate sdl2;

pub mod math;
pub mod model;

use std::thread;
use std::time::{Duration, Instant};

use failure::{err_msg, Error};
use floating_duration::TimeFormat;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::FullscreenType;

use model::Model;

/// Runs the game.
pub fn run() -> Result<(), Error> {
    let sdl = sdl2::init().map_err(err_msg)?;
    let video = sdl.video().map_err(err_msg)?;
    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    let window = video.window("Grot", 640, 480).build()?;
    let mut canvas = window.into_canvas().build()?;

    let mut model = Model::new(60);

    let frame_duration = Duration::from_secs(1) / 60;
    let mut frame_start_time = Instant::now();
    let mut frame_deadline = frame_start_time + frame_duration;

    debug!("Running main loop");
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    debug!("Quitting");
                    return Ok(());
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    repeat: false,
                    ..
                } => {
                    // Toggle fullscreen state
                    let window = canvas.window_mut();
                    let new_fullscreen_state = match window.fullscreen_state() {
                        FullscreenType::Off => FullscreenType::Desktop,
                        _ => FullscreenType::Off,
                    };
                    debug!("New fullscreen state: {:?}", new_fullscreen_state);
                    window
                        .set_fullscreen(new_fullscreen_state)
                        .map_err(err_msg)?;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    repeat: false,
                    ..
                } => model.left_pressed(),
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => model.left_released(),
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    repeat: false,
                    ..
                } => model.right_pressed(),
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => model.right_released(),

                _ => trace!("Unhandled event of type {:?}", event),
            }
        }

        model.update(frame_duration);
        model.render(&mut canvas)?;
        canvas.present();

        let now = Instant::now();
        let process_duration = now - frame_start_time;
        if now < frame_deadline {
            let sleep_duration = frame_deadline - now;
            trace!(
                "Processing frame took {}, {} ahead of deadline",
                TimeFormat(process_duration),
                TimeFormat(sleep_duration)
            );
            thread::sleep(sleep_duration);
        } else {
            let lateness = now - frame_deadline;
            trace!(
                "Processing frame took {}, {} behind deadline",
                TimeFormat(process_duration),
                TimeFormat(lateness)
            );
            if lateness > Duration::from_secs(1) {
                warn!("Frame is {} late; resetting deadline", TimeFormat(lateness));
                frame_deadline = now;
            }
        }
        frame_start_time = Instant::now();
        frame_deadline += frame_duration;
    }
}

fn main() {
    pretty_env_logger::init();
    if let Err(error) = run() {
        eprintln!("Error: {}", error);
        for cause in error.causes().skip(1) {
            eprintln!("Cause: {}", cause);
        }
        eprintln!("{}", error.backtrace());
        std::process::exit(1);
    }
}
