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
use eframe::epaint::image;
use egui::{vec2, Checkbox, Context, FullOutput, Image, Rgba, RichText};
use egui::load::SizedTexture;
use sdl2::event::Event;
use sdl2::video::{GLProfile, SwapInterval};


fn main() {
    // Gameboy size constant
    let gb_width = 160;
    let gb_height = 144;

    // let file: &str = "./roms/blargg tests/cpu_instrs/individual/01-special.gb";
    // Getting cartridge header data and game data
    let bytes: Vec<u8> = fs::read("./roms/kirby.gb").unwrap();
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
    
    let window = video_subsystem
        .window(
            "Demo: egui rustyboy",
            600,
            600,
        )
        .opengl()
        .build()
        .unwrap();

    // Create a window context
    let gl_ctx = window.gl_create_context().unwrap();
    
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
            // Clear the screen to green
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        
        painter.update_user_texture_rgba8_data(gb_texture, screen_buffer);
        
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

        // Process ouput
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

