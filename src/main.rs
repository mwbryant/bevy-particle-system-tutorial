#![allow(clippy::redundant_field_names)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::{prelude::*, render::camera::ScalingMode, window::PresentMode};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};

pub const CLEAR: Color = Color::rgb(0.3, 0.3, 0.3);
pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

#[derive(Component, Clone, Copy)]
pub struct ParticleSize {
    start: f32,
    end: f32,
}

#[derive(Component, Clone, Copy)]
pub struct ParticleVelocity {
    start: Vec2,
    end: Vec2,
}

#[derive(Component)]
pub struct Particle {
    lifetime: Timer,
}

#[derive(Component)]
pub struct ParticleSpawner {
    rate: f32,
    timer: Timer,
    amount_per_burst: usize,
    position_variance: f32,
    particle_lifetime: f32,
    particle_size: Option<ParticleSize>,
    particle_velocity: Option<ParticleVelocity>,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "Bevy Template".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(WorldInspectorParams {
            enabled: false,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(spawn_camera)
        .add_system(toggle_inspector)
        .add_startup_system(spawn_particle_spawner)
        .add_system(update_particle_lifetime)
        .add_system(update_particle_size)
        .add_system(update_particle_position)
        .add_system(emit_particles)
        .run();
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

fn lerp_vec2(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a * (1.0 - t) + b * t
}

fn update_particle_lifetime(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Particle)>,
    time: Res<Time>,
) {
    for (ent, mut particle) in particles.iter_mut() {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.finished() {
            commands.entity(ent).despawn();
        }
    }
}
fn update_particle_size(mut particles: Query<(&Particle, &ParticleSize, &mut Sprite)>) {
    for (particle, size, mut sprite) in particles.iter_mut() {
        let size = lerp(size.start, size.end, particle.lifetime.percent());
        sprite.custom_size = Some(Vec2::splat(size));
    }
}

fn update_particle_position(
    mut particles: Query<(&Particle, &ParticleVelocity, &mut Transform)>,
    time: Res<Time>,
) {
    for (particle, velocity, mut transform) in particles.iter_mut() {
        let velocity = lerp_vec2(velocity.start, velocity.end, particle.lifetime.percent());
        transform.translation += (velocity * time.delta_seconds()).extend(0.0);
    }
}

fn emit_particles(
    mut commands: Commands,
    mut spawners: Query<(Entity, &mut ParticleSpawner)>,
    time: Res<Time>,
) {
    for (ent, mut spawner) in spawners.iter_mut() {
        spawner.timer.tick(time.delta());
        if spawner.timer.just_finished() {
            for _i in 0..spawner.amount_per_burst {
                let particle = commands
                    .spawn()
                    .insert(Particle {
                        lifetime: Timer::from_seconds(spawner.particle_lifetime, false),
                    })
                    .id();

                let mut sprite = SpriteBundle::default();
                sprite.transform.translation = Vec3::new(
                    spawner.position_variance * (2.0 * rand::random::<f32>() - 1.0),
                    spawner.position_variance * (2.0 * rand::random::<f32>() - 1.0),
                    0.0,
                );

                if let Some(size) = spawner.particle_size {
                    sprite.sprite.custom_size = Some(Vec2::splat(size.start));
                    commands.entity(particle).insert(size);
                }
                if let Some(velocity) = spawner.particle_velocity {
                    commands.entity(particle).insert(velocity);
                }
                commands.entity(particle).insert_bundle(sprite);
                commands.entity(ent).add_child(particle);
            }
        }
    }
}

fn spawn_particle_spawner(mut commands: Commands) {
    commands
        .spawn_bundle(TransformBundle::default())
        .insert(ParticleSpawner {
            //rate: 0.5,
            rate: 0.01,
            timer: Timer::from_seconds(0.05, true),
            amount_per_burst: 3,
            position_variance: 5.0,
            particle_lifetime: 2.5,
            particle_size: Some(ParticleSize {
                start: 20.0,
                end: 1.0,
            }),
            particle_velocity: Some(ParticleVelocity {
                start: Vec2::new(40.0, 200.0),
                end: Vec2::new(80.0, 100.0),
            }),
        });
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.right = HEIGHT / 2.0 * RESOLUTION;
    camera.orthographic_projection.left = -HEIGHT / 2.0 * RESOLUTION;

    camera.orthographic_projection.top = HEIGHT / 2.0;
    camera.orthographic_projection.bottom = -HEIGHT / 2.0;

    camera.orthographic_projection.scaling_mode = ScalingMode::None;

    commands.spawn_bundle(camera);
}

fn toggle_inspector(
    input: ResMut<Input<KeyCode>>,
    mut window_params: ResMut<WorldInspectorParams>,
) {
    if input.just_pressed(KeyCode::Grave) {
        window_params.enabled = !window_params.enabled
    }
}

#[allow(dead_code)]
fn slow_down() {
    std::thread::sleep(std::time::Duration::from_secs_f32(1.000));
}
