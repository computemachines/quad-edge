use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::WgpuFeatures;
use bevy::render::settings::WgpuSettings;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use bevy::utils::tracing::span::Entered;
use bevy_arrow::ATTRIBUTE_WEIGHT;
use bevy_egui::egui::{Label, RichText, Sense};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;
use quad_edge::mesh::quad::PrimalDEdgeEntity;

mod mouse;

#[derive(Component, Clone, Copy, PartialEq)]
struct PDEdgeEntity(usize);
impl From<PrimalDEdgeEntity> for PDEdgeEntity {
    fn from(e: PrimalDEdgeEntity) -> Self {
        Self(e.0)
    }
}

#[derive(Default)]
struct SelectedDedge(Option<PDEdgeEntity>);

pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(mouse::SimpleMouse)
        .add_plugin(bevy_arrow::ArrowPlugin)
        .insert_resource(ClearColor(Color::WHITE))
        .init_resource::<SelectedDedge>()
        .insert_non_send_resource(mesh)
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_arrow_frames)
        .add_startup_system(setup_system)
        .add_startup_system(add_mesh)
        .add_system(ui_system)
        .add_system(update_mesh)
        .add_system(animate_pulsing_arrow_frame)
        .run();
}

fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut counter: Local<usize>,
    edges: Query<&PDEdgeEntity>,
    mut selected_dedge: ResMut<SelectedDedge>,
) {
    egui::Window::new("Primal DEdges").show(egui_context.ctx_mut(), |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for i in edges.iter() {
                    let mut text = RichText::new(format!("{}", i.0));
                    if let Some(selected) = selected_dedge.0 {
                        if selected == *i {
                            text = text.strong();
                        }
                    }
                    let label = Label::new(text).sense(Sense::click());
                    if ui.add(label).clicked() {
                        selected_dedge.0 = Some(*i);
                    }
                }
            });
    });
}

fn add_mesh(
    mut commands: Commands,
    mesh: NonSend<DelaunayMesh>,
    red_arrow_frame: Query<Entity, (With<RedArrowFrame>, Without<PulsingArrowFrame>)>,
    white_arrow_frame: Query<Entity, (With<WhiteArrowFrame>, Without<PulsingArrowFrame>)>,
) {
    let red_arrow_frame = red_arrow_frame.single();
    let white_arrow_frame = white_arrow_frame.single();

    info!("setting up mesh");
    for (ent, dedge) in mesh
        .primal_dedges
        .iter()
        .enumerate()
        .map(|(i, d)| (PrimalDEdgeEntity(i), d))
    {
        if let Some(dedge) = dedge {
            let origin = dedge.borrow().org;
            let origin = mesh.get_vertex(origin).borrow().clone();

            let dest = mesh.get_primal(ent.sym()).borrow().org;
            let dest = mesh.get_vertex(dest).borrow().clone();

            let arrow_frame = if mesh.is_delaunay(ent) {
                red_arrow_frame
            } else {
                white_arrow_frame
            };

            commands
                .spawn()
                .insert(bevy_arrow::Arrow {
                    tail: Vec3::new(origin.x as f32, origin.y as f32, 0.0),
                    head: Vec3::new(dest.x as f32, dest.y as f32, 0.0),
                    arrow_frame,
                    width: 16.0,
                })
                .insert(PDEdgeEntity::from(ent));
        }
    }
}

fn update_mesh(
    selected_dedge: Res<SelectedDedge>,
    red_arrow_frame: Query<Entity, (With<RedArrowFrame>, Without<PulsingArrowFrame>)>,
    white_arrow_frame: Query<Entity, (With<WhiteArrowFrame>, Without<PulsingArrowFrame>)>,
    pulsing_red_arrow_frame: Query<Entity, (With<RedArrowFrame>, With<PulsingArrowFrame>)>,
    pulsing_white_arrow_frame: Query<Entity, (With<WhiteArrowFrame>, With<PulsingArrowFrame>)>,
    mut query: Query<(&mut bevy_arrow::Arrow, &PDEdgeEntity)>,
) {
    let red_arrow_frame = red_arrow_frame.single();
    let white_arrow_frame = white_arrow_frame.single();
    let pulsing_red_arrow_frame = pulsing_red_arrow_frame.single();
    let pulsing_white_arrow_frame = pulsing_white_arrow_frame.single();

    let set_pulsing_from = |old_frame: Entity, pulsing: bool| -> Entity {
        if old_frame == red_arrow_frame || old_frame == pulsing_red_arrow_frame {
            match pulsing {
                true => pulsing_red_arrow_frame,
                false => red_arrow_frame,
            }
        } else {
            match pulsing {
                true => pulsing_white_arrow_frame,
                false => white_arrow_frame,
            }
        }
    };

    for (mut arrow, dedge) in query.iter_mut() {
        arrow.arrow_frame = set_pulsing_from(arrow.arrow_frame, selected_dedge.0 == Some(*dedge));
    }
}

#[derive(Component)]
struct WhiteArrowFrame;

#[derive(Component)]
struct RedArrowFrame;

#[derive(Component)]
struct PulsingArrowFrame;

pub fn setup_arrow_frames(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_handle = Mesh2dHandle(meshes.add(bevy_arrow::build_arrow_strip_mesh()));
    let pulsing_mesh_handle = Mesh2dHandle(meshes.add(bevy_arrow::build_arrow_strip_mesh()));

    let white_texture_handle: Handle<Image> = asset_server.load("images/node_arrow_80x16.png");
    let red_texture_handle: Handle<Image> = asset_server.load("images/node_arrow_red_80x16.png");

    // Static arrow frames
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: mesh_handle.clone(),
            texture: white_texture_handle.clone(),
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(WhiteArrowFrame);
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: mesh_handle,
            texture: red_texture_handle.clone(),
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(RedArrowFrame);

    // pulsing arrow frames
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: pulsing_mesh_handle.clone(),
            texture: white_texture_handle,
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(WhiteArrowFrame)
        .insert(PulsingArrowFrame);
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: pulsing_mesh_handle,
            texture: red_texture_handle,
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(RedArrowFrame)
        .insert(PulsingArrowFrame);
}

fn setup_system(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // asset_server: Res<AssetServer>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // commands.spawn_bundle(MaterialMesh2dBundle {
    //     material: materials.add(ColorMaterial::from(Color::WHITE)),
    //     mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
    //     transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
    //         .with_scale(Vec3::splat(9999.0)),
    //     ..Default::default()
    // });
}

// TODO: This could be improved. Does not check for repeated mesh_handle.
fn animate_pulsing_arrow_frame(
    mesh_handles: Query<&Mesh2dHandle, With<PulsingArrowFrame>>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    for mesh_handle in mesh_handles.iter() {
        let mut mesh = meshes.get_mut(mesh_handle.clone().0).unwrap();
        let t = time.seconds_since_startup() * 2.0 % 1.0;
        let weights = mesh.attribute_mut(ATTRIBUTE_WEIGHT).unwrap();
        if let VertexAttributeValues::Float32(values) = weights {
            for i in 4..8 {
                values[i] = 0.3 - t as f32 * 0.2;
            }
        }
    }
}
