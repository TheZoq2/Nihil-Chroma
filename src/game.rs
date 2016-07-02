extern crate ecs;
extern crate nalgebra;

use sdl2::render::{Renderer};
use sdl2::pixels::PixelFormatEnum;
use std::rc::Rc;
use std::cell::RefCell;
use std::f64::consts;
use nalgebra::{Vector2};

use ecs::{World, BuildData, System, DataHelper, EntityIter};
use ecs::system::{EntityProcess, EntitySystem, LazySystem};

use sprite::{Sprite, Transform, load_texture};
use constants::*;

pub struct RenderingSystem<'a> {
    pub renderer: RefCell<Renderer<'a>>,
    pub game_renderer: RefCell<Renderer<'a>>,
    pub angle: f64,
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

                    let is_in_cone = is_in_cone(x, y, self.angle);

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

        self.angle += 0.03;

        if self.angle > consts::PI * 2.0
        {
            self.angle = self.angle - consts::PI * 2.0;
        }
        else if self.angle < 0.0
        {
            self.angle = self.angle + consts::PI * 2.0;
        }
    }
}

components! {
    struct MyComponents {
        #[hot] transform: Transform,
        #[hot] sprite: Sprite,
    }
}

systems! {
    // struct MySystems<MyComponents, ()>;
    struct MySystems<MyComponents, ()> {
        active: {
            rendering: LazySystem<EntitySystem<RenderingSystem<'static>>> = LazySystem::new(),
        },
        passive: {}
    }
}


pub fn create_world<'a>(renderer: Renderer<'static>,
                        game_renderer: Renderer<'static>) -> World<MySystems>
{
    let mut world = World::<MySystems>::new();

    let texture = Rc::new(load_texture(&game_renderer, String::from("data/test.png")));
    let texture2 = Rc::new(load_texture(&game_renderer, String::from("data/test2.png")));

    let test_sprite = Sprite::new(texture);
    let test_sprite2 = Sprite::new(texture2);

    let transform1 = Transform { pos: Vector2::new(0.0, 0.0), angle: 0.0,
                                 scale: Vector2::new(1.0, 1.0) };
    let transform2 = Transform { pos: Vector2::new(150.0, 150.0), angle: 0.0,
                                 scale: Vector2::new(0.5, 0.5) };


    // Create some entites with some components
    world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
            data.transform.add(&entity, transform1);
            data.sprite.add(&entity, test_sprite);
        }
    );

    world.create_entity(
        |entity: BuildData<MyComponents>, data: &mut MyComponents| {
            data.transform.add(&entity, transform2);
            data.sprite.add(&entity, test_sprite2);
        }
    );

    let renderref = RefCell::new(renderer);
    let game_renderref = RefCell::new(game_renderer);

    world.systems.rendering.init(EntitySystem::new(
        RenderingSystem {renderer: renderref, game_renderer: game_renderref, angle: 0.0},
        aspect!(<MyComponents> all: [transform, sprite])
    ));

    return world;
}

fn is_in_cone(x: u32, y: u32, angle: f64) -> bool
{
    let cone_size = 0.07;

    let angle_threshold = cone_size * consts::PI * 2.;

    //Center the pixels
    let x = x as i32 - RESOLUTION.0 as i32 / 2;
    let y = y as i32 - RESOLUTION.1 as i32 / 2;

    let pixel_angle = (y as f64).atan2(x as f64) + consts::PI;

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
