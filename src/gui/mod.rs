use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use bevy_arrow::ATTRIBUTE_WEIGHT;
use bevy_egui::egui::{Label, RichText, Sense};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;
use quad_edge::mesh::quad::{PrimalDEdgeEntity, VertexEntity};

mod default_arrows;
mod mesh_draw;
mod mouse;
mod shapes;

pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(mouse::SimpleMouse)
        .add_plugin(bevy_arrow::ArrowPlugin)
        .add_plugin(default_arrows::DefaultArrows)
        .insert_non_send_resource(mesh)
        .add_plugin(mesh_draw::MeshDraw)
        .insert_resource(ClearColor(Color::WHITE))
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        .add_startup_system(setup_system)
        .add_startup_system(init_new_node)
        .add_system(ui_system)
        .add_system(move_node_to_click)
        .run();
}

#[derive(Component)]
struct NodeSprite;

fn init_new_node(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("images/node-open.png"),
            ..Default::default()
        })
        .insert(NodeSprite);
}

fn move_node_to_click(
    mut transform: Query<&mut Transform, With<NodeSprite>>,
    mouse_button: Res<Input<MouseButton>>,
    mouse_position: Res<mouse::MousePosition>,
    mut mesh_events: EventWriter<mesh_draw::MeshEvent>,
) {
    let mut transform = transform.single_mut();
    if mouse_button.just_pressed(MouseButton::Left) {
        *transform = Transform::from_translation((mouse_position.0, 0.0).into());
        mesh_events.send(mesh_draw::MeshEvent::Insert(mouse_position.0));
    }
}

fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    edges: Query<&mesh_draw::PDEdgeEntity>,
    mut selected_dedge: ResMut<mesh_draw::SelectedDedge>,
    mut spread: ResMut<f32>,
    mut mesh_events: EventWriter<mesh_draw::MeshEvent>,
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
            mesh_events.send(mesh_draw::MeshEvent::Swap(selected_dedge.0.unwrap()));
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
