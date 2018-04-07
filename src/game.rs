extern crate specs;
extern crate nalgebra as na;
extern crate rand;

use std::f64::consts;
use rand::Rng;

use nalgebra::{Vector2, Norm};

use rand::distributions::{IndependentSample, Range};

use specs::{Join, VecStorage};

use sprite::Sprite;
use constants::*;

use components::{Transform, Velocity, MaxVelocity, ObamaComponent, OrbitComponent};

#[derive(Component, Copy, Clone, Debug, PartialEq)]
#[component(VecStorage)]
pub struct RespawnComponent
{
    pub max_radius: f32,
    pub max_speed: f32,
    pub min_speed: f32,
}

pub struct MotionSystem {
    pub frametime: f32,
}

impl<'a> specs::System<'a> for MotionSystem {
    type SystemData = (specs::WriteStorage<'a, Transform>, specs::ReadStorage<'a, Velocity>);
    fn run(&mut self, (mut transforms, velocities): Self::SystemData) {
        for (transform, &Velocity(vel)) in (&mut transforms, &velocities).join() {
            transform.pos += vel * self.frametime;
        }
    }
}

pub struct ObamaSystem {
    pub too_few_obamas: bool
}

fn way_off_screen(pos: Vector2<f32>) -> bool {
    (pos.x as i32) < -100 || pos.x as u32 > RESOLUTION.0 + 100 ||
        (pos.y as i32) < -100 || pos.y as u32 > RESOLUTION.1 + 100
}

impl<'a> specs::System<'a> for ObamaSystem {
    type SystemData = (
        specs::Entities<'a>,
        specs::ReadStorage<'a, Transform>,
    );
    fn run(&mut self, (entities, transforms): Self::SystemData) {
        let mut obama_amount = 0;

        for (entity, transform) in (&*entities, &transforms).join() {
            obama_amount += 1;

            // Remove obamas that are too far out
            if way_off_screen(transform.pos) {
                entities.delete(entity).unwrap();
            }
        }

        self.too_few_obamas = obama_amount < 4;
    }
}

pub struct MaxVelSystem;

impl<'a> specs::System<'a> for MaxVelSystem
{
    type SystemData = (specs::WriteStorage<'a, Velocity>, specs::ReadStorage<'a, MaxVelocity>);
    fn run(&mut self, (mut velocities, max_velocities): Self::SystemData) {
        for (velocity, &MaxVelocity(max_vel)) in (&mut velocities, &max_velocities).join() {
            if velocity.0.norm_squared() > max_vel * max_vel {
                velocity.0 = velocity.0.normalize() * max_vel;
            }
        }
    }
}

pub struct RespawnSystem;

impl<'a> specs::System<'a> for RespawnSystem
{
    type SystemData = (
        specs::WriteStorage<'a, Transform>,
        specs::WriteStorage<'a, Velocity>,
        specs::ReadStorage<'a, RespawnComponent>,
    );
    fn run(&mut self, (mut transforms, mut velocities, respawns): Self::SystemData) {
        let center = Vector2::new((RESOLUTION.0 / 2) as f32, (RESOLUTION.1 / 2) as f32);
        let mut rng = rand::thread_rng();

        for (transform, velocity, respawn) in (&mut transforms, &mut velocities, &respawns).join() {
            let diff = transform.pos - center;

            if diff.x.powi(2) + diff.y.powi(2) > respawn.max_radius.powi(2) {
                //select a random position
                let angle = rng.gen_range(0., consts::PI*2.) as f32;

                transform.pos = Vector2::new((respawn.max_radius) * angle.cos(),
                                         (respawn.max_radius) * angle.sin()) +
                    center;

                let min_speed = respawn.min_speed;
                let max_speed = respawn.max_speed;

                //select a random direction
                let angle = rng.gen_range(0., consts::PI*2.) as f32;
                let speed = min_speed + rng.gen_range(0., max_speed - min_speed);
                velocity.0 = Vector2::new(speed * angle.cos(), speed * angle.sin());
            }
        }
    }
}

pub struct OrbitSystem
{
    pub player: specs::Entity,
    pub nuke_angle: f32,
}

impl<'a> specs::System<'a> for OrbitSystem
{
    type SystemData = (specs::WriteStorage<'a, Transform>, specs::WriteStorage<'a, OrbitComponent>);
    fn run(&mut self, (mut transforms, mut orbiters): Self::SystemData)
    {
        let player_pos = transforms.get(self.player).unwrap().pos;
        for (mut orbit, mut transform) in (&mut orbiters, &mut transforms).join() {
            orbit.angle += orbit.angular_velocity;

            let target_orbit = orbit.target_radius;
            let radius = orbit.radius;

            let diff = radius - target_orbit;

            orbit.radius -= diff * 0.003;

            let pos = player_pos + Vector2::new(
                radius * orbit.angle.cos(),
                radius * orbit.angle.sin(),
            );

            transform.pos = pos;
            transform.angle = orbit.angle as f64 - consts::PI * 0.5; 

            self.nuke_angle = orbit.angle;
        }
    }
}

fn random_edge_position() -> Vector2<f32>
{
    // Distance to corners going clockwise
    let c1 = RESOLUTION.0; // top left
    let c2 = c1 + RESOLUTION.1; // top right
    let c3 = c2 + RESOLUTION.0; // bottom right

    let mut rng = rand::thread_rng();
    let between_corners = Range::new(0, RESOLUTION.0*2 + RESOLUTION.1*2);
    let rand_dist = between_corners.ind_sample(&mut rng);
    let edge_offset = 50.0;

    if rand_dist < c1 {
        Vector2::new(rand_dist as f32, -edge_offset)
    } else if rand_dist < c2 {
        Vector2::new(RESOLUTION.0 as f32 + edge_offset, (rand_dist - c1) as f32)
    } else if rand_dist < c3 {
        Vector2::new((rand_dist - c2) as f32, RESOLUTION.1 as f32 + edge_offset)
    } else {
        Vector2::new(-edge_offset, (rand_dist - c3) as f32)
    }
}

pub fn create_obama(world: &mut specs::World, obama_sprites: &Vec<Sprite>)
{
    let between_angle = Range::new(0.0f32, (2.0*consts::PI) as f32);
    let mut rng = rand::thread_rng();

    let obama_pos = random_edge_position();
    let obama_speed = 20.0;

    let random_angle = between_angle.ind_sample(&mut rng);
    let random_velocity = Vector2::new(random_angle.cos()*obama_speed,
                                       random_angle.sin()*obama_speed);

    let obama_sprite = rng.choose(obama_sprites).unwrap();
    let obama_transform = Transform {
        pos: obama_pos,
        angle: 0.0,
        scale: Vector2::new(0.5, 0.5)
    };

    world.create_entity()
        .with(obama_sprite.clone())
        .with(obama_transform)
        .with(Velocity(random_velocity))
        .with(ObamaComponent).build();
}
