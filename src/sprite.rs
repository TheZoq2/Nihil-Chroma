#![allow(dead_code)]

extern crate nalgebra as na;

use sdl2::rect::{Rect};
use sdl2::render::{Renderer, Texture};

use std::rc::{Rc};

use nalgebra::Vector2;

pub struct Sprite 
{
    texture: Rc<Texture>,

    pos: Vector2<f32>,
    scale: Vector2<f32>,
    angle: f64,

    size: Vector2<f32>,
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

    pub fn draw(&self, target: &mut Renderer) 
    {
        target.copy_ex(
            &self.texture, 
            None, 
            Some(Rect::new(self.pos.x as i32, self.pos.y as i32, self.size.x as u32, self.size.y as u32)),
            self.angle,
            None,
            false,
            false).unwrap();
    }
}

