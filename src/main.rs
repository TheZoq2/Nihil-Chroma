#![allow(dead_code)]
extern crate image;
extern crate sdl2;
extern crate sdl2_ttf;
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
use std::path::Path;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Renderer, Texture};
use sdl2::surface::Surface;
use sdl2_ttf::Font;

use nalgebra::Vector2;
use ecs::{Entity, World};

use sprite::load_texture;
use sprite::Sprite;

use constants::*;

use game::{RespawnComponent, MySystems};
use components::{MyComponents, Transform, BallType};

struct BallSpawner
{
    spawn_time: f32,
    last_spawn: f32,

    types: Vec<(BallType, Rc<Texture>)>,
}
impl BallSpawner
{
    pub fn new(types: Vec<(BallType, Rc<Texture>)>) -> BallSpawner
    {
        BallSpawner {
            spawn_time: 3.,
            last_spawn: 0.,

            types: types,
        }
    }

    pub fn do_spawn(&mut self, world: &mut ecs::World<game::MySystems>)
    {
        let curr_time = time::precise_time_s() as f32;
        if curr_time > self.last_spawn + self.spawn_time
        {
            self.spawn_ball(world);

            self.last_spawn = curr_time;
        }
    }

    pub fn spawn_ball(&self, world: &mut ecs::World<game::MySystems>)
    {
        let mut rng = rand::thread_rng();

        let respawn_comp = RespawnComponent{max_radius: 400.0, max_speed: 120.0, min_speed: 70.0};
        let transform = Transform { pos: Vector2::new(600.0, 600.0), angle: 0.0,
                             scale: Vector2::new(0.25, 0.25) };

        let bound = components::BoundingCircle { radius: 28.0 * 0.5 };

        let ball_type = self.types[rng.gen_range(0, self.types.len())].clone();

        //TODO: This could cause a crash depending on what the .clone method does
        let sprite = sprite::Sprite::new(ball_type.1.clone());

        world.create_entity(
        |entity: ecs::BuildData<MyComponents>, data: &mut MyComponents| {
            data.transform.add(&entity, transform);
            data.velocity.add(&entity, Vector2::new(0.0, 0.0));
            data.sprite.add(&entity, sprite);
            data.respawn_component.add(&entity, respawn_comp.clone());
            data.bounding_box.add(&entity, bound);
            data.ball_type.add(&entity, ball_type.0.clone());
        });
    }
}

fn create_text_texture<'a>(text: &'a str, font: &Font, world: &mut World<MySystems>) -> Texture
{
    // render a surface, and convert it to a texture bound to the renderer
    let surface = font.render(text)
        .blended(Color::RGBA(255, 255, 255, 255)).unwrap();
    let renderer = world.systems.rendering.inner.as_ref().unwrap().game_renderer.borrow_mut();
    renderer.create_texture_from_surface(&surface).unwrap()
}

fn create_text_entity<'a>(text: &'a str, font: &Font, world: &mut World<MySystems>) -> Entity
{
    let texture = create_text_texture(text, font, world);

    // Create text entity
    world.create_entity(
        |entity: ecs::BuildData<MyComponents>, data: &mut MyComponents| {
            data.transform.add(&entity, Transform {
                pos: Vector2::new(40.0, 10.0),
                angle: 0.0,
                scale: Vector2::new(0.1, 0.1)
            });
            data.sprite.add(&entity, Sprite::new(Rc::new(texture)));
        })
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2_ttf::init().unwrap();

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
        (BallType::Good, Rc::new(load_texture(&game_renderer, "data/good.png"))),
        (BallType::Neutral, Rc::new(load_texture(&game_renderer, "data/neutral.png"))),
        (BallType::Bad, Rc::new(load_texture(&game_renderer, "data/bad.png"))),
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

    // Create font
    let font = ttf_context.load_font(&Path::new("data/font.ttf"), 128).unwrap();

    let event_pump = sdl_context.event_pump().unwrap();

    let mut world = game::create_world(renderer, game_renderer, event_pump);
    let mut points = 0;

    let mut score_entity = create_text_entity("Score: 0", &font, &mut world);

    for _ in 0..20
    {
        ball_spawner.spawn_ball(&mut world);
    }

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

        //Handle points and endgame
        let new_points = world.services.new_points;
        points += new_points;

        if new_points != 0 {
            world.remove_entity(score_entity);
            let score_string = "Score: ".to_string() + &points.to_string();
            score_entity = create_text_entity(&score_string, &font, &mut world)
        }

        if world.services.hit_bad
        {
            println!("You died, final score: {}", points);
            return;
        }

        let should_exit = world.systems.input.inner.as_ref().unwrap().should_exit;

        if should_exit
        {
            return;
        }
    }
}
