extern crate ecs;
extern crate nalgebra;

use sdl2::render::{Renderer};
use ecs::{World, BuildData, System, Process, DataHelper, EntityIter};
use sprite::{Sprite, load_texture};
use std::rc::Rc;
use nalgebra::{Vector2};
use ecs::system::{EntityProcess, EntitySystem};

pub struct RenderingSystem<'a> {
    pub renderer: Option<Renderer<'a>>,
}

impl<'a> System for RenderingSystem<'a> {
    type Components = MyComponents;
    type Services = ();
}

impl EntityProcess for RenderingSystem<'static> {
    fn process(&mut self, entities: EntityIter<MyComponents>,
                       data: &mut DataHelper<MyComponents, ()>)
    {
        self.renderer.unwrap().clear();

        for e in entities {
            data.sprite[e].draw(&mut self.renderer.unwrap());
        }

        self.renderer.unwrap().present();
    }
}

components! {
    struct MyComponents {
        #[hot] position: Vector2<f32>,
        #[hot] sprite: Sprite,
        #[hot] angle: f32,
    }
}

systems! {
    struct MySystems<MyComponents, ()> {
        active: {
            motion: EntitySystem<RenderingSystem<'static>> = EntitySystem::new(
                RenderingSystem { renderer: None },
                aspect!(<MyComponents> all: [position, sprite])
            ),
        },
        passive: {}
    }
}


pub fn create_world(renderer: &Renderer) -> World<MySystems>
{
    let mut world = World::<MySystems>::new();

    let texture = Rc::new(load_texture(&renderer, String::from("data/test.png")));
    let texture2 = Rc::new(load_texture(&renderer, String::from("data/test2.png")));

    let mut test_sprite = Sprite::new(texture);
    let mut test_sprite2 = Sprite::new(texture2);
    test_sprite2.set_position(Vector2::new(150.0, 150.0));
    test_sprite2.set_scale(Vector2::new(0.5, 0.5));


    // Create some entites with some components
    let entity1 = world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
            data.position.add(&entity, Vector2::new(0.0, 0.0));
            data.sprite.add(&entity, test_sprite);
            data.angle.add(&entity, 0.0);
        }
    );

    let entity2 = world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
            data.position.add(&entity, Vector2::new(0.0, 0.0));
            data.sprite.add(&entity, test_sprite2);
        }
    );

    return world;
}
