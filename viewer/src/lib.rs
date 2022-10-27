use bevy::prelude::*;
use bevy::{
    ecs::event::{EventReader, Events},
    input::mouse::MouseMotion,
    math::Vec2,
    render::camera::Camera,
    render::mesh::Mesh,
    render::render_resource::PrimitiveTopology,
    sprite::MaterialMesh2dBundle,
};
use tracing::trace;

mod wasm {

    use wasm_bindgen::prelude::*;

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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn().insert_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(LineGraph {
            points: vec![
                Vec3::ZERO,
                Vec3::new(100.0, 100.0, 0.0),
                Vec3::new(100.0, 0.0, 0.0),
                Vec3::new(0.0, 100.0, 0.0),
            ],
        })).into(),
        material: materials.add(Color::BLUE.into()),
        ..default()
    });

    commands.spawn_bundle(Camera2dBundle::default());
}

#[derive(Debug, Clone)]
pub struct LineGraph {
    pub points: Vec<Vec3>,
}

impl From<LineGraph> for Mesh {
    fn from(line: LineGraph) -> Self {
        let mut vertices = vec![];
        let mut normals = vec![];
        let mut uvs = vec![];
        for pos in line.points {
            vertices.push(pos.to_array());
            normals.push(Vec3::ZERO.to_array());
            uvs.push([0.0; 2]);
        }

        // This tells wgpu that the positions are a list of points
        // where a line will be drawn between each consecutive point
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        // Normals are currently required by bevy, but they aren't used by the [`LineMaterial`]
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

fn update_mouse_motion(
    mut event_reader: EventReader<MouseMotion>,
    _events: Res<Events<MouseMotion>>,
    mut cameras: Query<&mut Transform, With<Camera>>,
) {
    let delta = event_reader.iter().fold(Vec2::ZERO, |acc, e| acc + e.delta);
    if delta == Vec2::ZERO {
        return;
    }

    let mut camera = cameras
        .get_single_mut()
        .expect("could not find scene camera");
    info!("camera = {:?}", camera);

    camera.translation += Vec3::new(1.0, 0.0, 0.0);
}
