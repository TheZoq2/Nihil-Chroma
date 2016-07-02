extern crate ecs;
extern crate nalgebra;
extern crate rand;

use sdl2::render::{Renderer, Texture};
use sdl2::pixels::PixelFormatEnum;
use std::rc::Rc;
use std::cell::RefCell;
use std::f64::consts;
use std::ops::Deref;
use nalgebra::{Vector2};

use rand::distributions::{IndependentSample, Range};

use ecs::{ServiceManager, Entity, World, BuildData, System, DataHelper, EntityIter};
use ecs::system::{EntityProcess, EntitySystem, LazySystem};

use sprite::{Sprite, Transform, load_texture};
use constants::*;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use player::{PlayerComponent};
use player;

pub struct MyServices {
    // pub remove_entity: Vec<Entity>,
    pub too_few_obamas: bool,
}

impl ServiceManager for MyServices {
}

impl Default for MyServices {
    fn default() -> MyServices{
        MyServices {
            // remove_entity: Vec::new(),
            too_few_obamas: false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoundingCircle {
    radius: f32
}

impl Default for BoundingCircle {
    fn default() -> BoundingCircle {
        BoundingCircle {
            radius: 1.0
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
    type Services = MyServices;
}

impl<'a> EntityProcess for RenderingSystem<'a> {
    fn process(&mut self, entities: EntityIter<MyComponents>,
                       data: &mut DataHelper<MyComponents, MyServices>)
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

        // println!("{}", plr_angle);

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

    pub should_exit: bool,
}

impl System for InputSystem {
    type Components = MyComponents;
    type Services = MyServices;
}

impl EntityProcess for InputSystem {
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, MyServices>)
    {
        //Run the event loop and store all the keycodes that were pressed
        let mut keys = Vec::<(Keycode, bool)>::new();

        let mut mouse_pos = None;

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
                Event::MouseMotion{x, y, ..} => {
                    mouse_pos = Some(Vector2::new(x as f32, y as f32));
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

            //All keys have been handled, let's use them
            if data.player_component[e].get_key(player::Keys::Up)
            {
                data.transform[e].pos += Vector2::new(0.0, -1.0);
            }
            if data.player_component[e].get_key(player::Keys::Down)
            {
                data.transform[e].pos += Vector2::new(0.0, 1.0);
            }
            if data.player_component[e].get_key(player::Keys::Right)
            {
                data.transform[e].pos += Vector2::new(1.0, 0.0);
            }
            if data.player_component[e].get_key(player::Keys::Left)
            {
                data.transform[e].pos += Vector2::new(-1.0, 0.0);
            }

            match mouse_pos
            {
                Some(pos) => {
                    let pos_diff = pos / 2.0 - data.transform[e].pos;

                    data.transform[e].angle = pos_diff.y.atan2(pos_diff.x) as f64;
                },
                None => {}
            }
        }
    }
}

pub struct CollisionSystem {
    pub player: Entity,
}

impl System for CollisionSystem {
    type Components = MyComponents;
    type Services = MyServices;
}

fn are_colliding(tr1: &Transform, bb1: &BoundingCircle,
                 tr2: &Transform, bb2: &BoundingCircle) -> bool
{
    let xdist = (tr1.pos.x - tr2.pos.x).abs();
    let ydist = (tr1.pos.y - tr2.pos.y).abs();

    let r = bb1.radius + bb2.radius;

    xdist * xdist + ydist * ydist < r * r
}

impl EntityProcess for CollisionSystem {
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, MyServices>)
    {
        let mut player_transform: Transform = Default::default();
        let mut player_box: BoundingCircle = Default::default();
        data.with_entity_data(&self.player, |entity, data| {
            player_transform = data.transform[entity];
            player_box = data.bounding_box[entity];
        });

        for e in entities {
            let transform = data.transform[e];
            let bounding_box = data.bounding_box[e];

            if are_colliding(&player_transform, &player_box, &transform, &bounding_box) {
                // println!("A collision has occurred!");
            }
        }
    }
}

pub struct MotionSystem;

impl System for MotionSystem {
    type Components = MyComponents;
    type Services = MyServices;
}

impl EntityProcess for MotionSystem {
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, MyServices>)
    {
        for e in entities {
            let velocity = data.velocity[e];
            data.transform[e].pos += velocity;
        }
    }
}

pub struct ObamaSystem;
pub struct ObamaComponent;

impl System for ObamaSystem {
    type Components = MyComponents;
    type Services = MyServices;
}

fn way_off_screen(pos: Vector2<f32>) -> bool {
    (pos.x as i32) < -100 || pos.x as u32 > RESOLUTION.0 + 100 ||
        (pos.y as i32) < -100 || pos.y as u32 > RESOLUTION.1 + 100
}

impl EntityProcess for ObamaSystem {
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, MyServices>)
    {
        let mut obama_amount = 0;

        for e in entities {
            obama_amount += 1;

            // Remove obamas that are too far out
            let position = data.transform[e].pos;
            if way_off_screen(position) {
                data.remove_entity(e.deref().deref().clone());
            }
        }

        data.services.too_few_obamas = obama_amount < 4;
    }
}


components! {
    struct MyComponents {
        #[hot] transform: Transform,
        #[hot] velocity: Vector2<f32>,
        #[hot] sprite: Sprite,
        #[hot] bounding_box: BoundingCircle,
        #[cold] player_component: PlayerComponent,
        #[cold] obama: ObamaComponent,
    }
}

systems! {
    // struct MySystems<MyComponents, MyServices>;
    struct MySystems<MyComponents, MyServices> {
        active: {
            rendering: LazySystem<EntitySystem<RenderingSystem<'static>>> = LazySystem::new(),
            input: LazySystem<EntitySystem<InputSystem>> = LazySystem::new(),
            collision: LazySystem<EntitySystem<CollisionSystem>> = LazySystem::new(),
            motion: EntitySystem<MotionSystem> = EntitySystem::new(
                MotionSystem,
                aspect!(<MyComponents> all: [transform, velocity])
            ),
            obama: EntitySystem<ObamaSystem> = EntitySystem::new(
                ObamaSystem,
                aspect!(<MyComponents> all: [obama, transform])
            ),
        },
        passive: {}
    }
}

pub fn create_obama(world: &mut World<MySystems>, obama_texture: &Rc<Texture>)
{
    // Distance to corners going clockwise
    let c1 = RESOLUTION.0;
    let c2 = c1 + RESOLUTION.1;
    let c3 = c2 + RESOLUTION.0;

    let between_corners = Range::new(0, RESOLUTION.0*2 + RESOLUTION.1*2);
    let between_angle = Range::new(0.0f32, (2.0*consts::PI) as f32);
    let mut rng = rand::thread_rng();

    let rand_dist = between_corners.ind_sample(&mut rng);
    let obama_pos = if rand_dist < c1 {
        Vector2::new(rand_dist as f32, 0.0)
    } else if rand_dist < c2 {
        Vector2::new(RESOLUTION.0 as f32, (rand_dist - c1) as f32)
    } else if rand_dist < c3 {
        Vector2::new(RESOLUTION.1 as f32, (rand_dist - c2) as f32)
    } else {
        Vector2::new(0.0, (rand_dist - c3) as f32)
    };

    let random_angle = between_angle.ind_sample(&mut rng);
    let random_velocity = Vector2::new(random_angle.cos()*2.0, random_angle.sin()*2.0);

    let obama_sprite = Sprite::new(obama_texture.clone());
    let obama_transform = Transform { pos: obama_pos, angle: 0.0,
                                      scale: Vector2::new(0.5, 0.5) };

    world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
            data.sprite.add(&entity, obama_sprite);
            data.transform.add(&entity, obama_transform);
            data.velocity.add(&entity, random_velocity);
            data.obama.add(&entity, ObamaComponent);
        }
    );
}


pub fn create_world(renderer: Renderer<'static>,
                    game_renderer: Renderer<'static>,
                    event_pump: EventPump) -> World<MySystems>
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

    let player_box = BoundingCircle { radius: 28.0 };


    // Create some entites with some components
    let player_entity = world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
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
            data.bounding_box.add(&entity, player_box);
        }
    );

    world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
            data.transform.add(&entity, transform3);
            data.sprite.add(&entity, test_sprite3);
            data.bounding_box.add(&entity, player_box);
        }
    );

    let renderref = RefCell::new(renderer);
    let game_renderref = RefCell::new(game_renderer);

    world.systems.rendering.init(EntitySystem::new(
        RenderingSystem {renderer: renderref, game_renderer: game_renderref,
                         player: player_entity},
        aspect!(<MyComponents> all: [transform, sprite])
    ));

    world.systems.input.init(EntitySystem::new(
        InputSystem{event_pump: event_pump, should_exit:false},
        aspect!(<MyComponents> all: [transform, player_component]))
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
