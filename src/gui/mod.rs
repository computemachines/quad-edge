use std::ops::Deref;

use bevy::ecs::event::Events;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::shape::Quad;
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
use quad_edge::mesh::quad::{PrimalDEdgeEntity, VertexEntity};

mod mouse;
mod shapes;

#[derive(Component, Clone, Copy, PartialEq)]
struct PDEdgeEntity(usize);
impl From<PrimalDEdgeEntity> for PDEdgeEntity {
    fn from(e: PrimalDEdgeEntity) -> Self {
        Self(e.0)
    }
}
impl From<PDEdgeEntity> for PrimalDEdgeEntity {
    fn from(e: PDEdgeEntity) -> Self {
        Self(e.0)
    }
}

#[derive(Default)]
struct SelectedDedge(Option<PDEdgeEntity>);

pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(mouse::SimpleMouse)
        .add_plugin(bevy_arrow::ArrowPlugin)
        .insert_resource(ClearColor(Color::WHITE))
        .init_resource::<SelectedDedge>()
        .insert_resource::<f32>(100.0)
        .insert_non_send_resource(mesh)
        .add_event::<MeshEvent>()
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_arrow_frames)
        .add_startup_system(setup_system)
        .add_startup_system(insert_initial_mesh)
        .add_startup_system(init_circles)
        .add_system(ui_system)
        .add_system(swap_mesh_dedges)
        .add_system(update_delaunay_spread)
        .add_system(update_mesh_positions.label("mesh position"))
        .add_system(update_mesh_selected.after("mesh position"))
        .add_system(animate_pulsing_arrow_frame)
        .add_system(debug_in_circle_test)
        .run();
}

enum MeshEvent {
    Swap(PDEdgeEntity),
}

fn swap_mesh_dedges(mut mesh_events: EventReader<MeshEvent>, mesh: NonSend<DelaunayMesh>) {
    for mesh_event in mesh_events.iter() {
        match mesh_event {
            MeshEvent::Swap(e) => mesh.swap_primal((*e).into())
        }
    }
}

fn set_mesh_vertex_spread(mesh: &mut DelaunayMesh, x: f32) {
    let mut v1 = mesh.get_vertex(VertexEntity(2)).borrow_mut();
    let mut v2 = mesh.get_vertex(VertexEntity(3)).borrow_mut();
    v1.x = x as f64;
    v2.x = -x as f64;
}

#[derive(Component)]
struct A;

#[derive(Component)]
struct X;

#[derive(Component)]
struct Y;

#[derive(Component)]
struct B;

fn init_circles(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let handle_A = materials.add(ColorMaterial::from(Color::Rgba {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
        alpha: 0.5,
    }));
    let handle_X = materials.add(ColorMaterial::from(Color::Rgba {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 0.5,
    }));
    let handle_Y = materials.add(ColorMaterial::from(Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 0.0,
        alpha: 0.5,
    }));
    let handle_B = materials.add(ColorMaterial::from(Color::Rgba {
        red: 0.0,
        green: 1.0,
        blue: 0.0,
        alpha: 0.5,
    }));
    let mesh = shapes::build_circle(2); //Mesh::from(Quad::default());
    let mesh_handle = Mesh2dHandle(meshes.add(mesh));

    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            material: handle_A,
            visibility: Visibility { is_visible: true },
            ..Default::default()
        })
        .insert(A);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            material: handle_X,
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(X);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            material: handle_Y,
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Y);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            material: handle_B,
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(B);
}

fn debug_in_circle_test(
    mut commands: Commands,
    mesh: NonSend<DelaunayMesh>,
    selected_dedge: Res<SelectedDedge>,
    mut visibility: Query<&mut Visibility>,
    mut transform: Query<&mut Transform>,
    entity_a: Query<Entity, With<A>>,
    entity_x: Query<Entity, With<X>>,
    entity_y: Query<Entity, With<Y>>,
    entity_b: Query<Entity, With<B>>,
) {
    if selected_dedge.0.is_none() {
        return;
    }

    let xy = selected_dedge.0.unwrap().into();

    let xy = mesh.primal(xy);
    let a = xy.onext().dest().borrow().clone();
    let x = xy.org().borrow().clone();
    let y = xy.dest().borrow().clone();
    let b = xy.oprev().dest().borrow().clone();

    *transform
        .get_component_mut::<Transform>(entity_a.single())
        .unwrap() = Transform {
        translation: ((a.x as f32, a.y as f32, 0.0).into()),
        rotation: Default::default(),
        scale: Vec3::splat(20.0),
    };
    visibility
        .get_component_mut::<Visibility>(entity_a.single())
        .unwrap()
        .is_visible = true;

    *transform
        .get_component_mut::<Transform>(entity_x.single())
        .unwrap() = Transform {
        translation: ((x.x as f32, x.y as f32, 0.0).into()),
        rotation: Default::default(),
        scale: Vec3::splat(20.0),
    };
    visibility
        .get_component_mut::<Visibility>(entity_x.single())
        .unwrap()
        .is_visible = true;

    *transform
        .get_component_mut::<Transform>(entity_y.single())
        .unwrap() = Transform {
        translation: ((y.x as f32, y.y as f32, 0.0).into()),
        rotation: Default::default(),
        scale: Vec3::splat(20.0),
    };
    visibility
        .get_component_mut::<Visibility>(entity_y.single())
        .unwrap()
        .is_visible = true;

    *transform
        .get_component_mut::<Transform>(entity_b.single())
        .unwrap() = Transform {
        translation: ((b.x as f32, b.y as f32, 0.0).into()),
        rotation: Default::default(),
        scale: Vec3::splat(20.0),
    };
    visibility
        .get_component_mut::<Visibility>(entity_b.single())
        .unwrap()
        .is_visible = true;
}

fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut counter: Local<usize>,
    edges: Query<&PDEdgeEntity>,
    mut selected_dedge: ResMut<SelectedDedge>,
    mut spread: ResMut<f32>,
    mut mesh_events: EventWriter<MeshEvent>,
) {
    egui::Window::new("Primal DEdges").show(egui_context.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut *spread, 0.0..=200.0).text("Spread"));
        ui.label(format!(
            "Selected Dedge: {}",
            selected_dedge
                .0
                .map_or("None".to_string(), |e| e.0.to_string())
        ));
        if ui
            .add_enabled(
                selected_dedge.0.is_some(),
                egui::widgets::Button::new("Swap"),
            )
            .clicked()
        {
            mesh_events.send(MeshEvent::Swap(selected_dedge.0.unwrap()));
        };
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

fn insert_initial_mesh(
    mut commands: Commands,
    mesh: NonSend<DelaunayMesh>,
    red_arrow_frame: Query<Entity, (With<RedArrowFrame>, Without<PulsingArrowFrame>)>,
    white_arrow_frame: Query<Entity, (With<WhiteArrowFrame>, Without<PulsingArrowFrame>)>,
) {
    let red_arrow_frame = red_arrow_frame.single();
    let white_arrow_frame = white_arrow_frame.single();

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

            let arrow_frame = if !mesh.is_delaunay(ent) {
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

fn update_delaunay_spread(mut mesh: NonSendMut<DelaunayMesh>, spread: Res<f32>) {
    set_mesh_vertex_spread(&mut *mesh, *spread)
}

fn update_mesh_positions(
    mesh: NonSend<DelaunayMesh>,
    mut query: Query<(&mut bevy_arrow::Arrow, &PDEdgeEntity)>,

    red_arrow_frame: Query<Entity, (With<RedArrowFrame>, Without<PulsingArrowFrame>)>,
    white_arrow_frame: Query<Entity, (With<WhiteArrowFrame>, Without<PulsingArrowFrame>)>,
) {
    let red_arrow_frame = red_arrow_frame.single();
    let white_arrow_frame = white_arrow_frame.single();

    for (mut arrow, ent) in query.iter_mut() {
        let ent = PrimalDEdgeEntity::from(*ent);
        let dedge = mesh.get_primal(ent);

        let origin = dedge.borrow().org;
        let origin = mesh.get_vertex(origin).borrow().clone();

        let dest = mesh.get_primal(ent.sym()).borrow().org;
        let dest = mesh.get_vertex(dest).borrow().clone();

        let arrow_frame = if !mesh.is_delaunay(ent) {
            red_arrow_frame
        } else {
            white_arrow_frame
        };

        arrow.tail.x = origin.x as f32;
        arrow.tail.y = origin.y as f32;
        arrow.head.x = dest.x as f32;
        arrow.head.y = dest.y as f32;
        arrow.arrow_frame = arrow_frame;
    }
}

// fn update_mesh_is_delaunay() {}

fn update_mesh_selected(
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
    info!("laksjdf");

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
        let mesh = meshes.get_mut(mesh_handle.clone().0).unwrap();
        let t = time.seconds_since_startup() * 2.0 % 1.0;
        let weights = mesh.attribute_mut(ATTRIBUTE_WEIGHT).unwrap();
        if let VertexAttributeValues::Float32(values) = weights {
            for i in 4..8 {
                values[i] = 0.3 - t as f32 * 0.2;
            }
        }
    }
}
