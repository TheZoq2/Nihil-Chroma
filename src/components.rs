extern crate ecs;
extern crate nalgebra;

use ecs::{ServiceManager};
use nalgebra::Vector2;
use sprite::{Sprite};
use player::{PlayerComponent};
use game::RespawnComponent;

pub struct ObamaComponent;

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoundingCircle {
    pub radius: f32
}


impl Default for BoundingCircle {
    fn default() -> BoundingCircle {
        BoundingCircle {
            radius: 1.0
        }
    }
}

components! {
    struct MyComponents {
        #[hot] transform: Transform,
        #[hot] velocity: Vector2<f32>,
        #[hot] max_velocity: f32,
        #[hot] sprite: Sprite,
        #[hot] bounding_box: BoundingCircle,
        #[cold] player_component: PlayerComponent,
        #[cold] obama: ObamaComponent,
        #[cold] respawn_component: RespawnComponent,
    }
}

pub struct MyServices {
    // pub remove_entity: Vec<Entity>,
    pub too_few_obamas: bool,
}

impl ServiceManager for MyServices {
}

impl Default for MyServices {
    fn default() -> MyServices{
        MyServices {
            // remove_entity: Vec::new(),
            too_few_obamas: false,
        }
    }
}
