#![feature(generic_const_exprs)]
#![feature(maybe_uninit_uninit_array)]
#![feature(const_maybe_uninit_uninit_array)]
#![allow(mutable_transmutes)]
#![feature(exclusive_range_pattern)]

use embedded_graphics::primitives::{Primitive, PrimitiveStyle, Rectangle};
use embedded_graphics_core::{
    pixelcolor::{Rgb565, Rgb888},
    prelude::*,
};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

mod h7_display;

pub fn main() -> Result<(), String> {
    // New thread because we need a bigger stack
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .name("lmao".into())
        .spawn(move || -> Result<(), String> {
            let sdl_context = sdl2::init()?;
            let video_subsystem = sdl_context.video()?;

            let window = video_subsystem
                .window(env!("CARGO_PKG_NAME"), 1280, 768)
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

            let mut display = h7_display::H7Display::<Rgb565, 1280, 768>::new();
            display.info();
            'running: loop {
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
                            let start = Instant::now();
                            display.swap_buffers();
                            let diff = Instant::now() - start;
                            println!("Swapped buffers took {diff:?}");
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Num1),
                            ..
                        } => {
                            let w = display.width();
                            let back = display.back_buffer_mut();
                            back[0..(w * 20)].fill(Rgb565::RED);
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Num2),
                            ..
                        } => {
                            let w = display.width();
                            let back = display.back_buffer_mut();
                            back[(w * 20)..(w * 40)].fill(Rgb565::GREEN);
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Num3),
                            ..
                        } => {
                            let w = display.width();
                            let back = display.back_buffer_mut();
                            back[(w * 40)..(w * 60)].fill(Rgb565::BLUE);
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::R),
                            ..
                        } => {
                            // display.clear(Rgb565::BLACK).unwrap();
                            display
                                .fill_solid(
                                    &Rectangle::new(Point::new(-50, -50), Size::new(100, 100)),
                                    Rgb565::CSS_CYAN,
                                )
                                .unwrap();
                            // let fill = PrimitiveStyle::with_fill();

                            //     .into_styled(fill)
                            //     .draw(&mut display)
                            //     .unwrap();
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::C),
                            ..
                        } => {
                            let start = Instant::now();
                            display.clear(Rgb565::BLACK).unwrap();
                            let diff = Instant::now() - start;
                            println!("Clear took {diff:?}");
                        }
                        _ => {}
                    }
                }

                // Clear the canvas
                canvas.set_draw_color(Color::RGB(255, 255, 255));
                canvas.clear();

                // Draw our own stuff on the canvas
                // canvas.set_draw_color(Color::RGB(255, 255, 255));
                // canvas.draw_point((10, 10))?;
                let start = Instant::now();
                let front = display.front_buffer();
                for x in 0..display.width() {
                    for y in 0..display.height() {
                        let color = Rgb888::from(*front.at(x, y));
                        canvas.set_draw_color((color.r(), color.g(), color.b()));
                        canvas.draw_point((x as i32, y as i32))?;
                    }
                }
                let diff = Instant::now() - start;
                // println!("Frame time: {diff:?}");

                // Swap sdl2 buffers
                canvas.present();

                ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
            }

            Ok(())
        });

    // 'running: loop {
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Some(Keycode::Escape),
    //                 ..
    //             } => break 'running,
    //             _ => {}
    //         }
    //     }

    //     // Clear the canvas
    //     canvas.set_draw_color(Color::RGB(0, 0, 0));
    //     canvas.clear();

    //     // Draw our own stuff on the canvas
    //     canvas.set_draw_color(Color::RGB(255, 255, 255));
    //     canvas.draw_point((10, 10))?;

    //     // Swap buffers
    //     canvas.present();

    //     ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    //     // The rest of the game loop goes here...

    handle.unwrap().join().unwrap()
}
