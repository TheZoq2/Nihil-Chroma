extern crate ecs;
extern crate nalgebra;
extern crate rand;

use sdl2::render::{Renderer, Texture};
use std::rc::Rc;
use std::cell::RefCell;
use std::f64::consts;
use std::ops::Deref;
use nalgebra::{Vector2};

use rand::distributions::{IndependentSample, Range};

use ecs::{Entity, World, BuildData, System, DataHelper, EntityIter};
use ecs::system::{EntityProcess, EntitySystem, LazySystem};

use sprite::{Sprite, load_texture};
use constants::*;

use sdl2::EventPump;

use player::{PlayerComponent};

use rendering::RenderingSystem;
use input::InputSystem;
use components::{MyServices, MyComponents, Transform, BoundingCircle, ObamaComponent};

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

fn random_edge_position() -> Vector2<f32>
{
    // Distance to corners going clockwise
    let c1 = RESOLUTION.0; // top left
    let c2 = c1 + RESOLUTION.1; // top right
    let c3 = c2 + RESOLUTION.0; // bottom right

    let mut rng = rand::thread_rng();
    let between_corners = Range::new(0, RESOLUTION.0*2 + RESOLUTION.1*2);
    let rand_dist = between_corners.ind_sample(&mut rng);
    let edge_offset = 50.0;

    if rand_dist < c1 {
        Vector2::new(rand_dist as f32, -edge_offset)
    } else if rand_dist < c2 {
        Vector2::new(RESOLUTION.0 as f32 + edge_offset, (rand_dist - c1) as f32)
    } else if rand_dist < c3 {
        Vector2::new(RESOLUTION.1 as f32 + edge_offset, (rand_dist - c2) as f32)
    } else {
        Vector2::new(edge_offset, (rand_dist - c3) as f32)
    }
}

pub fn create_obama(world: &mut World<MySystems>, obama_texture: &Rc<Texture>)
{
    let between_angle = Range::new(0.0f32, (2.0*consts::PI) as f32);
    let mut rng = rand::thread_rng();

    let obama_pos = random_edge_position();

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
