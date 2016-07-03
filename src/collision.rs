extern crate ecs;

use nalgebra::{Vector2};

use ecs::{Entity, System, EntityIter, DataHelper};
use ecs::system::{EntityProcess};
use components::{MyServices, MyComponents, Transform, BoundingCircle, BallType};

pub struct CollisionSystem {
    pub player: Entity,
}

impl System for CollisionSystem {
    type Components = MyComponents;
    type Services = MyServices;
}

fn are_colliding(tr1: &Transform, bb1: &BoundingCircle,
                 tr2: &Transform, bb2: &BoundingCircle) -> bool
{
    let xdist = (tr1.pos.x - tr2.pos.x).abs();
    let ydist = (tr1.pos.y - tr2.pos.y).abs();

    let r = bb1.radius + bb2.radius;

    xdist * xdist + ydist * ydist < r * r
}

impl EntityProcess for CollisionSystem {
    fn process(&mut self, entities: EntityIter<MyComponents>,
               data: &mut DataHelper<MyComponents, MyServices>)
    {
        data.services.new_points = 0;

        let mut player_transform: Transform = Default::default();
        let mut player_box: BoundingCircle = Default::default();
        data.with_entity_data(&self.player, |entity, data| {
            player_transform = data.transform[entity];
            player_box = data.bounding_box[entity];
        });

        for e in entities {
            let transform = data.transform[e];
            let bounding_box = data.bounding_box[e];

            if are_colliding(&player_transform, &player_box, &transform, &bounding_box) {
                //Respawn the ball
                data.transform[e].pos = Vector2::new(5000., 5000.);

                match data.ball_type[e]
                {
                    BallType::Good => data.services.new_points += 1,
                    BallType::Neutral => {},
                    BallType::Bad => data.services.hit_bad = true,
                }
            }
        }
    }
}
