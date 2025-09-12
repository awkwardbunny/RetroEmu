mod config;

use crate::config::Config;
use crate::config::Machines::AppleIiE;
use libretro::machine::apple_iie_e_display::{
    DISPLAY_HEIGHT, DISPLAY_SCALE, DISPLAY_WIDTH, Display,
};
use libretro::machine::{AppleIIe, Machine};
use libretro::{DisplayCommand, EmulatorCommand};
use log::{error, info};
use pixels::{Pixels, SurfaceTexture};
use std::io::Write;
use std::sync::{Arc, mpsc};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;

fn start_emulation_thread(
    config: Config,
    cmd_rx: mpsc::Receiver<EmulatorCommand>,
    gui_tx: mpsc::Sender<DisplayCommand>,
) {
    let (cycle_tx, cycle_rx) = mpsc::channel::<()>();
    let (draw_tx, draw_rx) = mpsc::channel::<()>();
    let gui2 = gui_tx.clone();

    thread::spawn(move || {
        let mut mach: Box<dyn Machine> = match config.machine {
            AppleIiE {
                ref disk1,
                ref disk2,
                freq,
            } => {
                let mut x = AppleIIe::new(gui_tx);
                if let Some(disk1) = disk1 {
                    x.load_disk1(config.get_file(disk1).expect("Failed to load disk1"));
                }
                if let Some(disk2) = disk2 {
                    x.load_disk1(config.get_file(disk2).expect("Failed to load disk1"));
                }
                thread::spawn(move || {
                    let period = 1000000000 / freq;
                    info!("Freq: {}khz", freq);
                    info!("Period: {}ns", period);
                    loop {
                        sleep(Duration::from_nanos(period as u64));
                        cycle_tx.send(()).unwrap();
                    }
                });
                thread::spawn(move || {
                    info!("Video Freq: 1khz");
                    loop {
                        sleep(Duration::from_millis(1));
                        draw_tx.send(()).unwrap();
                    }
                });
                Box::new(x)
            }
        };

        let mut is_running = false;

        loop {
            while let Ok(()) = cycle_rx.try_recv() {
                if is_running {
                    mach.as_mut().cycle();
                }
            }
            while let Ok(()) = draw_rx.try_recv() {
                gui2.send(DisplayCommand::Redraw).unwrap();
            }
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    EmulatorCommand::Cycle => mach.as_mut().cycle(),
                    EmulatorCommand::Step => mach.as_mut().step(),
                    EmulatorCommand::Run => is_running = true,
                    EmulatorCommand::Stop => is_running = false,
                    EmulatorCommand::Reset => mach.as_mut().reset(),
                };
            }
        }
    });
}

fn start_terminal_thread(
    cmd_tx: mpsc::Sender<EmulatorCommand>,
    gui_tx: mpsc::Sender<DisplayCommand>,
) {
    thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());

        let mut is_running = false;

        let (ctrlc_tx, ctrlc_rx) = mpsc::channel::<()>();
        ctrlc::set_handler(move || {
            ctrlc_tx.send(()).expect("Error sending ctrlc");
        })
        .unwrap();

        loop {
            if let Ok(_) = ctrlc_rx.try_recv() {
                is_running = false;
                cmd_tx.send(EmulatorCommand::Stop).unwrap();
            }

            if !is_running {
                print!("retro> ");
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                if let Ok(n) = reader.read_line(&mut input) {
                    if n == 0 {
                        break;
                    }
                    match input.trim() {
                        "continue" | "c" => {
                            is_running = true;
                            cmd_tx.send(EmulatorCommand::Run).unwrap()
                        }
                        "step" | "s" => cmd_tx.send(EmulatorCommand::Step).unwrap(),
                        "cycle" => cmd_tx.send(EmulatorCommand::Cycle).unwrap(),
                        "reset" | "r" => cmd_tx.send(EmulatorCommand::Reset).unwrap(),
                        "exit" | "quit" | "q" => break,
                        "" => {}
                        _ => println!("Unknown command!"),
                    }
                }
            }
        }
        gui_tx.send(DisplayCommand::Exit(0)).unwrap();
    });
}

fn start_display_thread(gui_rx: mpsc::Receiver<DisplayCommand>) {
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

    event_loop.run(move |event, _, control_flow| {
        while let Ok(cmd) = gui_rx.try_recv() {
            match cmd {
                DisplayCommand::Write(addr, data) => {
                    display.write_text(pixels.frame_mut(), addr, data as char);
                }
                DisplayCommand::Redraw => {
                    window.request_redraw();
                }
                DisplayCommand::Exit(_) => {
                    *control_flow = ControlFlow::Exit;
                }
            };
        }
        match event {
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
                // window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if let Err(err) = pixels.render() {
                    error!("Failed to draw: {}", err);
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    })
}

fn main() {
    let config = Config::load();

    info!("Starting RetroEmu");

    let (cmd_tx, cmd_rx) = mpsc::channel::<EmulatorCommand>();
    let (gui_tx, gui_rx) = mpsc::channel::<DisplayCommand>();
    let gui_tx_2 = gui_tx.clone();

    start_emulation_thread(config, cmd_rx, gui_tx);
    start_terminal_thread(cmd_tx, gui_tx_2);
    start_display_thread(gui_rx);
}
