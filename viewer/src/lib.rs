use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::{
    ecs::event::EventReader, render::mesh::Mesh, render::render_resource::PrimitiveTopology,
    sprite::MaterialMesh2dBundle, window::WindowResized,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use chrono::NaiveDate;
use postcard::from_bytes;
use serde::{Deserialize, Serialize};
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
        .insert_resource(State::new())
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(on_resize)
        .add_system(ui)
        .add_system(clock)
        .run();

    trace!("start up done");
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct GraphList {
    graphs: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Graph {
    // x: Scale,
    // y: Scale,
    points: Vec<(NaiveDate, f32)>,
}

struct State {
    startup: bool,
    loaded_legend: Arc<AtomicBool>,
    graph_list: Arc<Mutex<Option<GraphList>>>,
    fetching_graphs: Arc<Mutex<HashMap<String, String>>>,
    graphs: Arc<Mutex<HashMap<String, String>>>,
    loaded_graphs: Arc<Mutex<HashMap<String, Graph>>>,
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
        }
    }
}

// thoughts:
// egui is immediate, bevy is not, this is a slight impedance mismatch
fn ui(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, mut egui_context: ResMut<EguiContext>, mut state: ResMut<State>) {
    if state.startup {
        state.startup = false;

        let graph_list = state.graph_list.clone();
        let legend_bool = state.loaded_legend.clone();

        let request = ehttp::Request::get("/api/graphs");
        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            match result {
                Ok(v) if v.status == 200 => {
                    let list: GraphList = from_bytes(&v.bytes).unwrap();
                    info!("got response {:?}", list);
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
                    let graphs = graphs.lock().unwrap();
                    let graph = graphs.get(label);
                    let mut present = graph.is_some();

                    let mut fetching = fetching_graphs.lock().unwrap();
                    let enabled = !present && fetching.get(label).is_none();
                    ui.add_enabled_ui(enabled, |ui| {
                        if ui.checkbox(&mut present, label).clicked() {
                            fetching.insert(label.clone(), url.clone());

                            let label = label.clone();
                            let request = ehttp::Request::get(url);

                            let loaded_graphs = state.loaded_graphs.clone();
                            let fetchin_graphs = state.fetching_graphs.clone();
                            ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
                                match result {
                                    Ok(v) if v.status == 200 => {
                                        let graph: Graph = from_bytes(&v.bytes).unwrap();
                                        info!("got graph {:?}", graph);
                                        fetchin_graphs.lock().unwrap().remove(&label);
                                        loaded_graphs.lock().unwrap().insert(label, graph);
                                    },
                                    _ => {}
                                }
                            });
                        }
                    });
                }
            });
    }

    let mut loaded_graphs = state.loaded_graphs.lock().unwrap();
    for (name, graph) in loaded_graphs.iter() {
        let mut graph_points = Vec::new();
        for (i, (x, y)) in graph.points.iter().enumerate() {
            graph_points.push(Vec3::new(i as f32, *y, 0.0));
        }

        commands.spawn().insert_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(LineGraph {points: graph_points}))
                .into(),
            material: materials.add(Color::YELLOW.into()),
            ..default()
        });
    }
    loaded_graphs.clear();
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
    let axes = Axes::new();
    let mesh_bundle = MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(&axes)).into(),
        material: materials.add(Color::BLACK.into()),
        ..default()
    };
    commands
        .spawn()
        .insert(axes)
        .insert(mesh_bundle.mesh.0.clone())
        .insert_bundle(mesh_bundle);

    commands.spawn_bundle(Camera2dBundle::default());
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Scale {
    pub label: String,
    pub min: f32,
    pub max: f32,
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

fn on_resize(
    mut resize_reader: EventReader<WindowResized>,
    mut axes: Query<(&mut Axes, &Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (mut axes, handle) = axes.get_single_mut().unwrap();
    for e in resize_reader.iter() {
        axes.view_size.width = e.width;
        axes.view_size.height = e.height;
        info!("resize {:.1} x {:.1}", e.width, e.height);
        meshes.set(handle, Mesh::from(&*axes));
    }
}
