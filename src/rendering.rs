extern crate ecs;

use sdl2::render::{Renderer};
use sdl2::pixels::PixelFormatEnum;
use std::f64::consts;
use std::cell::RefCell;
use ecs::{System, DataHelper, EntityIter};
use ecs::system::{EntityProcess};
use components::{MyServices, MyComponents};
use nalgebra::{Vector2, Norm};

use constants::*;

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


//This doesn't work :/
pub struct StretchSystem;

impl System for StretchSystem{
    type Components = MyComponents;
    type Services = MyServices;
}
impl EntityProcess for StretchSystem
{
    fn process(&mut self, entities: EntityIter<MyComponents>,
                       data: &mut DataHelper<MyComponents, MyServices>)
    {
        for e in entities
        {
            let stretch_amount = data.stretch[e].amount * (data.velocity[e].norm());
            let original_scale = data.stretch[e].original;

            data.transform[e].scale = Vector2::new(
                original_scale.x * (1. + stretch_amount), 
                original_scale.y / (1. + stretch_amount));
        }
    }
}
