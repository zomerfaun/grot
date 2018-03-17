extern crate failure;
extern crate floating_duration;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate sdl2;
#[macro_use]
extern crate structopt;

pub mod model;

use std::thread;
use std::time::{Duration, Instant};

use failure::{err_msg, Error};
use floating_duration::TimeFormat;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::FullscreenType;
use structopt::StructOpt;

use model::Model;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(short = "f", long = "fullscreen", help = "Run fullscreen at desktop resolution")]
    pub fullscreen: bool,
    #[structopt(short = "F", long = "framerate", default_value = "60",
                help = "Limit frame rate to at most <fps>, or 0 for unlimited")]
    pub fps: u32,
    #[structopt(short = "v", long = "vsync", help = "Enable vsync")] pub vsync: bool,
}

/// Runs the game.
pub fn run(options: &Options) -> Result<(), Error> {
    debug!("Running game with {:?}", options);
    let sdl = sdl2::init().map_err(err_msg)?;
    let video = sdl.video().map_err(err_msg)?;
    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    let mut window_builder = video.window("Grot", 640, 480);
    if options.fullscreen {
        window_builder.fullscreen_desktop();
    }
    let window = window_builder.build()?;
    let mut canvas_builder = window.into_canvas();
    if options.vsync {
        canvas_builder = canvas_builder.present_vsync();
    }
    let mut canvas = canvas_builder.build()?;

    let mut model = Model::new(150);

    let limit_fps = options.fps != 0;
    let frame_duration = Duration::from_secs(1)
        .checked_div(options.fps)
        .unwrap_or_default();

    debug!("Running main loop");
    let mut last_update_time = Instant::now();
    loop {
        trace!("Start new frame");
        let frame_started = Instant::now();
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

        // Update model with the time passed since the previous update
        let update_time = Instant::now();
        let time_passed = update_time - last_update_time;
        trace!("Time passed for model update: {}", TimeFormat(time_passed));
        model.update(time_passed);
        last_update_time = update_time;

        model.render(&mut canvas)?;
        canvas.present();

        let frame_finished = Instant::now();
        let frame_process_time = frame_finished - frame_started;
        trace!("Processing frame took {}", TimeFormat(frame_process_time));
        if limit_fps {
            if frame_process_time < frame_duration {
                let sleep_duration = frame_duration - frame_process_time;
                trace!("Frame is {} early; sleeping", TimeFormat(sleep_duration));
                thread::sleep(sleep_duration);
            } else {
                let lateness = frame_process_time - frame_duration;
                trace!("Frame is {} late", TimeFormat(lateness));
            }
        }
    }
}

fn main() {
    pretty_env_logger::init();
    let options = Options::from_args();
    if let Err(error) = run(&options) {
        eprintln!("Error: {}", error);
        for cause in error.causes().skip(1) {
            eprintln!("Cause: {}", cause);
        }
        eprintln!("{}", error.backtrace());
        std::process::exit(1);
    }
}
