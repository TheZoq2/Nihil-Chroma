#![allow(dead_code)]
extern crate image;
extern crate sdl2;

extern crate nalgebra;

#[macro_use]
extern crate ecs;

mod sprite;
mod game;
mod constants;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Renderer, BlendMode};
use sdl2::surface::Surface;

use std::string::String;
use std::path::Path;
use std::rc::Rc;
use std::f64::consts;

use image::GenericImage;

use nalgebra::Vector2;
use sprite::load_texture;

use constants::*;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo: Video", RESOLUTION.0 * UPSCALING, RESOLUTION.1 * UPSCALING)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    //The renderer which actually renders to the game window
    let mut renderer = window.renderer().build().unwrap();

    //Renderer where the game is rendered in full color
    let game_surface = Surface::new(RESOLUTION.0, RESOLUTION.1, PixelFormatEnum::RGB888).unwrap();
    let mut game_renderer = Renderer::from_surface(game_surface).unwrap();
    game_renderer.set_draw_color(Color::RGB(150, 150, 0));

    let mut world = game::create_world(renderer, game_renderer);
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

        world.update();
    }
}
