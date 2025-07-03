mod rusty_boy;

use std::fs;
use egui_sdl2_gl::{gl, DpiScaling, ShaderVersion};
use std::time::{Duration, Instant};
use egui::{vec2, Checkbox, Context, Direction, FullOutput, Image};
use egui::load::SizedTexture;
use sdl2::event::Event;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::video::{GLProfile, SwapInterval};
use crate::rusty_boy::RustyBoy;

fn main() {
    // Gameboy size constant
    let gb_width = 160;
    let gb_height = 144;

    // init gameboy 
    let mut rusty = RustyBoy::new();
    
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
            480,
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
    
    // set screen buffer placeholder
    let mut screen_buffer = rusty.update_and_render();

    // texture
    let gb_texture = painter.new_user_texture_rgba8((gb_width, gb_height), screen_buffer, false);
    let start_time = Instant::now();

    // counter 
    let mut counter = 0;
    let mut frame_counter = Instant::now();

    // frame
    let my_frame = egui::containers::Frame::new().fill(egui::Color32::DARK_GRAY);
    
    // egui automatically runs at 60 fps
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        // Escape to quit
                        Some(Keycode::ESCAPE) => break 'running,
                        _ => rusty.handle_events(keycode, false)
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    match keycode {
                        // WASD up left down right
                        _ => rusty.handle_events(keycode, true)
                    }
                }
                _ => {
                    // Process input event
                    egui_state.process_input(&window, event, &mut painter);
                }
            }
        }
        
        screen_buffer = rusty.update_and_render();

        //fps check
        counter += 1;

        // Set fps counter in window
        if frame_counter.elapsed() >= Duration::from_secs(1) {
            let fps = counter as f64 / frame_counter.elapsed().as_secs_f64();
            window.set_title(&format!("RustyBoy FPS: {:.2}", fps)).unwrap();
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

        // GUI layout
        egui::CentralPanel::default().frame(my_frame)
            .show(&egui_ctx, |ui| {
                ui.add_space(5.0);
                ui.columns(5, |col| {
                    col[0].vertical_centered(|ui| {
                        if ui.add_sized([80.0, 20.0], egui::Button::new("Load ROM")).clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                rusty.load_rom(path);
                            }
                        }
                    })
                });
                ui.end_row();
                ui.add(egui::Separator::default().spacing(5.0));
                ui.with_layout(egui::Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                    // Actual emulator screen
                    ui.add(Image::new(SizedTexture::new(gb_texture, vec2((gb_width * 3) as f32, (gb_height * 3) as f32))))
                });
            });

        let FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output,
        } = egui_ctx.end_pass();

        // process output to egui
        egui_state.process_output(&window, &platform_output);
        
        // render egui
        let paint_jobs = egui_ctx.tessellate(shapes, pixels_per_point);
        painter.paint_jobs(None, textures_delta, paint_jobs);
        window.gl_swap_window();
    }
}

