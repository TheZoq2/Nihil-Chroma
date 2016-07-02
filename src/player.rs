use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash)]
pub enum Keys
{
    Up,
    Down,
    Left,
    Right,
}

pub struct PlayerComponent
{
    pressed_keys: HashMap<Keys, bool>,
}

impl PlayerComponent
{
    pub fn new() -> PlayerComponent
    {
        PlayerComponent
        {
            pressed_keys: HashMap::new(),
        }
    }

    pub fn set_key(&mut self, key: Keys, pressed: bool)
    {
        self.pressed_keys.insert(key, pressed);
    }
    pub fn get_key(&self, key: Keys) -> bool
    {
        match self.pressed_keys.get(&key)
        {
            Some(down) => down.clone(),
            None => false,
        }
    }
}
