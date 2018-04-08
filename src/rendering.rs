extern crate specs;
extern crate rand;

use constants::*;
use components::Transform;
use components::{HitBad, ScreenShake};
use sprite::Sprite;

use sdl2::surface::Surface;
use sdl2::render::{Canvas, Texture};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::video::Window;
use std::f64::consts;
use nalgebra::Vector2;
use rand::Rng;
use specs::Join;

pub struct RenderingSystem<'s> {
    pub canvas: Canvas<Window>,
    pub game_texture: Texture,
    pub game_canvas: Canvas<Surface<'s>>,
    pub player: specs::Entity,
    pub shake_amount: f32,
}

impl<'s> RenderingSystem<'s> {
    pub fn new(
        canvas: Canvas<Window>,
        game_canvas: Canvas<Surface<'s>>,
        player: specs::Entity,
        shake_amount: f32,
    ) -> RenderingSystem<'s> {
        // Creating a new texture to which we will 'copy' the pixels from the
        // game renderer and make some of them grayscale
        let game_texture = canvas.texture_creator().create_texture_streaming(
            PixelFormatEnum::RGB888,
            RESOLUTION.0,
            RESOLUTION.1).unwrap();

        RenderingSystem {
            canvas, game_canvas, game_texture, player, shake_amount
        }
    }
}

impl<'a, 's> specs::System<'a> for RenderingSystem<'s> {
    type SystemData = (
        specs::ReadStorage<'a, Transform>,
        specs::ReadStorage<'a, Sprite>,
        specs::Fetch<'a, HitBad>,
        specs::Fetch<'a, ScreenShake>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (transforms, sprites, hit_bad, screenshake) = data;
        //Getting some parameters about the player
        let player_transform = transforms.get(self.player).unwrap();
        let plr_angle = player_transform.angle;
        let player_pos = player_transform.pos;

        self.game_canvas.clear();

        for (transform, sprite) in (&transforms, &sprites).join() {
            sprite.draw(&transform, &mut self.game_canvas);
        }

        let game_surface = self.game_canvas.surface();
        self.game_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            let surface_data = game_surface.without_lock().unwrap();

            for y in 0..RESOLUTION.1 {
                for x in 0..RESOLUTION.0 {
                    let is_in_cone = is_in_cone(player_pos, x, y, plr_angle);

                    //Doing the grayscale stuff
                    let surface_offset = (y) * game_surface.pitch() as u32 + (x) * 4;

                    let raw_r = surface_data[(surface_offset + 0) as usize];
                    let raw_g = surface_data[(surface_offset + 1) as usize];
                    let raw_b = surface_data[(surface_offset + 2) as usize];

                    let texture_offset = y*pitch as u32 + x*4;
                    if is_in_cone {
                        buffer[(texture_offset + 0) as usize] = raw_r;
                        buffer[(texture_offset + 1) as usize] = raw_g;
                        buffer[(texture_offset + 2) as usize] = raw_b;
                    } else {
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

        if hit_bad.0 == true {
            self.shake_amount = 10.;
        }

        if self.shake_amount > 0. {
            offset = Vector2::new(
                rng.gen_range(-self.shake_amount, self.shake_amount) as i32,
                rng.gen_range(-self.shake_amount, self.shake_amount) as i32
            );

            self.shake_amount = self.shake_amount - 0.2;
        } else if self.shake_amount < 5. {
            self.shake_amount = 0.
        }

        //Add outside screenshake stimulation
        match screenshake.0 {
            Some(amount) => self.shake_amount = amount,
            None => {}
        }

        let size = self.canvas.output_size().unwrap();
        let screen_rect = Rect::new(offset.x, offset.y, size.0 as u32, size.1 as u32);

        //Render the new texture on the screen
        self.canvas.copy(&self.game_texture, Some(screen_rect), None).unwrap();

        self.canvas.present();
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
