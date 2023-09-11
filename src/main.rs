use sdl2::{event::Event, pixels::Color, rect::Rect, render::Canvas, video::Window};
use std::{env, fs::File, io::Read};

use cpu::{CPU, SCREEN_HEIGHT, SCREEN_WIDTH};

mod cpu;

const SCALE: u32 = 10;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run /path/to/game");
        return;
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Rusty Chip8", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut cpu = CPU::new();

    let mut rom = File::open(&args[1]).expect("unable to open ROM file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer);
    cpu.load(&buffer);

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'gameloop;
                }
                _ => (),
            }
        }

        cpu.tick();
        draw_screen(&cpu, &mut canvas);
    }
}

fn draw_screen(cpu: &CPU, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    let screen_buffer = cpu.screen;
    canvas.set_draw_color(Color::WHITE);

    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect);
        }
    }

    canvas.present();
}
