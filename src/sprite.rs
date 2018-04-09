extern crate nalgebra as na;
extern crate sdl2;
extern crate specs;

use components::Transform;

use sdl2::rect::{Rect};
use sdl2::render::{Canvas, RenderTarget, Texture, TextureCreator};
use sdl2::image::LoadTexture;
use specs::VecStorage;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct TextureId(isize);

pub trait Key {
    fn new() -> Self;
    fn next(&self) -> Self;
}

impl Key for TextureId {
    fn new() -> TextureId {
        TextureId(0)
    }

    fn next(&self) -> TextureId {
        TextureId(self.0 + 1)
    }
}

// Resource manager code taken from https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/resource-manager.rs
// Generic struct to cache any resource loaded by a ResourceLoader
pub struct ResourceManager<'l, K, Q, R, L>
    where K: Hash + Eq,
          Q: Key + Eq + Copy + Key,
          L: 'l + ResourceLoader<'l, R>
{
    loader: &'l L,
    cache: HashMap<K, Q>,
    next_key: Q,
    storage: HashMap<Q, Rc<R>>,
}

pub type TextureManager<'l, T> = ResourceManager<'l, String, TextureId, Texture<'l>, TextureCreator<T>>;

impl<'l, K, Q, R, L> ResourceManager<'l, K, Q, R, L>
    where K: Hash + Eq,
          Q: Key + Hash + Eq + Copy + Key,
          L: ResourceLoader<'l, R>
{
    pub fn new(loader: &'l L) -> Self {
        ResourceManager {
            cache: HashMap::new(),
            storage: HashMap::new(),
            next_key: Q::new(),
            loader: loader,
        }
    }

    // Generics magic to allow a HashMap to use String as a key
    // while allowing it to use &str for gets
    pub fn load<D>(&mut self, details: &D) -> Result<Q, String>
        where L: ResourceLoader<'l, R, Args = D>,
              D: Eq + Hash + ?Sized,
              K: Borrow<D> + for<'a> From<&'a D>
    {
        self.cache
            .get(details)
            .cloned()
            .map_or_else(|| {
                             let resource = Rc::new(self.loader.load(details)?);
                             let id = self.next_key;
                             self.cache.insert(details.into(), id);
                             self.storage.insert(id, resource.clone());
                             self.next_key = id.next();
                             Ok(id)
                         },
                         Ok)
    }

    pub fn get(&self, key: Q) -> Option<Rc<R>> {
        self.storage.get(&key).cloned()
    }
}

// Generic trait to Load any Resource Kind
pub trait ResourceLoader<'l, R> {
    type Args: ?Sized;
    fn load(&'l self, data: &Self::Args) -> Result<R, String>;
}

// TextureCreator knows how to load Textures
impl<'l, T> ResourceLoader<'l, Texture<'l>> for TextureCreator<T> {
    type Args = str;
    fn load(&'l self, path: &str) -> Result<Texture, String> {
        println!("Loading {}", path);
        self.load_texture(path)
    }
}

// pub struct TextureRegistry<'a> {
//     next_id: TextureId,
//     textures: HashMap<TextureId, Texture<'a>>,
// }

// impl<'a> TextureRegistry<'a> {
//     pub fn new() -> TextureRegistry<'a> {
//         TextureRegistry {
//             next_id: TextureId(0),
//             textures: HashMap::new(),
//         }
//     }

//     pub fn add(&mut self, texture: Texture<'a>) -> TextureId {
//         let id = self.next_id;
//         self.next_id = id.next();
//         self.textures.insert(id, texture);
//         id
//     }

//     pub fn replace(&mut self, id: TextureId, new_texture: Texture<'a>) {
//         self.textures.insert(id, new_texture);
//     }

//     pub fn remove(&mut self, id: TextureId) {
//         self.textures.remove(&id);
//     }

//     pub fn get(&self, id: TextureId) -> Option<&Texture<'a>> {
//         self.textures.get(&id)
//     }
// }

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
