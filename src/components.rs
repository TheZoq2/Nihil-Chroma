extern crate ecs;
extern crate nalgebra;

use ecs::{Entity, ServiceManager};
use nalgebra::Vector2;
use sprite::{Sprite};
use player::{PlayerComponent};
use game::RespawnComponent;

#[derive(Clone)]
pub enum BallType
{
    Good,
    Neutral,
    Bad,
}

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
        #[cold] ball_type: BallType,
    }
}

pub struct MyServices {
    pub swap_sprite_with_text: Vec<(Entity, String)>,
    pub too_few_obamas: bool,
    pub new_points: u32,
    pub hit_bad: bool,
}

impl ServiceManager for MyServices {
}

impl Default for MyServices {
    fn default() -> MyServices{
        MyServices {
            swap_sprite_with_text: Vec::new(),
            too_few_obamas: false,
            new_points: 0,
            hit_bad: false,
        }
    }
}
