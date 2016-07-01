#![allow(dead_code)]

extern crate sdl2;
extern crate nalgebra as na;
extern crate image;

use sdl2::rect::{Rect};
use sdl2::render::{Renderer, Texture, BlendMode};
use sdl2::pixels::{PixelFormatEnum};
use image::GenericImage;

use std::rc::{Rc};
use std::path::Path;

use nalgebra::Vector2;

//Loads a texture from a file and returns an SDL2 texture object
pub fn load_texture(renderer: &Renderer, path: String) -> sdl2::render::Texture
{
    let img = image::open(&Path::new(&path)).unwrap();

    let mut result = renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, img.dimensions().0, img.dimensions().1)
                    .unwrap();

    result.set_blend_mode(BlendMode::Blend);

    //Take the pixels from the image and put them on the texture
    result.with_lock(None, |buffer: &mut [u8], pitch: usize|{
        for y in 0..img.dimensions().0
        {
            for x in 0..img.dimensions().1
            {
                let offset = y*pitch as u32 + x*4;
                buffer[(offset + 0) as usize] = img.get_pixel(x, y)[3]; //A
                buffer[(offset + 1) as usize] = img.get_pixel(x, y)[2]; //B
                buffer[(offset + 2) as usize] = img.get_pixel(x, y)[1]; //G
                buffer[(offset + 3) as usize] = img.get_pixel(x, y)[0]; //R
            }
        }
    }).unwrap();

    result
}


pub struct Sprite
{
    texture: Rc<Texture>,

    pos: Vector2<f32>,
    scale: Vector2<f32>,
    angle: f64,

    size: Vector2<f32>,

    depth: i32,
}

impl Sprite
{
    pub fn new(texture: Rc<Texture>) -> Sprite
    {
        let mut result = Sprite {
            texture: texture,

            pos: na::zero(),
            scale: na::one(),
            angle: 0.0,

            size: na::one(),

            depth: 0,
        };

        result.set_scale(Vector2::new(1.0, 1.0));

        result
    }

    pub fn set_angle(&mut self, angle: f64)
    {
        self.angle = angle;
    }
    pub fn get_angle(&self) -> f64
    {
        return self.angle;
    }

    pub fn set_position(&mut self, pos: Vector2<f32>)
    {
        self.pos = pos;
    }
    pub fn get_position(&self) -> Vector2<f32>
    {
        return self.pos;
    }

    pub fn set_scale(&mut self, scale: Vector2<f32>)
    {
        //Storing the scale value
        self.scale = scale;

        //calculating the size value
        self.size.x = self.scale.x * self.texture.query().width as f32;
        self.size.y = self.scale.y * self.texture.query().height as f32;
    }
    pub fn get_scale(&self) -> Vector2<f32>
    {
        return self.scale;
    }

}

impl Sprite
{
    pub fn draw(&self, renderer: &mut Renderer)
    {
        renderer.copy_ex(
            &self.texture,
            None,
            Some(Rect::new(self.pos.x as i32, self.pos.y as i32, self.size.x as u32, self.size.y as u32)),
            self.angle,
            None,
            false,
            false).unwrap();
    }
}
