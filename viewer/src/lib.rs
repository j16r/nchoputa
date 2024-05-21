use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::{
    asset::AssetMetaCheck,
    ecs::event::EventReader,
    input::mouse::MouseButton,
    input::mouse::MouseMotion,
    input::mouse::MouseWheel,
    render::mesh::Mesh,
    render::render_resource::PrimitiveTopology,
    render::render_asset::RenderAssetUsages,
    render::camera::ClearColorConfig,
    sprite::MaterialMesh2dBundle,
    window::{PrimaryWindow, WindowResized, PresentMode},
    render::view::visibility::RenderLayers,
    log::LogPlugin,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use chrono::NaiveDate;
use postcard::from_bytes;
use serde::{Deserialize, Serialize};
use shared::response::{Graph, GraphList};

mod wasm {

    use tracing_subscriber::{EnvFilter, fmt::format::Pretty, filter::LevelFilter, prelude::*};
    use tracing_web::{MakeWebConsoleWriter, performance_layer};
    use wasm_bindgen::prelude::*;

    #[allow(non_snake_case)]
    #[wasm_bindgen(start)]
    pub fn run() {
        console_error_panic_hook::set_once();

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_level(false)
            .with_ansi(false)
            .without_time()
            .with_writer(MakeWebConsoleWriter::new().with_pretty_level());
        let perf_layer = performance_layer()
            .with_details_from_fields(Pretty::default());

        tracing_subscriber::registry()
            .with(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::WARN.into())
                    .parse("viewer=trace")
                    .unwrap()
            )
            .with(fmt_layer)
            .with(perf_layer)
            .init();

        super::main();
    }
}

pub fn main() {
    tracing::info!("starting up...");

    let mut app = App::new();
    app
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ncho".to_string(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }).disable::<LogPlugin>())
        // XXX: Doesn't do any filtering?
        // }).set(LogPlugin{
        //     level: Level::ERROR,
        //     // filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,nchoputa=trace".to_string(),
        //     filter: "nchoputa=trace".to_string(),
        //     update_subscriber: None,
        // }))
        .insert_resource(State::new())
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                on_resize,
                graph_added_listener,
                graph_removed_listener,
                ui,
                on_mousewheel,
                on_mousemotion,
            ),
        )
        .add_event::<EventGraphAdded>()
        .add_event::<EventGraphRemoved>()
        .run();

    tracing::info!("start up complete");
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
    loaded_graphs: Arc<Mutex<HashMap<String, Graph>>>,
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

#[derive(Event)]
struct EventGraphAdded {
    graph_name: String,
    graph: Graph,
}

#[derive(Event)]
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
        tracing::trace!("performing ui startup");
        state.startup = false;

        let graph_list = state.graph_list.clone();
        let legend_bool = state.loaded_legend.clone();

        let request = ehttp::Request::get("/api/graphs");
        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            match result {
                Ok(v) if v.status == 200 => {
                    let list: GraphList = from_bytes(&v.bytes).unwrap();
                    tracing::info!("server responded with legend = {:?}", &list);
                    *graph_list.lock().unwrap() = Some(list);
                }
                _ => {
                    tracing::warn!("error loading legend");
                }
            }
            legend_bool.store(true, Ordering::SeqCst);
        });
    }

    if state.loaded_legend.load(Ordering::SeqCst) {
        let graph_list = state.graph_list.clone();
        let fetching_graphs = state.fetching_graphs.clone();
        let graphs = state.graphs.clone();

        egui::Window::new("Datasets")
            .enabled(true)
            .vscroll(true)
            .resizable(false)
            .movable(true)
            .auto_sized()
            .anchor(egui::Align2::RIGHT_TOP, [-100.0, 100.0])
            .show(egui_context.ctx_mut(), |ui| {
                let graph_list = graph_list.lock().unwrap();
                for (label, url) in graph_list.as_ref().unwrap().graphs.iter() {
                    let mut graphs = graphs.lock().unwrap();
                    let graph = graphs.get(label);
                    let mut present = graph.is_some();

                    let mut fetching = fetching_graphs.lock().unwrap();
                    let enabled = fetching.get(label).is_none();
                    // tracing::trace!("rendering item {}", label);
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
                                                .insert(label, graph);
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
            graph: graph.clone(),
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
    mut cameras: Query<&mut Transform, With<SceneCamera>>,
    mut axes: Query<(&mut Axes, &Handle<Mesh>)>,
) {
    for event in events.read() {
        let points = &event.graph.points;

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
            material: materials.add(Color::rgb_u8(event.graph.color.0, event.graph.color.1, event.graph.color.2)),
            ..default()
        };
        commands
            .spawn((
                mesh_bundle,
                RenderLayers::layer(0),
            ))
            .insert(GraphName(event.graph_name.to_string()))
            .insert(GraphPoints(graph_points))
            .insert(GraphLabels(graph_labels));

        // Recalculate the scales
        let (mut axes, handle) = axes.get_single_mut().unwrap();
        axes.x.min = points
            .iter()
            .map(|(a, _)| date_scale(a))
            .fold(f32::MAX, |a, b| a.min(b));
        axes.x.max = points
            .iter()
            .map(|(a, _)| date_scale(a))
            .fold(f32::MIN, |a, b| a.max(b));

        axes.y.min = points
            .iter()
            .map(|(_, a)| a)
            .fold(f32::MAX, |a, b| a.min(*b));
        axes.y.max = points
            .iter()
            .map(|(_, a)| a)
            .fold(f32::MIN, |a, b| a.max(*b));

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

        let mesh = meshes.get_mut(handle).unwrap();
        axes.update(mesh);
    }
}

fn graph_removed_listener(
    mut events: EventReader<EventGraphRemoved>,
    mut commands: Commands,
    graphs: Query<(Entity, &GraphName)>,
) {
    // TODO: this feels inefficient, somehow store the Entity instead?
    for event in events.read() {
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
struct Crosshair;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            Camera2dBundle::default(),
            RenderLayers::from_layers(&[0])
        ))
        .insert(SceneCamera);

    // Overlay camera, where axes etc. gets rendered
    commands
        .spawn((
            Camera2dBundle {
                camera_2d: Camera2d,
                camera: Camera {
                    clear_color: ClearColorConfig::None,
                    order: 1,
                    ..default()
                },
                ..default()
            },
            RenderLayers::from_layers(&[1])
        ))
        .insert(OverlayCamera);

    let axes = Axes::new();
    let mesh_bundle = MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(&axes)).into(),
        material: materials.add(Color::SILVER),
        ..default()
    };
    commands
        .spawn((
            mesh_bundle.clone(),
            RenderLayers::layer(1),
        ))
        .insert(axes)
        .insert(mesh_bundle.mesh.0.clone());

    let font = asset_server.load("/s/FiraMono-Medium.ttf");
    // Bevy does not support woff2 see https://github.com/bevyengine/bevy/issues/12194
    // let font = asset_server.load("/s/FiraMono-Medium.woff2");
    let text_style = TextStyle {
        font,
        font_size: 16.0,
        color: Color::WHITE,
    };
    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(new_crosshair_mesh()).into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                ..default()
            },
            RenderLayers::layer(1),
        ))
        .insert(Crosshair {})
        .insert(Text2dBundle {
            text: Text::from_section("x, y", text_style),
            transform: Transform{translation: Vec3{x: 70.0, y: 8.0, z: 1.0}, ..default()},
            ..default()
        })
        .insert(Visibility::Hidden);
}

fn new_crosshair_mesh() -> Mesh {
    let mut vertices = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];

    vertices.push([0.0, -10.0, 0.0]);
    vertices.push([0.0, 10.0, 0.0]);

    vertices.push([-10.0, 0.0, 0.0]);
    vertices.push([10.0, 0.0, 0.0]);

    normals.push(Vec3::ZERO.to_array());
    uvs.push([0.0; 2]);
    normals.push(Vec3::ZERO.to_array());
    uvs.push([0.0; 2]);
    normals.push(Vec3::ZERO.to_array());
    uvs.push([0.0; 2]);
    normals.push(Vec3::ZERO.to_array());
    uvs.push([0.0; 2]);

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
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
    max_ticks: usize,
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
            max_ticks: 10,
        }
    }

    fn nice_num(&self, lst: f32, rround: bool) -> f32 {
        let exponent = f32::floor(f32::log10(lst));
        let fraction = lst / f32::powf(10.0, exponent);

        let nice_fraction = if rround {
            if fraction < 1.5 {
                1.0
            } else if fraction < 3.0 {
                2.0
            } else if fraction < 7.0 {
                5.0
            } else {
                10.0
            }
        } else {
            if fraction <= 1.0 {
                1.0
            } else if fraction <= 2.0 {
                2.0
            } else if fraction <= 5.0 {
                5.0
            } else {
                10.0
            }
        };

        nice_fraction * f32::powf(10.0, exponent)
    }

    fn range(&self) -> f32 {
        self.nice_num(self.x.max - self.x.min, false)
    }

    fn tick_spacing(&self) -> f32 {
        self.nice_num(self.range() / (self.max_ticks - 1) as f32, true)
    }

    fn scale_x_max(&self) -> f32 {
        f32::ceil(self.x.max / self.tick_spacing()) * self.tick_spacing()
    }

    fn scale_x_min(&self) -> f32 {
        f32::ceil(self.x.min / self.tick_spacing()) * self.tick_spacing()
    }

    fn scale_y_max(&self) -> f32 {
        f32::ceil(self.y.max / self.tick_spacing()) * self.tick_spacing()
    }

    fn scale_y_min(&self) -> f32 {
        f32::ceil(self.y.min / self.tick_spacing()) * self.tick_spacing()
    }
}

#[test]
fn test_axes_scale() {
    let a = Axes::new();
    assert_eq!(a.scale_x_min(), 0.0);
    assert_eq!(a.scale_x_max(), 100.0);
    assert_eq!(a.tick_spacing(), 10.0);
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

        vertices.iter().for_each(|_| {
            normals.push(Vec3::ZERO.to_array());
            uvs.push([0.0; 2]);
        });

        // This tells wgpu that the positions are a list of points
        // where a line will be drawn between each consecutive point
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default());

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
        let width = self.view_size.width - padding;
        let height = self.view_size.height - padding;
        let min_x = width / 2.0;
        let min_y = height / 2.0;

        vertices.push([-min_x, min_y, 0.0]);

        let (min, max) = (self.scale_y_min(), self.scale_y_max());
        for point in std::iter::successors(Some(min), |i| {
            let next = i + self.tick_spacing();
            (next <= max).then_some(next)
        }) {
            let range = max - min;
            let offset = point - min;
            let ratio = offset / range;
            let y = (ratio * height) - min_y;

            vertices.push([-min_x, -y, 0.0]);
            vertices.push([-min_x - 15.0, -y, 0.0]);
            vertices.push([-min_x, -y, 0.0]);
        }

        vertices.push([-min_x, -min_y, 0.0]);

        let (min, max) = (self.scale_x_min(), self.scale_x_max());
        for point in std::iter::successors(Some(min), |i| {
            let next = i + self.tick_spacing();
            (next <= max).then_some(next)
        }) {
            let range = max - min;
            let offset = point - min;
            let ratio = offset / range;
            let x = (ratio * width) - min_x;

            vertices.push([-x, -min_y, 0.0]);
            vertices.push([-x, -min_y - 15.0, 0.0]);
            vertices.push([-x, -min_y, 0.0]);
        }

        vertices.push([min_x, -min_y, 0.0]);

        vertices.iter().for_each(|_| {
            normals.push(Vec3::ZERO.to_array());
            uvs.push([0.0; 2]);
        });
        // normals.push(Vec3::ZERO.to_array());
        // uvs.push([0.0; 2]);
        // normals.push(Vec3::ZERO.to_array());
        // uvs.push([0.0; 2]);

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
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default());

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        // Normals are currently required by bevy, but they aren't used by the [`LineMaterial`]
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

fn on_mousewheel(
    mut event_reader: EventReader<MouseWheel>,
    mut cameras: Query<&mut Transform, With<SceneCamera>>,
) {
    let span = 16.0;
    for e in event_reader.read() {
        let mut camera = cameras
            .get_single_mut()
            .expect("could not find scene camera");

        let factor = if e.y >= 0.0 {
            e.y / span
        } else {
            1.0 / (f32::abs(e.y) / span)
        };
        tracing::trace!("e.y = {}, factor = {}", e.y, factor);
        camera.scale *= Vec3::new(factor, factor, 1.0);
    }
}

fn on_mousemotion(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut event_reader: EventReader<MouseMotion>,
    mut cameras: Query<(&mut Camera, &mut Transform, &mut GlobalTransform), With<SceneCamera>>,
    graphs: Query<(&GraphPoints, &GraphLabels, &GraphName)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cursor: Query<(&Crosshair, &mut Transform, &mut Text, &mut Visibility), Without<SceneCamera>>,
) {
    let window = windows
        .get_single()
        .expect("could not get the primary window");

    for e in event_reader.read() {
        let (camera, mut camera_transform, camera_global_transform) = cameras.single_mut();

        if mouse_button_input.pressed(MouseButton::Middle) {
            let x = -e.delta.x * camera_transform.scale.x;
            let y = e.delta.y * camera_transform.scale.y;
            camera_transform.translation += Vec3::new(x, y, 0.0);
        }

        if let Some(scene_position) = window.cursor_position()
            .and_then(|c| camera.viewport_to_world_2d(&camera_global_transform, c)) {

            let (_, mut crosshair, mut text, mut visibility) =
                cursor.get_single_mut().expect("could not get crosshair");
            *visibility = Visibility::Hidden;

            // graphs.iter().map(|(points, labels, name)| {
            //     find_closest_point(scene_position, points.0.iter())
            // }).flatten()

            for (points, labels, name) in graphs.iter() {
                // Is the mouse near a point on this graph?
                if let Some((index, Vec2{x: px, y: py})) = find_closest_point(scene_position, points.0.iter()) {
                    if let Some(highlighted_position) = camera.world_to_viewport(&camera_global_transform, Vec3{x: px, y: py, z: 0.0}) {
                        crosshair.translation.x = highlighted_position.x - window.width() / 2.0;
                        crosshair.translation.y = window.height() / 2.0 - highlighted_position.y;

                        let label = labels.0.get(index).unwrap();
                        // FIXME: hack right now to pad text away from the crosshair, perhaps need a
                        // parent child relationship here so we can position text relative to
                        // cursor?
                        text.sections[0].value = format!("    {} = {}, {:.2}", name.0, label.0, label.1);
                        *visibility = Visibility::Visible;

                        // FIXME: no!
                        return;
                    }
                }
            }
        }
    }
}

fn find_closest_point<'a>(to: Vec2, points: impl Iterator<Item = &'a (f32, f32)>) -> Option<(usize, Vec2)> {
    let mut smallest_distance = f32::MAX;
    let mut item: Option<(usize, Vec2)> = None;
    for (index, (x, y)) in points.enumerate() {
        let actual_distance = f32::sqrt(f32::powi(to.x - x, 2) + f32::powi(to.y - y, 2));
        if actual_distance < smallest_distance {
            smallest_distance = actual_distance;
            item = Some((index, Vec2{x: *x, y: *y}));
        }
    }

    item
}

fn on_resize(
    mut resize_reader: EventReader<WindowResized>,
    mut axes: Query<(&mut Axes, &Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (mut axes, handle) = axes.get_single_mut().unwrap();
    for e in resize_reader.read() {
        axes.view_size.width = e.width;
        axes.view_size.height = e.height;

        let mesh = meshes.get_mut(handle).unwrap();
        axes.update(mesh);
    }
}
