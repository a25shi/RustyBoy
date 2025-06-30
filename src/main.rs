mod cartridge_header;
mod memory;
mod cpu;
mod cartridge;
mod timer;
mod motherboard;
mod screen;
mod joypad;

use std::fs;
use std::any::type_name;
use std::fmt::format;
use std::time::{Duration, Instant};
use sdl2::event::Event;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use crate::cpu::CPU;

fn main() {
    // let file: &str = "./roms/blargg tests/cpu_instrs/individual/01-special.gb";
    // Getting cartridge header data and game data
    let bytes: Vec<u8> = fs::read("./roms/kirby.gb").unwrap();
    let header = cartridge_header::CartridgeHeader::from_bytes(&bytes);
    println!("{:?}", header);

    // Cpu init
    let mut cpu = CPU::new(bytes);

    // Sdl context items
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut window = video_subsystem.window("RustyBoy", 640, 576)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    
    // Icon for application
    let icon_surface = Surface::from_file("icon.png").unwrap();
    canvas.window_mut().set_icon(icon_surface);
    
    // autoscaling canvas
    canvas.set_logical_size(640, 576).unwrap();
    let texture_creator = canvas.texture_creator();

    // create texture
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
        .expect("failed to create streaming texture");

    // event pump
    let mut event_pump = sdl_context.event_pump().unwrap();

    // framerate control
    let mut last_render = Instant::now();

    let render_delay = Duration::from_millis(20);

    let mut frame_count = 0;

    let mut frame_count_time = Instant::now();
    
    // main loop
    'running: loop {
        // Grab joypad lock
        let mut joypad = cpu.motherboard.joypad.borrow_mut();

        // Handle events
        for event in event_pump.poll_iter() {
            let mut interrupt = false;
            // Handle key
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        // WASD up left down right
                        Some(Keycode::W) => interrupt = joypad.handle_input(2, false),
                        Some(Keycode::A) => interrupt = joypad.handle_input(1, false),
                        Some(Keycode::S) => interrupt = joypad.handle_input(3, false),
                        Some(Keycode::D) => interrupt = joypad.handle_input(0, false),
                        // A
                        Some(Keycode::K) => interrupt = joypad.handle_input(4, false),
                        // B
                        Some(Keycode::L) => interrupt = joypad.handle_input(5, false),
                        // Select
                        Some(Keycode::I) => interrupt = joypad.handle_input(6, false),
                        // Start
                        Some(Keycode::O) => interrupt = joypad.handle_input(7, false),
                        // Escape to quit
                        Some(Keycode::ESCAPE) => break 'running,
                        _ => {}
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    match keycode {
                        // WASD up left down right
                        Some(Keycode::W) => interrupt = joypad.handle_input(2, true),
                        Some(Keycode::A) => interrupt = joypad.handle_input(1, true),
                        Some(Keycode::S) => interrupt = joypad.handle_input(3, true),
                        Some(Keycode::D) => interrupt = joypad.handle_input(0, true),
                        // A
                        Some(Keycode::K) => interrupt = joypad.handle_input(4, true),
                        // B
                        Some(Keycode::L) => interrupt = joypad.handle_input(5, true),
                        // Select
                        Some(Keycode::I) => interrupt = joypad.handle_input(6, true),
                        // Start
                        Some(Keycode::O) => interrupt = joypad.handle_input(7, true),
                        _ => {}
                    }
                }
                _ => {}
            }

            if interrupt {
                cpu.set_interrupt(4);
            }
        }
        // Drop joypad lock
        drop(joypad);

        // Run cpu for 1 tick
        cpu.update();
        
        // Borrow screen
        let mut screen = cpu.motherboard.screen.borrow_mut();

        // count frames
        if screen.frame_done {
            frame_count += 1;
            screen.frame_done = false;
        }

        // frame count logic
        if frame_count_time.elapsed() >= Duration::from_secs(1) {
            let fps = frame_count as f64 / frame_count_time.elapsed().as_secs_f64();

            // set window title to fps
            canvas.window_mut().set_title(&format!("RustyBoy FPS: {:.2}", fps)).unwrap();

            // reset frame count
            frame_count = 0;
            frame_count_time = Instant::now();
        }

        // render logic
        if last_render.elapsed() >= render_delay {

            // pitch is width * 3 (RGB)
            texture.update(None, &screen.screen_buffer, 160 * 3).unwrap();
            
            // Clear canvas an apply texture
            canvas.clear();
            canvas.copy(&texture, None, None).expect("Cannot render texture");
            canvas.present();
            
            last_render = Instant::now();
        }
    }

}