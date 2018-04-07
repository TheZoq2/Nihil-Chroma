extern crate specs;
extern crate nalgebra;

use specs::{VecStorage, NullStorage};
use nalgebra::Vector2;

// struct MyComponents {
//     #[hot] transform: Transform,
//     #[hot] velocity: Vector2<f32>,
//     #[hot] sprite: Sprite<'static>,
//     #[hot] bounding_box: BoundingCircle,
//     #[cold] player_component: PlayerComponent,
//     #[cold] obama: ObamaComponent,
//     #[cold] respawn_component: RespawnComponent,
//     #[cold] ball_type: BallType,
//     #[cold] stretch: StretchComponent,
//     #[cold] max_velocity: f32,
//     #[cold] orbit: OrbitComponent,
// }

#[derive(Component, Debug, Clone)]
#[component(VecStorage)]
pub enum BallType
{
    Good,
    Neutral,
    Bad,
}

#[derive(Component, Default)]
#[component(NullStorage)]
pub struct ObamaComponent;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
#[component(VecStorage)]
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

#[derive(Component)]
#[component(VecStorage)]
pub struct Velocity(pub Vector2<f32>);

#[derive(Component)]
#[component(VecStorage)]
pub struct MaxVelocity(pub f32);

#[derive(Component, Copy, Clone, Debug, PartialEq)]
#[component(VecStorage)]
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

pub struct StretchComponent
{
    pub amount: f32,
    pub original: Vector2<f32>,
}

#[derive(Component)]
#[component(VecStorage)]
pub struct OrbitComponent
{
    pub radius: f32,
    pub target_radius: f32,
    pub angular_velocity: f32,
    pub angle: f32,
}

pub struct HitBad(pub bool);
pub struct HitNeutral(pub bool);
pub struct HitGood(pub bool);
pub struct ScreenShake(pub Option<f32>);
