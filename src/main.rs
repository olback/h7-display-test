#![feature(generic_const_exprs)]
#![feature(maybe_uninit_uninit_array)]
#![feature(const_maybe_uninit_uninit_array)]
#![allow(mutable_transmutes)]
#![feature(duration_constants)]
#![feature(const_mut_refs)]

use embedded_graphics::{mono_font::MonoTextStyle, primitives::Rectangle, text::Text};
use embedded_graphics_core::{
    pixelcolor::{Rgb565, Rgb888},
    prelude::*,
};
use h7_display::{FrameBuffer, H7Display};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::{
    alloc::{alloc, dealloc, Layout},
    mem::{align_of, size_of},
    time::{Duration, Instant},
};

mod h7_display;

const WIDTH: usize = 1280;
const HEIGHT: usize = 768;
type COLOR = Rgb565;

macro_rules! sz_al_of {
    ($type:ty) => {
        println!(
            "{}: sz = {}, al = {}",
            stringify!($type),
            size_of::<$type>(),
            align_of::<$type>()
        )
    };
}

fn timer<D: core::fmt::Display, F>(text: D, mut func: F)
where
    F: FnMut(),
{
    let start = Instant::now();
    func();
    let diff = Instant::now() - start;
    let fps = Duration::SECOND.as_micros() as f64 / diff.as_micros() as f64;
    println!("{text}: {diff:?}, FPS: {fps:.02}");
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(env!("CARGO_PKG_NAME"), WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .vulkan()
        // .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let (vram_ptr, vram_layout, front_buffer, back_buffer) = unsafe {
        const FBSZ: usize = size_of::<FrameBuffer<COLOR, WIDTH, HEIGHT>>();
        const FBAL: usize = align_of::<FrameBuffer<COLOR, WIDTH, HEIGHT>>();
        // This assertion makes sure that consecutive framebuffers will be properly aligned.
        assert_eq!(FBSZ % FBAL, 0);
        let layout = Layout::from_size_align_unchecked(FBSZ * 2, FBAL);
        let vram_ptr = alloc(layout);
        let front_buffer = &mut *(vram_ptr as *mut _);
        let back_buffer = &mut *(vram_ptr.add(FBSZ) as *mut _);
        (vram_ptr, layout, front_buffer, back_buffer)
    };
    sz_al_of!(FrameBuffer<COLOR, WIDTH, HEIGHT>);
    sz_al_of!(H7Display::<COLOR, WIDTH, HEIGHT>);
    println!("vram_layout: {vram_layout:?}");
    let mut display = H7Display::<COLOR, WIDTH, HEIGHT>::new(front_buffer, back_buffer);
    'running: loop {
        let sof = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    timer("Swap buffers", || {
                        display.swap_buffers();
                    });
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => {
                    let w = display.width();
                    let back = display.back_buffer_mut();
                    back[0..(w * 20)].fill(COLOR::RED);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => {
                    let w = display.width();
                    let back = display.back_buffer_mut();
                    back[(w * 20)..(w * 40)].fill(COLOR::GREEN);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => {
                    let w = display.width();
                    let back = display.back_buffer_mut();
                    back[(w * 40)..(w * 60)].fill(COLOR::BLUE);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    display
                        .fill_solid(
                            &Rectangle::new(Point::new(0, 0), Size::new(100, 100)),
                            COLOR::CSS_CYAN,
                        )
                        .unwrap();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    timer("Clear", || {
                        display.clear(COLOR::BLACK).unwrap();
                    });
                }
                Event::KeyDown {
                    keycode: Some(Keycode::L),
                    ..
                } => {
                    timer("Lines", || {
                        let line_height = 32;
                        for line in 0..(display.height() / line_height) {
                            let color = if line % 2 == 0 {
                                COLOR::CSS_GRAY
                            } else {
                                COLOR::CSS_LIGHT_GRAY
                            };
                            display
                                .fill_solid(
                                    &Rectangle::new(
                                        Point::new(0, (line * line_height) as i32),
                                        Size::new(display.width() as u32, line_height as u32),
                                    ),
                                    color,
                                )
                                .unwrap();
                        }
                    });
                }
                Event::KeyDown {
                    keycode: Some(Keycode::T),
                    ..
                } => {
                    timer("Text", || {
                        let text_style =
                            MonoTextStyle::new(&profont::PROFONT_24_POINT, COLOR::BLACK);
                        let line_height = 32;
                        for line in 0..(display.height() / line_height) {
                            Text::new(
                                &format!("{line}"),
                                Point::new(50, ((line * line_height) + line_height - 8) as i32),
                                text_style,
                            )
                            .draw(&mut display)
                            .unwrap();
                        }
                    });
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    timer("Scroll up", || display.scroll(-32, COLOR::GREEN));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    timer("Scroll down", || display.scroll(32, COLOR::RED));
                }
                _ => {}
            }
        }

        // Draw our own stuff on the canvas
        let front = display.front_buffer();
        for x in 0..display.width() {
            for y in 0..display.height() {
                let color = Rgb888::from(*front.at(x, y));
                canvas.set_draw_color((color.r(), color.g(), color.b()));
                canvas.draw_point((x as i32, y as i32))?;
            }
        }

        // Swap sdl2 buffers
        timer("Present", || {
            canvas.present();
        });
        display.swap_buffers();

        let diff = Instant::now() - sof;
        let fps = 1000f64 / diff.as_millis() as f64;
        println!("Frame time: {diff:?}, FPS: {fps:.02}");

        // TODO: Sleep enough to limit fps to 30
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    unsafe { dealloc(vram_ptr, vram_layout) };

    Ok(())
}
