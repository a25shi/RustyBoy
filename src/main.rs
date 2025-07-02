mod cartridge;
mod cartridge_header;
mod cpu;
mod joypad;
mod memory;
mod motherboard;
mod screen;
mod timer;

use crate::cpu::CPU;
use egui_sdl2_gl::{gl, DpiScaling, ShaderVersion};
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};
use egui::{vec2, Checkbox, Context, FullOutput, Image};
use egui::load::SizedTexture;
use sdl2::event::Event;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::video::{GLProfile, SwapInterval};


fn main() {
    // Gameboy size constant
    let gb_width = 160;
    let gb_height = 144;

    // let file: &str = "./roms/blargg tests/cpu_instrs/individual/01-special.gb";
    // Getting cartridge header data and game data
    let bytes: Vec<u8> = fs::read("./roms/super mario.gb").unwrap();
    let header = cartridge_header::CartridgeHeader::from_bytes(&bytes);
    let mut cpu = CPU::new(bytes);
    
    println!("{:?}", header);
    
    // sdl context stuff
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    // open gl / sdl2 stuff
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_double_buffer(true);
    gl_attr.set_framebuffer_srgb_compatible(true);
    
    let mut window = video_subsystem
        .window(
            "RustyBoy",
            500,
            450,
        )
        .opengl()
        .build()
        .unwrap();

    // Create a window context
    let gl_ctx = window.gl_create_context().unwrap();

    // app icon
    window.set_icon(Surface::from_file("icon.png").unwrap());

    // Enable vsync
    if let Err(error) = window.subsystem().gl_set_swap_interval(SwapInterval::VSync) {
        println!(
            "Failed to gl_set_swap_interval(SwapInterval::VSync): {}",
            error
        );
    };
    
    // Init egui stuff
    let shader_ver = ShaderVersion::Default;
    let (mut painter, mut egui_state) =
        egui_sdl2_gl::with_sdl2(&window, shader_ver, DpiScaling::Default);
    let egui_ctx = egui::Context::default();
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    // set screen buffer
    let mut screen_buffer = cpu.motherboard.screen.borrow().screen_buffer.clone();

    // texture
    let gb_texture = painter.new_user_texture_rgba8((gb_width, gb_height), screen_buffer, false);
    let start_time = Instant::now();
    
    // counter 
    let mut counter = 0;
    let mut frame_counter = Instant::now();
    
    // egui automatically runs at 60 fps
    'running: loop {
        // Handle events
        let mut joypad = cpu.motherboard.joypad.borrow_mut();
        let mut interrupt = false;
        for event in event_pump.poll_iter() {
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
                _ => {
                    // Process input event
                    egui_state.process_input(&window, event, &mut painter);
                }
            }
            if interrupt {
                cpu.set_interrupt(4);
            }
        }
        // Drop joypad mut
        drop(joypad);
        
        // run emulator for one frame
        cpu.run_one_frame();
        screen_buffer = cpu.motherboard.screen.borrow().screen_buffer.clone();

        //fps check
        counter += 1;

        if frame_counter.elapsed() >= Duration::from_secs(1) {
            let fps = counter as f64 / frame_counter.elapsed().as_secs_f64();
            println!("{:.2}", fps);
            // reset frame count
            counter = 0;
            frame_counter = Instant::now();
        }
        
        // egui start
        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_pass(egui_state.input.take());
        
        unsafe {
            // Clear the screen
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        
        // update screen buffer
        painter.update_user_texture_rgba8_data(gb_texture, screen_buffer);
        
        // add image to gui
        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            ui.add(Image::new(SizedTexture::new(gb_texture, vec2((gb_width * 3) as f32, (gb_height * 3) as f32))))
        });

        let FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output,
        } = egui_ctx.end_pass();

        // render gui to window
        egui_state.process_output(&window, &platform_output);

        // For default dpi scaling only, Update window when the size of resized window is very small (to avoid egui::CentralPanel distortions).
        // if egui_ctx.used_size() != painter.screen_rect.size() {
        //     println!("resized.");
        //     let _size = egui_ctx.used_size();
        //     let (w, h) = (_size.x as u32, _size.y as u32);
        //     window.set_size(w, h).unwrap();
        // }

        let paint_jobs = egui_ctx.tessellate(shapes, pixels_per_point);
        painter.paint_jobs(None, textures_delta, paint_jobs);
        window.gl_swap_window();
    }
}

