//! Draw a grid of triangles, of random colors

use std::f32::consts::PI;

use bevy::{
    prelude::{
        default, shape, App, Assets, Camera2dBundle, Color, Commands, Image, Mesh, Quat, Query,
        Res, ResMut, Transform,
    },
    render::render_resource::TextureFormat,
    sprite::{ColorMaterial, MaterialMesh2dBundle, SpriteBundle},
    window::{Window, WindowResolution},
    DefaultPlugins,
};
use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    // camera
    commands.spawn(Camera2dBundle::default());

    // "constants"
    let triangle_radius = 32.;
    let triangle_side = 2. * triangle_radius * (PI / 3.).sin();
    let triangle_short_radius = triangle_radius * (PI / 3.).cos();
    let triangle_height = triangle_radius + triangle_short_radius;

    let window_w = window.single().resolution.width();
    let window_h = window.single().resolution.height();

    let iw = (window_w / triangle_side / 2.) as i32 + 1;
    let jw = (window_h / triangle_height / 2.) as i32 + 1;

    // triangles strip
    let mut rng = ChaCha8Rng::from_entropy();
    let color_value_dist = Uniform::new(0., 1.);
    for i in -iw..iw {
        for j in -jw..jw {
            let mut x = i as f32 * triangle_side;
            if j % 2 == 0 {
                x += triangle_side / 2.;
            }
            let mut y = j as f32 * (triangle_radius + triangle_short_radius);

            let color = Color::rgb(
                color_value_dist.sample(&mut rng),
                color_value_dist.sample(&mut rng),
                color_value_dist.sample(&mut rng),
            );
            commands.spawn(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::RegularPolygon::new(triangle_radius, 3).into())
                    .into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_xyz(x, y, 0.),
                ..default()
            });

            x += 2. * triangle_short_radius * (PI / 6.).cos();
            y += 2. * triangle_short_radius * (PI / 6.).sin();
            let color = Color::rgb(
                color_value_dist.sample(&mut rng),
                color_value_dist.sample(&mut rng),
                color_value_dist.sample(&mut rng),
            );
            commands.spawn(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::RegularPolygon::new(triangle_radius, 3).into())
                    .into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_xyz(x, y, 0.).with_rotation(Quat::from_rotation_z(PI)),
                ..default()
            });
        }
    }
}
