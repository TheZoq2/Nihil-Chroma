extern crate specs;

use nalgebra::{Vector2};

use components::{Transform, BoundingCircle, BallType};
use components::{HitBad, HitNeutral, HitGood};

use specs::Join;

pub struct CollisionSystem {
    pub player: specs::Entity,
    pub new_points: i32,
}

fn are_colliding(tr1: &Transform, bb1: &BoundingCircle,
                 tr2: &Transform, bb2: &BoundingCircle) -> bool
{
    let xdist = (tr1.pos.x - tr2.pos.x).abs();
    let ydist = (tr1.pos.y - tr2.pos.y).abs();

    let r = bb1.radius + bb2.radius;

    xdist * xdist + ydist * ydist < r * r
}

impl<'a> specs::System<'a> for CollisionSystem {
    type SystemData = (
        specs::WriteStorage<'a, Transform>,
        specs::ReadStorage<'a, BoundingCircle>,
        specs::ReadStorage<'a, BallType>,
        specs::FetchMut<'a, HitBad>,
        specs::FetchMut<'a, HitNeutral>,
        specs::FetchMut<'a, HitGood>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (mut transforms, bounding_circles, ball_types, mut hit_bad, mut hit_neutral, mut hit_good) = data;
        hit_bad.0 = false;
        hit_neutral.0 = false;
        hit_good.0 = false;

        self.new_points = 0;

        let player_transform = transforms.get(self.player).unwrap().clone();
        let player_circle = bounding_circles.get(self.player).unwrap();

        for (mut transform, bounding_circle, ball_type) in (&mut transforms, &bounding_circles, &ball_types).join() {
            if are_colliding(&player_transform, &player_circle, &transform, &bounding_circle) {
                //Respawn the ball
                transform.pos = Vector2::new(5000., 5000.);

                match *ball_type {
                    BallType::Good => { self.new_points += 1; hit_good.0 = true },
                    BallType::Neutral => hit_neutral.0 = true,
                    BallType::Bad => hit_bad.0 = true,
                }
            }
        }
    }
}
