extern crate image;
extern crate sdl2;

extern crate nalgebra;

mod sprite;
mod drawer;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Renderer, BlendMode};

use std::string::String;

use std::path::Path;

use image::GenericImage;

use std::rc::Rc;

use nalgebra::Vector2;

//Loads a texture from a file and returns an SDL2 texture object
fn load_texture(renderer: &Renderer, path: String) -> sdl2::render::Texture
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

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();

    let texture = Rc::new(load_texture(&renderer, String::from("data/test.png")));
    let texture2 = Rc::new(load_texture(&renderer, String::from("data/test2.png")));

    let mut test_sprite = sprite::Sprite::new(texture);
    let mut test_sprite2 = sprite::Sprite::new(texture2);
    test_sprite2.set_position(Vector2::new(150.0, 150.0));
    test_sprite2.set_scale(Vector2::new(0.5, 0.5));

    renderer.set_draw_color(Color::RGB(100, 100, 0));
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut angle = 0.0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        renderer.clear();

        test_sprite.set_angle(angle);
        test_sprite.draw(&mut renderer);

        test_sprite2.set_angle(angle);
        test_sprite2.draw(&mut renderer);
        
        angle += 0.01;
        renderer.present();
    }
}
