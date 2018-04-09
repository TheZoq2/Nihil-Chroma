extern crate nalgebra as na;
extern crate sdl2;
extern crate specs;

use components::Transform;

use sdl2::rect::{Rect};
use sdl2::render::{Canvas, RenderTarget, Texture, TextureCreator};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::ttf::Font;
use specs::VecStorage;

use std::collections::HashMap;
use std::rc::Rc;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct TextureId(isize);

pub trait Key {
    fn new() -> Self;
    fn next(&self) -> Self;
}

impl TextureId {
    fn new() -> TextureId {
        TextureId(0)
    }

    fn next(&self) -> TextureId {
        TextureId(self.0 + 1)
    }
}

pub struct TextureManager<'l, T: 'l>
{
    loader: &'l TextureCreator<T>,
    cache: HashMap<String, TextureId>,
    next_key: TextureId,
    storage: HashMap<TextureId, Rc<Texture<'l>>>,
}

impl<'l, T> TextureManager<'l, T>
{
    pub fn new(loader: &'l TextureCreator<T>) -> Self {
        TextureManager {
            cache: HashMap::new(),
            storage: HashMap::new(),
            next_key: TextureId::new(),
            loader: loader,
        }
    }

    pub fn load(&mut self, path: &str) -> Result<TextureId, String>
    {
        match self.cache.get(path).cloned() {
            Some(id) => Ok(id),
            None => {
                println!("Loading {}", path);
                let resource = Rc::new(self.loader.load_texture(path)?);
                let id = self.next_key;
                self.cache.insert(path.into(), id);
                self.storage.insert(id, resource.clone());
                self.next_key = id.next();
                Ok(id)
            }
        }
    }

    pub fn make_text_texture(&mut self, text: &str, font: &Font, id_to_replace: Option<TextureId>) -> Result<TextureId, String> {
        // render a surface, and convert it to a texture bound to the renderer
        let surface = font.render(text)
            .blended(Color::RGBA(255, 255, 255, 255))
            .map_err(|e| e.to_string())?;
        let tex = self.loader
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let id = id_to_replace.unwrap_or_else(|| {
            let id = self.next_key;
            self.next_key = id.next();
            id
        });
        self.storage.insert(id, Rc::new(tex));
        Ok(id)
    }

    pub fn get(&self, key: TextureId) -> Option<Rc<Texture<'l>>> {
        self.storage.get(&key).cloned()
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

    pub fn draw<T: RenderTarget>(&self, transform: &Transform, canvas: &mut Canvas<T>, texture_manager: &TextureManager<T::Context>) {
        let texture = texture_manager.get(self.texture_id).unwrap();
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
