extern crate specs;

use nalgebra::Vector2;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use specs::Join;

use components::{Transform, Velocity};
use constants::*;
use player;
use player::PlayerComponent;

pub struct InputSystem {
    pub event_pump: EventPump,
    pub mouse_pos: Vector2<f32>,

    pub should_exit: bool
}

impl<'a> specs::System<'a> for InputSystem {
    type SystemData = (
        specs::WriteStorage<'a, PlayerComponent>,
        specs::WriteStorage<'a, Velocity>,
        specs::WriteStorage<'a, Transform>,
    );
    fn run(&mut self, (mut players, mut velocities, mut transforms): Self::SystemData) {
        //Run the event loop and store all the keycodes that were pressed
        let mut keys = Vec::<(Keycode, bool)>::new();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    self.should_exit = true;
                    return;
                },
                Event::KeyDown { keycode: Some(code), .. } => {
                    keys.push((code, true));
                },
                Event::KeyUp { keycode: Some(code), .. } => {
                    keys.push((code, false));
                }
                Event::MouseMotion{x, y, ..} => {
                    self.mouse_pos = Vector2::new(x as f32, y as f32);
                }
                _ => {}
            }
        }

        for (mut player_component, mut velocity, mut transform) in (&mut players, &mut velocities, &mut transforms).join() {
            for key in &keys {
                let keycode: Option<player::Keys> = match key.0{
                    Keycode::W => Some(player::Keys::Up),
                    Keycode::S => Some(player::Keys::Down),
                    Keycode::D => Some(player::Keys::Right),
                    Keycode::A => Some(player::Keys::Left),
                    _ => None
                };

                match keycode{
                    Some(code) => player_component.set_key(code, key.1),
                    None => {}
                };
            }

            let add_vel = 15.;
            //All keys have been handled, let's use them
            if player_component.get_key(player::Keys::Up) {
                velocity.0 += Vector2::new(0.0, -add_vel);
            }
            if player_component.get_key(player::Keys::Down) {
                velocity.0 += Vector2::new(0.0, add_vel);
            }
            if player_component.get_key(player::Keys::Right) {
                velocity.0 += Vector2::new(add_vel, 0.0);
            }
            if player_component.get_key(player::Keys::Left) {
                velocity.0 += Vector2::new(-add_vel, 0.0);
            }

            let pos_diff = self.mouse_pos / UPSCALING as f32 - transform.pos;

            transform.angle = pos_diff.y.atan2(pos_diff.x) as f64;
        }
    }
}
