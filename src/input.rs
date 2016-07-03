use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use ecs::{System, DataHelper, EntityIter};
use ecs::system::{EntityProcess};

use nalgebra::{Vector2};

use components::{MyServices, MyComponents};

use player;

pub struct InputSystem {
    pub event_pump: EventPump,

    pub should_exit: bool,
}

impl System for InputSystem {
    type Components = MyComponents;
    type Services = MyServices;
}

impl EntityProcess for InputSystem {
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, MyServices>)
    {
        //Run the event loop and store all the keycodes that were pressed
        let mut keys = Vec::<(Keycode, bool)>::new();

        let mut mouse_pos = None;

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
                    mouse_pos = Some(Vector2::new(x as f32, y as f32));
                }
                _ => {}
            }
        }

        for e in entities
        {
            for key in &keys
            {
                let keycode: Option<player::Keys> = match key.0{
                    Keycode::W => Some(player::Keys::Up),
                    Keycode::S => Some(player::Keys::Down),
                    Keycode::D => Some(player::Keys::Right),
                    Keycode::A => Some(player::Keys::Left),
                    _ => None
                };

                match keycode{
                    Some(code) => data.player_component[e].set_key(code, key.1),
                    None => {}
                };
            }

            //All keys have been handled, let's use them
            if data.player_component[e].get_key(player::Keys::Up)
            {
                data.transform[e].pos += Vector2::new(0.0, -1.0);
            }
            if data.player_component[e].get_key(player::Keys::Down)
            {
                data.transform[e].pos += Vector2::new(0.0, 1.0);
            }
            if data.player_component[e].get_key(player::Keys::Right)
            {
                data.transform[e].pos += Vector2::new(1.0, 0.0);
            }
            if data.player_component[e].get_key(player::Keys::Left)
            {
                data.transform[e].pos += Vector2::new(-1.0, 0.0);
            }

            match mouse_pos
            {
                Some(pos) => {
                    let pos_diff = pos / 2.0 - data.transform[e].pos;

                    data.transform[e].angle = pos_diff.y.atan2(pos_diff.x) as f64;
                },
                None => {}
            }
        }
    }
}
