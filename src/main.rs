extern crate sdl2;
extern crate nalgebra;
extern crate rand;
extern crate time;

extern crate specs;
#[macro_use]
extern crate specs_derive;

mod sprite;
mod game;
mod constants;
mod player;
mod rendering;
mod components;
mod input;
mod collision;

use collision::CollisionSystem;
use components::{Transform, Velocity, MaxVelocity, BallType, BoundingCircle, ObamaComponent, OrbitComponent};
use components::{HitBad, HitNeutral, HitGood, ScreenShake};
use constants::*;
use game::{RespawnComponent, MaxVelSystem, MotionSystem, ObamaSystem, OrbitSystem, RespawnSystem};
use input::InputSystem;
use player::PlayerComponent;
use sprite::{Sprite, TextureId, TextureRegistry};
use rendering::RenderingSystem;

use rand::Rng;

use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use std::path::Path;

use nalgebra::{Vector2, zero};

use sdl2::image::LoadTexture;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::mixer::{AUDIO_S16LSB, DEFAULT_CHANNELS, INIT_OGG};

use specs::{Entity, RunNow, World};

// use sfml::audio::{Music};

struct BallSpawner {
    spawn_time: f32,
    last_spawn: f32,

    types: Vec<(BallType, Sprite)>,
}

impl BallSpawner {
    pub fn new(types: Vec<(BallType, Sprite)>) -> BallSpawner {
        BallSpawner {
            spawn_time: 3.,
            last_spawn: 0.,

            types: types,
        }
    }

    pub fn do_spawn(&mut self, world: &mut World) {
        let curr_time = time::precise_time_s() as f32;
        if curr_time > self.last_spawn + self.spawn_time {
            self.spawn_ball(world);

            self.last_spawn = curr_time;
        }
    }

    pub fn spawn_ball(&self, world: &mut World) {
        let mut rng = rand::thread_rng();

        let respawn_comp = RespawnComponent{max_radius: 400.0, max_speed: 120.0, min_speed: 70.0};
        let transform = Transform {
            pos: Vector2::new(600.0, 600.0),
            angle: 0.0,
            scale: Vector2::new(0.25, 0.25)
        };
        let bound = components::BoundingCircle { radius: 28.0 * 0.5 };
        let (ball_type, ball_sprite) = self.types[rng.gen_range(0, self.types.len())];

        world.create_entity()
            .with(transform)
            .with(Velocity(Vector2::new(0.0, 0.0)))
            .with(ball_sprite)
            .with(respawn_comp)
            .with(bound)
            .with(ball_type)
            .build();
    }
}

fn create_text_texture<'r, T>(
    text: &str,
    font: &Font,
    texture_creator: &'r TextureCreator<T>,
) -> Texture<'r> {
    // render a surface, and convert it to a texture bound to the renderer
    let surface = font.render(text)
        .blended(Color::RGBA(255, 255, 255, 255)).unwrap();
    texture_creator.create_texture_from_surface(&surface).unwrap()
}

fn create_text_entity(texture: TextureId, world: &mut World) -> Entity {
    // Create text entity
    world.create_entity()
        .with(Transform {
            pos: Vector2::new(140.0, 10.0),
            angle: 0.0,
            scale: Vector2::new(0.1, 0.1)
        })
        .with(Sprite::new(texture))
        .build()
}

// TODO: split this into multiple functions
pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let frequency = 44_100;
    let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size).unwrap();
    let _mixer_context = sdl2::mixer::init(INIT_OGG).unwrap();

    let window = video_subsystem.window(
        "rust-sdl2 demo: Video", RESOLUTION.0 * UPSCALING, RESOLUTION.1 * UPSCALING)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    // The renderer which actually renders to the game window
    let canvas = window.into_canvas().build().unwrap();

    // Renderer where the game is rendered in full color
    let game_surface = Surface::new(
        RESOLUTION.0, RESOLUTION.1, PixelFormatEnum::RGB888).unwrap();
    let mut game_canvas = game_surface.into_canvas().unwrap();
    game_canvas.set_draw_color(Color::RGB(200, 80, 50));

    let game_texture_creator = game_canvas.texture_creator();
    let texture_registry_ref = Rc::new(RefCell::new(TextureRegistry::new()));

    let load_texture = |filename| {
        println!("Loading {}", filename);
        let tex = game_texture_creator.load_texture(filename).unwrap();
        texture_registry_ref.borrow_mut().add(tex)
    };

    let ball_data = vec! {
        (BallType::Good, "data/good.png"),
        (BallType::Neutral, "data/neutral.png"),
        (BallType::Bad, "data/bad.png"),
    };

    let mut ball_spawner = BallSpawner::new(ball_data.into_iter().map(|(ball_type, ball_texture_file)| {
        (ball_type, Sprite::new(load_texture(ball_texture_file)))
    }).collect());

    let sausage_sprite = Sprite::new(load_texture("data/sausage.png"));
    let sausage_transform = Transform {
        pos: Vector2::new(100000000., RESOLUTION.1 as f32 / 2.0),
        angle: 0.0,
        scale: Vector2::new(1.5, 1.5)
    };
    let nuke_sprite = Sprite::new(load_texture("data/nuke.png"));
    let nuke_transform = Transform {
        pos: Vector2::new(100000., RESOLUTION.1 as f32 / 2.0),
        angle: 0.0,
        scale: Vector2::new(0.25, 0.25)
    };
    let mut sausage_is_spawned = false;
    let mut nuke_is_spawned = false;

    let obama_files = fs::read_dir("data/obamas").unwrap();
    let obama_file_names = obama_files.map(
        |file| file.unwrap().path().to_str().unwrap().to_string()
    );
    // TODO: merge this with the load_texture closure
    let obama_sprites = obama_file_names.map(|file_name| {
        println!("Loading {}", file_name);
        let tex = game_texture_creator.load_texture(&file_name).unwrap();
        Sprite::new(texture_registry_ref.borrow_mut().add(tex))
    }).collect();

    // Create font
    let font = ttf_context.load_font(&Path::new("data/font.ttf"), 128).unwrap();

    let event_pump = sdl_context.event_pump().unwrap();
    
    let mut world = World::new();
    world.register::<Transform>();
    world.register::<Velocity>();
    world.register::<Sprite>();
    world.register::<BoundingCircle>();
    world.register::<PlayerComponent>();
    world.register::<ObamaComponent>();
    world.register::<RespawnComponent>();
    world.register::<BallType>();
    world.register::<MaxVelocity>();
    world.register::<OrbitComponent>();

    let good_texture = load_texture("data/good.png");
    let test_sprite = Sprite::new(good_texture);
    let sprite_scale = 0.25;
    let player_transform = Transform {
        pos: Vector2::new(RESOLUTION.0 as f32 / 2., RESOLUTION.1 as f32 / 2.0),
        angle: 0.0,
        scale: Vector2::new(sprite_scale, sprite_scale)
    };

    let player_box = BoundingCircle { radius: 56.0 * sprite_scale };

    // Create some entites with some components
    let player_entity = world.create_entity()
        .with(Velocity(Vector2::new(0.0, 0.0)))
        .with(MaxVelocity(100.0))
        .with(player_transform)
        .with(test_sprite)
        .with(PlayerComponent::new())
        .with(player_box)
        .build();

    // TODO: use the specs dispatcher with tread_local for the rendering system
    let mut motion_system = MotionSystem { frametime: 0. };
    let mut obama_system = ObamaSystem { too_few_obamas: false };
    let mut rendering_system = RenderingSystem::new(canvas, game_canvas, player_entity, 5.0, texture_registry_ref.clone());
    let mut input_system = InputSystem {
        event_pump: event_pump,
        should_exit: false,
        mouse_pos: zero(),
    };
    let mut collision_system = CollisionSystem { player: player_entity, new_points: 0 };
    let mut orbit_system = OrbitSystem { player: player_entity, nuke_angle: 0. };
    let mut max_vel_system = MaxVelSystem;
    let mut respawn_system = RespawnSystem;

    let mut points = 0;
    let mut life = 3;

    let score_texture_id = texture_registry_ref.borrow_mut().add(
        create_text_texture("Score: 0", &font, &game_texture_creator)
    );
    let score_entity = create_text_entity(score_texture_id, &mut world);

    for _ in 0..20 {
        ball_spawner.spawn_ball(&mut world);
    }

    let start_time = time::precise_time_s() as f32;
    let music = sdl2::mixer::Music::from_file("data/music.ogg").unwrap();
    music.play(-1).unwrap();

    world.add_resource(HitBad(false));
    world.add_resource(HitNeutral(false));
    world.add_resource(HitGood(false));
    world.add_resource(ScreenShake(None));

    let mut old_time = 0.0;
    'running: loop {
        let curr_time = time::precise_time_s() as f32;
        let frametime = curr_time - old_time;
        old_time = curr_time;
        motion_system.frametime = frametime;

        obama_system.run_now(&world.res);
        motion_system.run_now(&world.res);
        obama_system.run_now(&world.res);
        input_system.run_now(&world.res);
        collision_system.run_now(&world.res);
        orbit_system.run_now(&world.res);
        max_vel_system.run_now(&world.res);
        respawn_system.run_now(&world.res);

        world.maintain();
        rendering_system.run_now(&world.res);

        if obama_system.too_few_obamas {
            game::create_obama(&mut world, &obama_sprites);
        }

        ball_spawner.do_spawn(&mut world);

        //Handle points and endgame
        let new_points = collision_system.new_points;
        points += new_points;

        // TODO: this could be optimized to only create a new texture when the text changes
        world.entities().delete(score_entity).unwrap();
        let score_string = format!(
            "Score: {} Life: {} Sausage countdown: {}",
            points,
            life,
            (curr_time - start_time) as i32 - 180
        );
        let new_score_texture = create_text_texture(&score_string, &font, &game_texture_creator);
        texture_registry_ref.borrow_mut().replace(score_texture_id, new_score_texture);

        {
            let mut hit_bad = world.write_resource::<HitBad>();
            if hit_bad.0 {
                life -= 1;
                *hit_bad = HitBad(false);
            }
            if life < 0 {
                println!("You died, final score: {}", points);
                break 'running;
            }
        }

        if input_system.should_exit {
            break 'running;
        }

        // TODO: everything below this point could probably be moved into a new system
        let nuke_time = 180.0;
        let nuke_angle = orbit_system.nuke_angle;

        if curr_time - start_time > nuke_time && sausage_is_spawned == false {
            sausage_is_spawned = true;

            let sausage_sprite = sausage_sprite;
            world.create_entity()
                .with(sausage_transform)
                .with(OrbitComponent{radius: 1000., target_radius:150., angle:0., angular_velocity: 0.02})
                .with(sausage_sprite)
                .build();
        }

        if curr_time - start_time > nuke_time + 10. {
            {
                let mut screen_shake = world.write_resource::<ScreenShake>();
                *screen_shake = ScreenShake(Some(10.));
            }
            if !nuke_is_spawned {
                nuke_is_spawned = true;

                for _ in 0..3 {
                    let bound = components::BoundingCircle { radius: 28.0 * 0.5 };
                    let ball_type = BallType::Bad;

                    world.create_entity()
                        .with(nuke_transform)
                        .with(OrbitComponent{radius: 250., target_radius:0., angle:nuke_angle, angular_velocity: 0.02})
                        .with(nuke_sprite)
                        .with(bound)
                        .with(ball_type.clone())
                        .build();
                }
            }
        }
    }
}
