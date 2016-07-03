extern crate ecs;
extern crate rand;

use rand::Rng;
use sdl2::render::{Renderer};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::f64::consts;
use std::cell::RefCell;
use ecs::{System, DataHelper, EntityIter};
use ecs::system::{EntityProcess};
use components::{MyServices, MyComponents};
use nalgebra::{Vector2};

use constants::*;

pub struct RenderingSystem<'a> {
    pub renderer: RefCell<Renderer<'a>>,
    pub game_renderer: RefCell<Renderer<'a>>,
    pub player: ecs::Entity,
    pub shake_amount: f32,
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
        
        //Screenshake
        let mut offset = Vector2::new(0, 0);

        let mut rng = rand::thread_rng();

        if data.services.hit_bad == true
        {
            self.shake_amount = 10.;
        }


        if self.shake_amount > 0.
        {
            offset = Vector2::new(
                rng.gen_range(-self.shake_amount, self.shake_amount) as i32,
                rng.gen_range(-self.shake_amount, self.shake_amount) as i32
            );

            self.shake_amount = self.shake_amount - 0.2;
        }
        else if self.shake_amount < 5.
        {
            self.shake_amount = 0.
        }

        //Add outside screenshake stimulation
        match data.services.screenshake
        {
            Some(amount) => self.shake_amount = amount,
            None => {}
        }


        let size = renderer.output_size().unwrap();
        let screen_rect = Rect::new(offset.x, offset.y, size.0 as u32, size.1 as u32);



        //Render the new texture on the screen
        renderer.copy(&game_texture, Some(screen_rect), None);

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


