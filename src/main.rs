#![allow(dead_code)]
extern crate image;
extern crate sdl2;

extern crate nalgebra;

#[macro_use]
extern crate ecs;

mod sprite;
mod game;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Renderer, BlendMode};

use std::string::String;

use std::path::Path;

use image::GenericImage;

use std::rc::Rc;

use nalgebra::Vector2;
use sprite::load_texture;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();
    renderer.set_draw_color(Color::RGB(100, 100, 0));


    let world = game::create_world(&renderer);

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut angle = 0.0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        renderer.clear();

        // test_sprite.draw(&mut renderer);
        // test_sprite2.draw(&mut renderer);

        renderer.present();

        // test_sprite.set_angle(angle);
        angle += 0.01;
    }
}
