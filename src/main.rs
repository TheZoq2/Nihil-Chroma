#![allow(dead_code)]
extern crate image;
extern crate sdl2;

extern crate nalgebra;

mod sprite;

use sdl2::pixels::{Color, PixelFormatEnum};
//use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Renderer, BlendMode};
use sdl2::surface::Surface;

use std::string::String;

use std::path::Path;

use image::GenericImage;

use std::rc::Rc;

use nalgebra::Vector2;

use std::f64::consts;

/////////////////////////////////////////////////
//              Global variables
/////////////////////////////////////////////////
const RESOLUTION: (u32, u32) = (800, 450);
const UPSCALING: u32 = 2;

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

    let window = video_subsystem.window("rust-sdl2 demo: Video", RESOLUTION.0 * UPSCALING, RESOLUTION.1 * UPSCALING)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    //The renderer which actually renders to the game window
    let mut renderer = window.renderer().build().unwrap();

    //Renderer where the game is rendered in full color
    let game_surface = Surface::new(RESOLUTION.0, RESOLUTION.1, PixelFormatEnum::RGB888).unwrap();
    let mut game_renderer = Renderer::from_surface(game_surface).unwrap();
    game_renderer.set_draw_color(Color::RGB(150, 150, 0));

    let texture = Rc::new(load_texture(&game_renderer, String::from("data/test.png")));
    let texture2 = Rc::new(load_texture(&game_renderer, String::from("data/test2.png")));


    let mut test_sprite = sprite::Sprite::new(texture);
    let mut test_sprite2 = sprite::Sprite::new(texture2);
    test_sprite2.set_position(Vector2::new(150.0, 150.0));
    test_sprite2.set_scale(Vector2::new(0.5, 0.5));


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

        game_renderer.clear();

        test_sprite.draw(&mut game_renderer);
        test_sprite2.draw(&mut game_renderer);
        
        //Careful, this overshadows the game surface outside the loop.
        let game_surface = game_renderer.surface().unwrap();

        //Creating a new texture to which we will 'copy' the pixels from the game renderer and make
        //some of them grayscale
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

                    let is_in_cone = is_in_cone(x, y, angle);

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
        });

        //we don't need to clear the screen here because we will fill the screen with the new
        //texture anyway
        
        //Render the new texture on the screen
        renderer.copy(&game_texture, None, None);

        renderer.present();
        
        test_sprite.set_angle(angle);
        angle += 0.03;

        if angle > consts::PI * 2.0
        {
            angle = angle - consts::PI * 2.0;
        }
        else if angle < 0.0
        {
            angle = angle + consts::PI * 2.0;
        }
    }
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
