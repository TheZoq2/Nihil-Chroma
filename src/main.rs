#![allow(dead_code)]
extern crate image;
extern crate sdl2;
extern crate nalgebra;
extern crate rand;

#[macro_use]
extern crate ecs;

mod sprite;
mod game;
mod constants;
mod player;
mod rendering;
mod components;

use std::rc::Rc;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Renderer};
use sdl2::surface::Surface;


use constants::*;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window(
        "rust-sdl2 demo: Video", RESOLUTION.0 * UPSCALING, RESOLUTION.1 * UPSCALING)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    //The renderer which actually renders to the game window
    let renderer = window.renderer().build().unwrap();

    //Renderer where the game is rendered in full color
    let game_surface = Surface::new(
        RESOLUTION.0, RESOLUTION.1, PixelFormatEnum::RGB888).unwrap();
    let mut game_renderer = Renderer::from_surface(game_surface).unwrap();
    game_renderer.set_draw_color(Color::RGB(100, 150, 50));

    let obama_texture = Rc::new(sprite::load_texture(
        &game_renderer, String::from("data/obama.png")
    ));

    let event_pump = sdl_context.event_pump().unwrap();

    let mut world = game::create_world(renderer, game_renderer, event_pump);

    'running: loop {

        world.update();

        if world.services.too_few_obamas {
            game::create_obama(&mut world, &obama_texture);
        }

        let should_exit = world.systems.input.inner.as_ref().unwrap().should_exit;

        if should_exit
        {
            return;
        }
    }
}
