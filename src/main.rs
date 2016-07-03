#![allow(dead_code)]
extern crate image;
extern crate sdl2;
extern crate nalgebra;
extern crate rand;
extern crate time;

#[macro_use]
extern crate ecs;

mod sprite;
mod game;
mod constants;
mod player;
mod rendering;
mod components;
mod input;
mod collision;

use rand::Rng;

use std::fs;
use std::rc::Rc;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Renderer, Texture};
use sdl2::surface::Surface;

use nalgebra::Vector2;

use sprite::load_texture;

use constants::*;

use game::{RespawnComponent};
use components::{MyComponents, Transform};

struct BallSpawner 
{
    spawn_time: f32,
    last_spawn: f32,
    
    textures: Vec<Rc<Texture>>,
}
impl BallSpawner
{
    pub fn new(textures: Vec<Rc<Texture>>) -> BallSpawner 
    {
        BallSpawner {
            spawn_time: 4.,
            last_spawn: 0.,

            textures: textures,
        }
    }

    pub fn do_spawn(&mut self, world: &mut ecs::World<game::MySystems>) 
    {
        let curr_time = time::precise_time_s() as f32;
        let mut rng = rand::thread_rng();

        if curr_time > self.last_spawn + self.spawn_time
        {
            let respawn_comp = RespawnComponent{max_radius: 400.0, max_speed: 80.0, min_speed: 40.0};
            let transform = Transform { pos: Vector2::new(600.0, 600.0), angle: 0.0,
                                 scale: Vector2::new(0.25, 0.25) };

            let bound = components::BoundingCircle { radius: 28.0 * 0.5 };

            //TODO: This could cause a crash depending on what the .clone method does
            let sprite = sprite::Sprite::new(self.textures[rng.gen_range(0, self.textures.len())].clone());

            world.create_entity(
            |entity: ecs::BuildData<MyComponents>, data: &mut MyComponents| {
                data.transform.add(&entity, transform);
                data.velocity.add(&entity, Vector2::new(0.0, 0.0));
                data.sprite.add(&entity, sprite);
                data.respawn_component.add(&entity, respawn_comp.clone());
                data.bounding_box.add(&entity, bound);
            });

            self.last_spawn = curr_time;
        }
    }
}

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
    game_renderer.set_draw_color(Color::RGB(200, 80, 50));

    let ball_textures = vec!{
        Rc::new(load_texture(&game_renderer, "data/good.png")),
        Rc::new(load_texture(&game_renderer, "data/neutral.png")),
        Rc::new(load_texture(&game_renderer, "data/bad.png")),
    };

    let mut ball_spawner = BallSpawner::new(ball_textures);


    let obama_files = fs::read_dir("data/obamas").unwrap();
    let mut obama_textures = Vec::new();
    for file in obama_files {
        let filename = file.unwrap().path().to_str().unwrap().to_string();
        let obama_texture = Rc::new (sprite::load_texture(
            &game_renderer, &filename
        ));
        obama_textures.push(obama_texture);
    }

    let event_pump = sdl_context.event_pump().unwrap();

    let mut world = game::create_world(renderer, game_renderer, event_pump);

    let mut old_time = 0.0;
    'running: loop {
        let curr_time = time::precise_time_s() as f32;
        let frametime = curr_time - old_time;
        old_time = curr_time;
        world.systems.motion.inner.frametime = frametime;

        world.update();

        if world.services.too_few_obamas {
            game::create_obama(&mut world, &obama_textures);
        }

        ball_spawner.do_spawn(&mut world);

        let should_exit = world.systems.input.inner.as_ref().unwrap().should_exit;

        if should_exit
        {
            return;
        }
    }
}
