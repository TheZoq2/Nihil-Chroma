extern crate nalgebra as na;
extern crate sdl2;
extern crate specs;

use components::Transform;

use sdl2::rect::{Rect};
use sdl2::render::{Texture, Canvas, RenderTarget};
use specs::VecStorage;

use std::collections::HashMap;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct TextureId(isize);

impl TextureId {
    pub fn next(&self) -> TextureId {
        TextureId(self.0 + 1)
    }
}

pub struct TextureRegistry<'a> {
    next_id: TextureId,
    textures: HashMap<TextureId, Texture<'a>>,
}

impl<'a> TextureRegistry<'a> {
    pub fn new() -> TextureRegistry<'a> {
        TextureRegistry {
            next_id: TextureId(0),
            textures: HashMap::new(),
        }
    }

    pub fn add(&mut self, texture: Texture<'a>) -> TextureId {
        let id = self.next_id;
        self.next_id = id.next();
        self.textures.insert(id, texture);
        id
    }

    pub fn replace(&mut self, id: TextureId, new_texture: Texture<'a>) {
        self.textures.insert(id, new_texture);
    }

    pub fn remove(&mut self, id: TextureId) {
        self.textures.remove(&id);
    }

    pub fn get(&self, id: TextureId) -> Option<&Texture<'a>> {
        self.textures.get(&id)
    }
}

#[derive(Copy, Clone, Component)]
#[component(VecStorage)]
pub struct Sprite {
    pub texture_id: TextureId,
    pub depth: i32,
}

impl Sprite {
    pub fn new(texture_id: TextureId) -> Sprite {
        Sprite {
            texture_id,
            depth: 0,
        }
    }

    pub fn draw<T: RenderTarget>(&self, transform: &Transform, canvas: &mut Canvas<T>, texture_registry: &TextureRegistry) {
        let texture = texture_registry.get(self.texture_id).unwrap();
        //calculating the size value
        let sizex = transform.scale.x * texture.query().width as f32;
        let sizey = transform.scale.y * texture.query().height as f32;

        canvas.copy_ex(
            &texture,
            None,
            Some(Rect::new((transform.pos.x - sizex / 2.)as i32, (transform.pos.y - sizey / 2.) as i32,
                           sizex as u32, sizey as u32)),
            transform.angle.to_degrees(),
            None,
            false,
            false).unwrap();
    }
}
