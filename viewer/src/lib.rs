use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, ecs::event::EventReader,
    input::mouse::MouseButton, input::mouse::MouseMotion, input::mouse::MouseWheel,
    render::mesh::Mesh, render::render_resource::PrimitiveTopology, sprite::MaterialMesh2dBundle,
    window::{PrimaryWindow, WindowResized},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use chrono::NaiveDate;
use postcard::from_bytes;
use serde::{Deserialize, Serialize};
use shared::response::{Graph, GraphList, Points};
use tracing::trace;

mod wasm {

    use wasm_bindgen::prelude::*;

    #[allow(non_snake_case)]
    #[wasm_bindgen(start)]
    pub fn run() {
        console_error_panic_hook::set_once();

        super::main();
    }
}

pub fn main() {
    trace!("nchoputa viewer starting up...");

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "ncho".to_string(),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(State::new())
    .add_event::<EventGraphAdded>()
    .add_event::<EventGraphRemoved>()
    .add_plugin(EguiPlugin)
    .add_startup_system(setup)
    .add_system(on_resize)
    .add_system(graph_added_listener)
    .add_system(graph_removed_listener)
    .add_system(ui)
    .add_system(on_mousewheel)
    .add_system(on_mousemotion)
    .run();

    trace!("start up done");
}

#[derive(Component)]
struct GraphName(String);

#[derive(Component)]
struct GraphPoints(Vec<(f32, f32)>);

#[derive(Component)]
struct GraphLabels(Vec<(NaiveDate, f32)>);

#[derive(Resource)]
struct State {
    startup: bool,
    loaded_legend: Arc<AtomicBool>,
    graph_list: Arc<Mutex<Option<GraphList>>>,
    fetching_graphs: Arc<Mutex<HashMap<String, String>>>,
    graphs: Arc<Mutex<HashMap<String, String>>>,
    loaded_graphs: Arc<Mutex<HashMap<String, Points>>>,
    unloaded_graphs: Arc<Mutex<Vec<String>>>,
}

impl State {
    fn new() -> Self {
        State {
            startup: true,
            loaded_legend: default(),
            graph_list: default(),
            fetching_graphs: default(),
            graphs: default(),
            loaded_graphs: default(),
            unloaded_graphs: default(),
        }
    }
}

struct EventGraphAdded {
    graph_name: String,
    graph_points: Points,
}

struct EventGraphRemoved {
    graph_name: String,
}

// thoughts:
// egui is immediate, bevy is not, this is a slight impedance mismatch
fn ui(
    mut egui_context: EguiContexts,
    mut state: ResMut<State>,
    mut added_events: EventWriter<EventGraphAdded>,
    mut removed_events: EventWriter<EventGraphRemoved>,
) {
    if state.startup {
        state.startup = false;

        let graph_list = state.graph_list.clone();
        let legend_bool = state.loaded_legend.clone();

        let request = ehttp::Request::get("/api/graphs");
        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            match result {
                Ok(v) if v.status == 200 => {
                    let list: GraphList = from_bytes(&v.bytes).unwrap();
                    *graph_list.lock().unwrap() = Some(list);
                }
                _ => {}
            }
            legend_bool.store(true, Ordering::SeqCst);
        });
    }

    if state.loaded_legend.load(Ordering::SeqCst) {
        let graph_list = state.graph_list.clone();
        let fetching_graphs = state.fetching_graphs.clone();
        let graphs = state.graphs.clone();

        egui::Window::new("Datasets")
            .vscroll(true)
            .resizable(false)
            .anchor(egui::Align2::RIGHT_TOP, [-100.0, 100.0])
            .show(egui_context.ctx_mut(), |ui| {
                let graph_list = graph_list.lock().unwrap();
                for (label, url) in graph_list.as_ref().unwrap().graphs.iter() {
                    let mut graphs = graphs.lock().unwrap();
                    let graph = graphs.get(label);
                    let mut present = graph.is_some();

                    let mut fetching = fetching_graphs.lock().unwrap();
                    let enabled = fetching.get(label).is_none();
                    ui.add_enabled_ui(enabled, |ui| {
                        if ui.checkbox(&mut present, label).clicked() {
                            if present {
                                graphs.insert(label.clone(), url.clone());
                                fetching.insert(label.clone(), url.clone());

                                let request = ehttp::Request::get(url);

                                let label = label.clone();
                                let loaded_graphs = state.loaded_graphs.clone();
                                let fetchin_graphs = state.fetching_graphs.clone();
                                ehttp::fetch(
                                    request,
                                    move |result: ehttp::Result<ehttp::Response>| match result {
                                        Ok(v) if v.status == 200 => {
                                            let graph: Graph = from_bytes(&v.bytes).unwrap();
                                            fetchin_graphs.lock().unwrap().remove(&label);
                                            loaded_graphs
                                                .lock()
                                                .unwrap()
                                                .insert(label, graph.points);
                                        }
                                        _ => {}
                                    },
                                );
                            } else {
                                graphs.remove(label);
                                state.unloaded_graphs.lock().unwrap().push(label.clone());
                            }
                        }
                    });
                }
            });
    }

    let mut loaded_graphs = state.loaded_graphs.lock().unwrap();
    for (name, graph) in loaded_graphs.iter() {
        added_events.send(EventGraphAdded {
            graph_name: name.to_string(),
            graph_points: graph.clone(),
        });
    }
    loaded_graphs.clear();

    let mut unloaded_graphs = state.unloaded_graphs.lock().unwrap();
    for graph_name in unloaded_graphs.iter() {
        removed_events.send(EventGraphRemoved {
            graph_name: graph_name.clone(),
        });
    }
    unloaded_graphs.clear();
}

fn date_scale(date: &NaiveDate) -> f32 {
    (*date - NaiveDate::from_ymd_opt(0, 1, 1).unwrap()).num_days() as f32
}

fn graph_added_listener(
    mut events: EventReader<EventGraphAdded>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<SceneCamera>)>,
    mut axes: Query<(&mut Axes, &Handle<Mesh>)>,
) {
    for event in events.iter() {
        let points = &event.graph_points;

        let mut mesh_points = Vec::new();
        let mut graph_points = Vec::new();
        let mut graph_labels = Vec::new();
        for (date, y) in points.iter() {
            graph_labels.push((*date, *y));
            let x = date_scale(date);
            graph_points.push((x, *y));
            mesh_points.push(Vec3::new(x, *y, 0.0));
        }

        let mesh_bundle = MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(LineGraph {
                    points: mesh_points,
                }))
                .into(),
            material: materials.add(Color::YELLOW.into()),
            ..default()
        };
        commands
            .spawn_empty()
            .insert(GraphName(event.graph_name.to_string()))
            .insert(GraphPoints(graph_points))
            .insert(GraphLabels(graph_labels))
            .insert(mesh_bundle);

        // Recalculate the scales
        let (mut axes, handle) = axes.get_single_mut().unwrap();
        axes.x.min = points
            .iter()
            .map(|(a, _)| date_scale(a))
            .fold(f32::INFINITY, |a, b| a.min(b));
        axes.x.max = points
            .iter()
            .map(|(a, _)| date_scale(a))
            .fold(f32::NEG_INFINITY, |a, b| a.max(b));

        axes.y.min = points
            .iter()
            .map(|(_, a)| a)
            .fold(f32::INFINITY, |a, b| a.min(*b));
        axes.y.max = points
            .iter()
            .map(|(_, a)| a)
            .fold(f32::NEG_INFINITY, |a, b| a.max(*b));

        let mut camera = cameras
            .get_single_mut()
            .expect("could not find scene camera");

        // Reposition the camera to center over the graph
        let camera_x = axes.x.min + (axes.x.max - axes.x.min) / 2.0;
        let camera_y = axes.y.min + (axes.y.max - axes.y.min) / 2.0;
        camera.translation = Vec3::new(camera_x, camera_y, 0.0);

        // Scale to fit the whole graph in
        camera.scale.x = (axes.x.max - axes.x.min) / axes.view_size.width;
        camera.scale.y = (axes.y.max - axes.y.min) / axes.view_size.height;

        let mut mesh = meshes.get_mut(handle).unwrap();
        axes.update(&mut mesh);
    }
}

fn graph_removed_listener(
    mut events: EventReader<EventGraphRemoved>,
    mut commands: Commands,
    graphs: Query<(Entity, &GraphName)>,
) {
    // TODO: this feels inefficient, somehow store the Entity instead?
    for event in events.iter() {
        for graph in graphs.iter() {
            if graph.1 .0 == event.graph_name {
                commands.entity(graph.0).despawn_recursive();
            }
        }
    }
}

#[derive(Component)]
struct SceneCamera;

#[derive(Component)]
struct OverlayCamera;

#[derive(Component)]
struct Cursor;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(SceneCamera);

    let axes = Axes::new();
    let mesh_bundle = MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(&axes)).into(),
        material: materials.add(Color::BLACK.into()),
        ..default()
    };

    // Overlay camera, where axes etc. gets rendered
    commands
        .spawn(Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        })
        .insert(OverlayCamera);

    commands
        .spawn_empty()
        .insert(axes)
        .insert(mesh_bundle.mesh.0.clone())
        .insert(mesh_bundle);

    let font = asset_server.load("/s/FiraMono-Medium.ttf");
    let text_style = TextStyle {
        font,
        font_size: 16.0,
        color: Color::WHITE,
    };
    commands
        .spawn_empty()
        .insert(Cursor {})
        .insert(MaterialMesh2dBundle {
            mesh: meshes.add(new_crosshair_mesh()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            ..default()
        })
        .insert(Text2dBundle{
            text: Text::from_section("x, y", text_style),
            ..default()
        })
        .insert(Visibility::Hidden);
}

fn new_crosshair_mesh() -> Mesh {

    let mut vertices = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];

    vertices.push([0.0, -1.0, 0.0]);
    vertices.push([0.0, 1.0, 0.0]);

    vertices.push([-1.0, 0.0, 0.0]);
    vertices.push([1.0, 0.0, 0.0]);

    normals.push(Vec3::ZERO.to_array());
    uvs.push([0.0; 2]);
    normals.push(Vec3::ZERO.to_array());
    uvs.push([0.0; 2]);
    normals.push(Vec3::ZERO.to_array());
    uvs.push([0.0; 2]);
    normals.push(Vec3::ZERO.to_array());
    uvs.push([0.0; 2]);

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Scale {
    pub label: String,
    pub min: f32,
    pub max: f32,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Component, Clone)]
pub struct Axes {
    x: Scale,
    y: Scale,
    view_size: Size,
}

impl Axes {
    fn new() -> Self {
        Axes {
            x: Scale {
                label: "x".to_string(),
                min: 0.0,
                max: 100.0,
            },
            y: Scale {
                label: "y".to_string(),
                min: 0.0,
                max: 100.0,
            },
            view_size: Size::default(),
        }
    }
}

impl From<&Axes> for Mesh {
    fn from(this: &Axes) -> Self {
        let mut vertices = vec![];
        let mut normals = vec![];
        let mut uvs = vec![];

        let padding = this.view_size.width * 0.08;
        let min_x = (this.view_size.width - padding) / 2.0;
        let min_y = (this.view_size.height - padding) / 2.0;

        vertices.push([-min_x, min_y, 0.0]);
        vertices.push([-min_x, -min_y, 0.0]);
        vertices.push([min_x, -min_y, 0.0]);
        normals.push(Vec3::ZERO.to_array());
        uvs.push([0.0; 2]);
        normals.push(Vec3::ZERO.to_array());
        uvs.push([0.0; 2]);
        normals.push(Vec3::ZERO.to_array());
        uvs.push([0.0; 2]);

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

impl Axes {
    fn update(&self, mesh: &mut Mesh) {
        let mut vertices = vec![];
        let mut normals = vec![];
        let mut uvs = vec![];

        let padding = self.view_size.width * 0.08;
        let min_x = (self.view_size.width - padding) / 2.0;
        let min_y = (self.view_size.height - padding) / 2.0;

        vertices.push([-min_x, min_y, 0.0]);
        vertices.push([-min_x, -min_y, 0.0]);
        vertices.push([min_x, -min_y, 0.0]);
        normals.push(Vec3::ZERO.to_array());
        uvs.push([0.0; 2]);
        normals.push(Vec3::ZERO.to_array());
        uvs.push([0.0; 2]);
        normals.push(Vec3::ZERO.to_array());
        uvs.push([0.0; 2]);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
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

fn on_mousewheel(
    mut event_reader: EventReader<MouseWheel>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<SceneCamera>)>,
) {
    for e in event_reader.iter() {
        let mut camera = cameras
            .get_single_mut()
            .expect("could not find scene camera");

        let factor = if e.y >= 0.0 {
            e.y / 10.0
        } else {
            1.0 / (f32::abs(e.y) / 10.0)
        };
        camera.scale *= Vec3::new(factor, factor, 1.0);
    }
}

fn on_mousemotion(
    mouse_button_input: Res<Input<MouseButton>>,
    mut event_reader: EventReader<MouseMotion>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<SceneCamera>)>,
    graphs: Query<(&GraphPoints, &GraphLabels, &GraphName)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cursor: Query<(&Cursor, &mut Transform, &mut Text, &mut Visibility), Without<SceneCamera>>,
    axes: Query<&Axes>,
) {
    for e in event_reader.iter() {
        let mut camera = cameras
            .get_single_mut()
            .expect("could not find scene camera");

        if mouse_button_input.pressed(MouseButton::Middle) {
            let x = -e.delta.x * camera.scale.x;
            let y = e.delta.y * camera.scale.y;
            camera.translation += Vec3::new(x, y, 0.0);
        }

        // Is the mouse near a point on a graph?
        if let Some(position) = windows
            .get_single()
            .expect("could not get the primary window")
            .cursor_position()
        {
            let axes = axes.get_single().expect("no axes");

            // Convert the mouse position to world position
            let x = (position.x - axes.view_size.width / 2.0) * camera.scale.x + camera.translation.x;
            let y = (position.y - axes.view_size.height / 2.0) * camera.scale.y + camera.translation.y;

            let (_, mut cursor, mut text, mut visibility) = cursor.get_single_mut().expect("could not get cursor");
            *visibility = Visibility::Hidden;

            for (points, labels, name) in graphs.iter() {
                // TODO: size is asymmetrical
                let size_x = 10.0 * camera.scale.x;
                let size_y = 10.0 * camera.scale.y;
                for (index, (px, py)) in points.0.iter().enumerate() {
                    if x > px - size_x && y > py - size_y && x < px + size_x && y < py + size_y {
                        cursor.scale.x = camera.scale.x;
                        cursor.scale.y = camera.scale.y;
                        cursor.translation.x = *px;
                        cursor.translation.y = *py;

                        // FIXME: hack right now to pad text away from the cursor, perhaps need a
                        // parent child relationship here so we can position text relative to
                        // cursor?
                        let label = labels.0.get(index).unwrap();
                        text.sections[0].value = format!("   {} = {}, {}", name.0, label.0, label.1);
                        info!("mouse near point {} {}", px, py);
                        *visibility = Visibility::Visible;
                        
                        // FIXME: This means we'll show the cursor near the first point, not the closest!
                        break;
                    }
                }
            }
        }
    }
}

fn on_resize(
    mut resize_reader: EventReader<WindowResized>,
    mut axes: Query<(&mut Axes, &Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (mut axes, handle) = axes.get_single_mut().unwrap();
    for e in resize_reader.iter() {
        axes.view_size.width = e.width;
        axes.view_size.height = e.height;

        let mut mesh = meshes.get_mut(handle).unwrap();
        axes.update(&mut mesh);
    }
}
