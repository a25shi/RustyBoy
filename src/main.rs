mod cartridge_header;
mod memory;
mod cpu;
mod cartridge;
mod timer;
mod motherboard;
mod screen;

use std::fs;
use std::any::type_name;
use std::fmt::format;
use std::time::{Duration, Instant};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::libc::printf;
use sdl2::pixels::PixelFormatEnum;
use crate::cpu::CPU;


fn print_type_of<T>(_: &T) {
    println!("{}", type_name::<T>());
}

fn main() {
    // let file: &str = "./roms/blargg tests/cpu_instrs/individual/01-special.gb";
    // Getting cartridge header data
    let bytes: Vec<u8> = fs::read("./roms/dmg-acid2.gb").unwrap();
    
    let header = cartridge_header::CartridgeHeader::from_bytes(&bytes);
    println!("{:?}", header);
    // Cpu
    let mut cpu = CPU::new(bytes);

    // Sdl context items
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut window = video_subsystem.window("RustyBoy", 320, 288)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    // autoscaling canvas
    canvas.set_logical_size(320, 288).unwrap();
    let texture_creator = canvas.texture_creator();

    // create texture
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
        .expect("failed to create streaming texture");

    // event pump
    let mut event_pump = sdl_context.event_pump().unwrap();

    // framerate control
    let mut last_render = Instant::now();

    let render_delay = Duration::from_millis(50);

    let mut frame_count = 0;

    let mut frame_count_time = Instant::now();

    // main loop
    'running: loop {
        // Key logic
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }
        // Run cpu for 1 tick
        cpu.update();
        
        // Borrow screen
        let mut screen = cpu.motherboard.screen.borrow_mut();
        
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

        // todo render logic
        if last_render.elapsed() >= render_delay {
            // pitch is width * 3 (RGB)
            texture.update(None, &screen.screen_buffer, 160 * 3).unwrap();
            
            // Clear canvas an apply texture
            canvas.clear();
            canvas.copy(&texture, None, None);
            canvas.present();
            
            last_render = Instant::now();
        }
    }

}