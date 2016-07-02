extern crate ecs;
extern crate nalgebra;

use sdl2::render::{Renderer};
use ecs::{Entity, World, BuildData, System, Process, DataHelper, EntityIter};
use sprite::{Sprite, load_texture};
use std::rc::Rc;
use std::cell::RefCell;
use nalgebra::{Vector2};
use ecs::system::{EntityProcess, EntitySystem, LazySystem};

pub struct RenderingSystem<'a> {
    pub renderer: RefCell<Renderer<'a>>,
}

impl<'a, 'b> System for RenderingSystem<'a> {
    type Components = MyComponents;
    type Services = ();
}

impl<'a, 'b> EntityProcess for RenderingSystem<'a> {
    fn process(&mut self, entities: EntityIter<MyComponents>,
                       data: &mut DataHelper<MyComponents, ()>)
    {
        self.renderer.borrow_mut().clear();

        for e in entities {
            data.sprite[e].draw(&mut self.renderer.borrow_mut());
        }

        self.renderer.borrow_mut().present();
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
    // struct MySystems<MyComponents, ()>;
    struct MySystems<MyComponents, ()> {
        active: {
            // Here I am totally lost on what to do with the lifetime parameters
            rendering: LazySystem<EntitySystem<RenderingSystem<'a>>> = LazySystem::new(),
        },
        passive: {}
    }
}


pub fn create_world<'a>(renderer: Renderer) -> World<MySystems>
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

    let renderref = RefCell::new(renderer);

    world.systems.rendering.init(EntitySystem::new(
        RenderingSystem {renderer: renderref},
        aspect!(<MyComponents> all: [position, sprite])
    ));

    return world;
}

/*
src/game.rs:84:34: 84:42 error: cannot infer an appropriate lifetime due to conflicting requirements [E0495]
src/game.rs:84     let renderref = RefCell::new(renderer);
                                                ^~~~~~~~
src/game.rs:87:36: 87:45 note: first, the lifetime cannot outlive the expression at 87:35...
src/game.rs:87         RenderingSystem {renderer: renderref},
                                                  ^~~~~~~~~
src/game.rs:87:36: 87:45 note: ...so type `std::cell::RefCell<sdl2::render::Renderer<'_>>` of expression is valid during the expression
src/game.rs:87         RenderingSystem {renderer: renderref},
                                                  ^~~~~~~~~
src/game.rs:84:21: 84:43 note: but, the lifetime must be valid for the call at 84:20...
src/game.rs:84     let renderref = RefCell::new(renderer);
                                   ^~~~~~~~~~~~~~~~~~~~~~
src/game.rs:84:21: 84:43 note: ...so type `std::cell::RefCell<sdl2::render::Renderer<'_>>` of expression is valid during the expression
src/game.rs:84     let renderref = RefCell::new(renderer);
*/

/* Compiler errors
src/game.rs:48:64: 48:66 error: use of undeclared lifetime name `'a` [E0261]
src/game.rs:48             rendering: LazySystem<EntitySystem<RenderingSystem<'a, 'b>>> = LazySystem::new(),
^~
src/game.rs:43:1: 52:2 note: in this expansion of systems! (defined in <ecs macros>)
src/game.rs:48:64: 48:66 help: run `rustc --explain E0261` to see a detailed explanation
src/game.rs:48:68: 48:70 error: use of undeclared lifetime name `'b` [E0261]
src/game.rs:48             rendering: LazySystem<EntitySystem<RenderingSystem<'a, 'b>>> = LazySystem::new(),
^~
src/game.rs:43:1: 52:2 note: in this expansion of systems! (defined in <ecs macros>)
src/game.rs:48:68: 48:70 help: run `rustc --explain E0261` to see a detailed explanation
*/

pub fn render_entities(world: &mut World<MySystems>, mut renderer: &mut Renderer)
{
    renderer.clear();

    let mut entity_vec = Vec::new();

    {
        let entities = world.entities().filter(aspect!(<MyComponents> all: [sprite]), &world);

        for entity in entities
        {
            entity_vec.push(entity.clone());
            // world.with_entity_data(&entity, |entity, data| {
            //     data.sprite[entity].draw(&mut renderer);
            // });
        }
    }


    for e in entity_vec
    {
        // world.with_entity_data(&e, |entity, data| {
        //     data.sprite[entity].draw(&mut renderer);
        // });
    }

    renderer.present();
}
