use bevy::prelude::*;
use bevy::{
    app::Events,
    window::WindowResized,
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
    let window = web_sys::window().unwrap();

    let mut app = App::build();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "ncho".to_string(),
            width: window.inner_width().unwrap().as_f64().unwrap() as f32,
            height: window.inner_height().unwrap().as_f64().unwrap() as f32,
            vsync: true,
            resizable: false,
            decorations: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_webgl2::WebGL2Plugin)
        .add_startup_system(setup.system())
        .add_startup_system(resize_notificator.system())
        .run();
}

// XXX: bevy doesn't yet support window resizing
fn resize_notificator(resize_event: Res<Events<WindowResized>>) {
    let mut reader = resize_event.get_reader();
    for e in reader.iter(&resize_event) {
        println!("width = {} height = {}", e.width, e.height);
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
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 5.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });
}
