use tracing::{debug, trace};
use bevy::prelude::*;
use bevy::{
    ecs::event::{Events, EventReader},
    input::mouse::MouseMotion,
    math::Vec2,
    render::camera::Camera,
};

mod wasm {
    use wasm_bindgen::prelude::*;
    use console_error_panic_hook;

    #[wasm_bindgen(start)]
    pub fn run() {
        console_error_panic_hook::set_once();

        super::main();
    }
}

pub fn main() {
    trace!("nchoputa viewer starting up...");

    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "ncho".to_string(),
            fit_canvas_to_parent: true,
            ..Default::default()
        })
        .insert_resource(DrumBeat(Timer::from_seconds(1.0, true)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(update_mouse_motion)
        .add_system(clock)
        .run();

    trace!("start up done");
}

#[derive(Component, Deref, DerefMut)]
struct DrumBeat(Timer);

fn clock(time: Res<Time>, mut timer: ResMut<DrumBeat>, _query: Query<&mut DrumBeat>) {
    if timer.0.tick(time.delta()).just_finished() {
        info!("tick = {:?}", time.delta());
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
        ..Default::default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 5.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });
}

fn update_mouse_motion(
    mut event_reader: EventReader<MouseMotion>,
    _events: Res<Events<MouseMotion>>,
    mut cameras: Query<&mut Transform, With<Camera>>,
) {
    let delta = event_reader
        .iter()
        .fold(Vec2::ZERO, |acc, e| acc + e.delta);
    if delta == Vec2::ZERO {
        return
    }

    let mut camera = cameras.get_single_mut().expect("could not find scene camera");
    info!("camera = {:?}", camera);

    camera.translation += Vec3::new(0.1, 0.0, 0.0);
}
