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

    let mut result = renderer.create_texture_streaming(
        PixelFormatEnum::RGBA8888, img.dimensions().0, img.dimensions().1).unwrap();

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform
{
    pub pos: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub angle: f64,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            pos: Vector2::new(0.0, 0.0),
            angle: 0.0,
            scale: Vector2::new(1.0, 1.0)
        }
    }
}

pub struct Sprite
{
    texture: Rc<Texture>,

    depth: i32,
}

impl Sprite
{
    pub fn new(texture: Rc<Texture>) -> Sprite
    {
        let result = Sprite {
            texture: texture,
            depth: 0,
        };

        result
    }
}

impl Sprite
{
    pub fn draw(&self, transform: &Transform, renderer: &mut Renderer)
    {
        //calculating the size value
        let sizex = transform.scale.x * self.texture.query().width as f32;
        let sizey = transform.scale.y * self.texture.query().height as f32;

        renderer.copy_ex(
            &self.texture,
            None,
            Some(Rect::new(transform.pos.x as i32, transform.pos.y as i32,
                           sizex as u32, sizey as u32)),
            transform.angle.to_degrees(),
            None,
            false,
            false).unwrap();
    }
}
