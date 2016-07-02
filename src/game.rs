extern crate ecs;
extern crate nalgebra as na;
extern crate rand;

use sdl2::render::{Renderer};
use sdl2::pixels::PixelFormatEnum;
use std::rc::Rc;
use std::cell::RefCell;
use std::f64::consts;
use rand::Rng;

use nalgebra::{Vector2, Norm, Absolute};

use ecs::{Entity, World, BuildData, System, DataHelper, EntityIter};
use ecs::system::{EntityProcess, EntitySystem, LazySystem};
use ecs::Aspect;

use sprite::{Sprite, Transform, load_texture};
use constants::*;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use player::{PlayerComponent};
use player;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RespawnComponent
{
    max_radius: f32,
    max_speed: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoundingBox {
    offset: Vector2<f32>,
    size: Vector2<f32>,
}

impl Default for BoundingBox {
    fn default() -> BoundingBox {
        BoundingBox {
            offset: Vector2::new(0.0, 0.0),
            size: Vector2::new(1.0, 1.0)
        }
    }
}

pub struct RenderingSystem<'a> {
    pub renderer: RefCell<Renderer<'a>>,
    pub game_renderer: RefCell<Renderer<'a>>,
    pub player: ecs::Entity,
}

impl<'a> System for RenderingSystem<'a> {
    type Components = MyComponents;
    type Services = ();
}

impl<'a> EntityProcess for RenderingSystem<'a> {
    fn process(&mut self, entities: EntityIter<MyComponents>,
                       data: &mut DataHelper<MyComponents, ()>)
    {
        let mut renderer = self.renderer.borrow_mut();
        let mut game_renderer = self.game_renderer.borrow_mut();

        //Getting some parameters about the player
        let mut plr_angle = 0.0;
        let mut player_pos = Vector2::new(0.0, 0.0);
        data.with_entity_data(&self.player, |entity, data|{
            plr_angle = data.transform[entity].angle;
            player_pos = data.transform[entity].pos;
        });

        game_renderer.clear();

        for e in entities {
            let ref transform = data.transform[e];
            data.sprite[e].draw(&transform, &mut game_renderer);
        }

        let game_surface = game_renderer.surface().unwrap();

        // Creating a new texture to which we will 'copy' the pixels from the
        // game renderer and make some of them grayscale
        let mut game_texture = renderer.create_texture_streaming(
                    PixelFormatEnum::RGB888,
                    RESOLUTION.0,
                    RESOLUTION.1).unwrap();

        game_texture.with_lock(None, |buffer: &mut [u8], pitch: usize|{
            let surface_data = game_surface.without_lock().unwrap();

            for y in 0..RESOLUTION.1
            {
                for x in 0..RESOLUTION.0
                {

                    let is_in_cone = is_in_cone(player_pos, x, y, plr_angle);

                    //Doing the grayscale stuff
                    let surface_offset = (y) * game_surface.pitch() as u32 + (x) * 4;

                    let raw_r = surface_data[(surface_offset + 0) as usize];
                    let raw_g = surface_data[(surface_offset + 1) as usize];
                    let raw_b = surface_data[(surface_offset + 2) as usize];

                    let texture_offset = y*pitch as u32 + x*4;
                    if is_in_cone
                    {
                        buffer[(texture_offset + 0) as usize] = raw_r;
                        buffer[(texture_offset + 1) as usize] = raw_g;
                        buffer[(texture_offset + 2) as usize] = raw_b;
                    }
                    else
                    {
                        let gray = ((raw_r as f32 + raw_g as f32 + raw_b as f32) / 3.0) as u8;

                        buffer[(texture_offset + 0) as usize] = gray;
                        buffer[(texture_offset + 1) as usize] = gray;
                        buffer[(texture_offset + 2) as usize] = gray;
                    }
                }
            }
        }).unwrap();

        //we don't need to clear the screen here because we will fill the screen with the new
        //texture anyway

        //Render the new texture on the screen
        renderer.copy(&game_texture, None, None);

        renderer.present();
    }
}

pub struct InputSystem {
    event_pump: EventPump,
    mouse_pos: Vector2<f32>,

    pub should_exit: bool,
}

impl System for InputSystem {
    type Components = MyComponents;
    type Services = ();
}

impl EntityProcess for InputSystem {
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, ()>)
    {
        //Run the event loop and store all the keycodes that were pressed
        let mut keys = Vec::<(Keycode, bool)>::new();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    self.should_exit = true;
                    return;
                },
                Event::KeyDown { keycode: Some(code), .. } => {
                    keys.push((code, true));
                },
                Event::KeyUp { keycode: Some(code), .. } => {
                    keys.push((code, false));
                }
                Event::MouseMotion{x: x, y: y, ..} => {
                    self.mouse_pos = Vector2::new(x as f32, y as f32);
                }
                _ => {}
            }
        }

        for e in entities
        {
            for key in &keys 
            {
                let keycode: Option<player::Keys> = match key.0{
                    Keycode::W => Some(player::Keys::Up),
                    Keycode::S => Some(player::Keys::Down),
                    Keycode::D => Some(player::Keys::Right),
                    Keycode::A => Some(player::Keys::Left),
                    _ => None
                };

                match keycode{
                    Some(code) => data.player_component[e].set_key(code, key.1),
                    None => {}
                };
            }

            let add_vel = 0.3;
            //All keys have been handled, let's use them
            if data.player_component[e].get_key(player::Keys::Up)
            {
                data.velocity[e] += Vector2::new(0.0, -add_vel);
            }
            if data.player_component[e].get_key(player::Keys::Down)
            {
                data.velocity[e] += Vector2::new(0.0, add_vel);
            }
            if data.player_component[e].get_key(player::Keys::Right)
            {
                data.velocity[e] += Vector2::new(add_vel, 0.0);
            }
            if data.player_component[e].get_key(player::Keys::Left)
            {
                data.velocity[e] += Vector2::new(-add_vel, 0.0);
            }

            let pos_diff = self.mouse_pos / 2.0 - data.transform[e].pos;

            data.transform[e].angle = pos_diff.y.atan2(pos_diff.x) as f64;
        }
    }
}

pub struct CollisionSystem {
    pub player: Entity,
}

impl System for CollisionSystem {
    type Components = MyComponents;
    type Services = ();
    }

fn are_colliding(tr1: Transform, bb1: BoundingBox, tr2: Transform, bb2: BoundingBox) -> bool {
    let left1 = tr1.pos.x + bb1.offset.x;
    let left2 = tr2.pos.x + bb2.offset.x;
    let right1 = left1 + bb1.size.x;
    let right2 = left2 + bb2.size.x;

    let top1 = tr1.pos.y + bb1.offset.y;
    let top2 = tr2.pos.y + bb2.offset.y;
    let bottom1 = top1 + bb1.size.y;
    let bottom2 = top2 + bb2.size.y;

    if left1 > right2 {return false};
    if left2 > right1 {return false};
    if top1 > bottom2 {return false};
    if top2 > bottom1 {return false};

    true
}

impl EntityProcess for CollisionSystem {
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, ()>)
    {
        let mut player_transform: Transform = Default::default();
        let mut player_box: BoundingBox = Default::default();
        data.with_entity_data(&self.player, |entity, data| {
            player_transform = data.transform[entity];
            player_box = data.bounding_box[entity];
        });

        for e in entities {
            let transform = data.transform[e];
            let bounding_box = data.bounding_box[e];

            if are_colliding(player_transform, player_box, transform, bounding_box) {
                println!("A collision has occurred!");
            }
        }
    }
}

pub struct MotionSystem;

impl System for MotionSystem {
    type Components = MyComponents;
    type Services = ();
}

impl EntityProcess for MotionSystem {
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, ()>)
    {
        for e in entities {
            let velocity = data.velocity[e];
            data.transform[e].pos += velocity;
        }
    }
}

pub struct MaxVelSystem;

impl System for MaxVelSystem {
    type Components = MyComponents;
    type Services = ();
}

impl EntityProcess for MaxVelSystem
{
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, ()>)
    {
        for e in entities {
            let velocity = data.velocity[e];
            let max_vel = data.max_velocity[e];
            
            if velocity.x * velocity.x + velocity.y * velocity.y > max_vel * max_vel
            {
                data.velocity[e] = velocity.normalize() * max_vel;
            }
        }
    }
}

pub struct RespawnSystem;

impl System for RespawnSystem {
    type Components = MyComponents;
    type Services = ();
}

impl EntityProcess for RespawnSystem
{
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, ()>)
    {
        let center = Vector2::new(RESOLUTION.0, RESOLUTION.1);
        let mut rng = rand::thread_rng();

        for e in entities 
        {
            let diff = Vector2::new(data.transform[e].pos.x - center.x as f32, data.transform[e].pos.y - center.y as f32);

            if diff.x.powi(2) + diff.x.powi(2) >  data.respawn_component[e].max_radius.powi(2)
            {
                //select a random position
                let angle = rng.gen_range(0., consts::PI*2.) as f32;

                data.transform[e].pos = Vector2::new((data.respawn_component[e].max_radius - 1.) * angle.cos(), 
                                                     (data.respawn_component[e].max_radius - 1.) * angle.sin());

                //select a random direction
                let angle = rng.gen_range(0., consts::PI*2.) as f32;
                let speed = rng.gen_range(0., data.respawn_component[e].max_speed);
                data.velocity[e] = Vector2::new(speed * angle.cos(), speed * angle.sin());
            }
        }
    }
}


components! {
    struct MyComponents {
        #[hot] transform: Transform,
        #[hot] velocity: Vector2<f32>,
        #[hot] max_velocity: f32,
        #[hot] sprite: Sprite,
        #[hot] bounding_box: BoundingBox,
        #[cold] player_component: PlayerComponent,
        #[cold] respawn_component: RespawnComponent,
    }
}

systems! {
    // struct MySystems<MyComponents, ()>;
    struct MySystems<MyComponents, ()> {
        active: {
            rendering: LazySystem<EntitySystem<RenderingSystem<'static>>> = LazySystem::new(),
            input: LazySystem<EntitySystem<InputSystem>> = LazySystem::new(),
            collision: LazySystem<EntitySystem<CollisionSystem>> = LazySystem::new(),
            motion: EntitySystem<MotionSystem> = EntitySystem::new(
                MotionSystem,
                aspect!(<MyComponents> all: [transform, velocity])
            ),
            max_vel: EntitySystem<MaxVelSystem> = EntitySystem::new(
                MaxVelSystem{}, 
                aspect!(<MyComponents> all: [velocity, max_velocity])
            ),
            respawn: EntitySystem<RespawnSystem> = EntitySystem::new(
                RespawnSystem{},
                aspect!(<MyComponents> all: [velocity, transform, respawn_component])
            ),
        },
        passive: {}
    }
}


pub fn create_world<'a>(renderer: Renderer<'static>,
                        game_renderer: Renderer<'static>, event_pump: EventPump) -> World<MySystems>
{
    let mut world = World::<MySystems>::new();

    let good_texture = Rc::new(load_texture(&game_renderer, String::from("data/good.png")));
    let neutral_texture = Rc::new(
        load_texture(&game_renderer, String::from("data/neutral.png")));
    let bad_texture = Rc::new(load_texture(&game_renderer, String::from("data/bad.png")));

    let test_sprite = Sprite::new(good_texture);
    let test_sprite2 = Sprite::new(neutral_texture);
    let test_sprite3 = Sprite::new(bad_texture);

    let player_transform = Transform { pos: Vector2::new(0.0, 0.0), angle: 0.0,
                                 scale: Vector2::new(0.5, 0.5) };
    let transform2 = Transform { pos: Vector2::new(150.0, 150.0), angle: 0.0,
                                 scale: Vector2::new(0.5, 0.5) };
    let transform3 = Transform { pos: Vector2::new(150.0, 300.0), angle: 0.0,
                                 scale: Vector2::new(0.5, 0.5) };

    let player_box = BoundingBox { offset: Vector2::new(0.0, 0.0),
                                   size: Vector2::new(10.0, 10.0) };

    let respawn_comp = RespawnComponent{max_radius: 150.0, max_speed: 1.0};


    // Create some entites with some components
    let player_entity = world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
            data.velocity.add(&entity, Vector2::new(0.0, 0.0));
            data.max_velocity.add(&entity, 4.0);
            data.transform.add(&entity, player_transform);
            data.sprite.add(&entity, test_sprite);
            data.player_component.add(&entity, PlayerComponent::new());
            data.bounding_box.add(&entity, player_box);
        }
    );

    world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
            data.transform.add(&entity, transform2);
            data.sprite.add(&entity, test_sprite2);
        }
    );

    world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
            data.transform.add(&entity, transform3);
            data.velocity.add(&entity, Vector2::new(0.0, 0.0));
            data.sprite.add(&entity, test_sprite3);
            data.respawn_component.add(&entity, respawn_comp.clone());
        }
    );

    let renderref = RefCell::new(renderer);
    let game_renderref = RefCell::new(game_renderer);

    world.systems.rendering.init(EntitySystem::new(
        RenderingSystem {renderer: renderref, game_renderer: game_renderref, player: player_entity},
        aspect!(<MyComponents> all: [transform, sprite])
    ));

    world.systems.input.init(EntitySystem::new(
        InputSystem{event_pump: event_pump, should_exit:false, mouse_pos: na::zero()},
        aspect!(<MyComponents> all: [velocity, player_component]))
    );

    world.systems.collision.init(EntitySystem::new(
        CollisionSystem {player: player_entity},
        aspect!(<MyComponents> all: [transform, bounding_box] none: [player_component])
    ));

    return world;
}

fn is_in_cone(center: Vector2<f32>, x: u32, y: u32, angle: f64) -> bool
{
    let cone_size = 0.07;

    let angle_threshold = cone_size * consts::PI * 2.;

    //Center the pixels
    let x = x as i32 - center.x as i32;
    let y = y as i32 - center.y as i32;

    let pixel_angle = (y as f64).atan2(x as f64);

    let mut angle_diff = (angle - pixel_angle).abs();

    if angle_diff > consts::PI * (1. - cone_size) * 2.
    {
        angle_diff = angle_diff - consts::PI * 2.;
    }

    if angle_diff < angle_threshold
    {
        return true;
    }
    false
}
