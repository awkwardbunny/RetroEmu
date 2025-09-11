use std::io;
use std::io::BufRead;
use std::sync::Arc;
use clap::Parser;
use env_logger;
use libretro::machine::AppleIIe;
use log::{LevelFilter, info, error};
use pixels::{Pixels, SurfaceTexture};
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use libretro::{Command, CommandParser};
use libretro::machine::apple_iie_e_display::{Display, DISPLAY_HEIGHT, DISPLAY_SCALE, DISPLAY_WIDTH};

const LOG_LEVEL: LevelFilter = LevelFilter::Info;

fn main() {
    env_logger::builder().filter_level(LOG_LEVEL).init();
    info!("Starting RetroEmu");

    let mut a2e = AppleIIe::new();
    a2e.reset();
    // a2e.run();

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(
            DISPLAY_WIDTH as u32 * DISPLAY_SCALE,
            DISPLAY_HEIGHT as u32 * DISPLAY_SCALE,
        );
        let window = WindowBuilder::new()
            .with_title("RetroEmu")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap();
        Arc::new(window)
    };
    window.set_resizable(false);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
        Pixels::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32, surface_texture)
            .expect("Failed to create pixels")
    };
    let display = Display::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    error!("Failed to resize window: {}", err);
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            if a2e.is_running() {
                a2e.step();
            } else {
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    info!(">");
                    let line = line.unwrap();
                    let parts = line.split_whitespace().collect::<Vec<_>>();
                    let p = CommandParser::parse_from(parts);
                    match &p.cmd {
                        Command::Step => a2e.step(),
                        Command::Run => {
                            a2e.run();
                        }
                    }
                    break;

                    // info!("USER: {}", line.unwrap());
                }
            }
            // world.update();
            window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            display.draw(pixels.frame_mut(), &a2e.get_memory());
            if let Err(err) = pixels.render() {
                error!("Failed to draw: {}", err);
                *control_flow = ControlFlow::Exit;
            }
        }
        _ => {}
    })
}
