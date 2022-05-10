use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use bevy_arrow::ATTRIBUTE_WEIGHT;
use bevy_egui::egui::{Label, RichText, Sense};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;
use cgmath::Point2;
use quad_edge::delaunay_voronoi::DelaunayMesh;
use quad_edge::mesh::quad::{PrimalDEdgeEntity, VertexEntity};

use self::animate_mesh::PointTarget;

mod animate_mesh;
mod default_arrows;
mod mesh_draw;
mod mouse;
mod shapes;

pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        // .insert_resource(ReportExecutionOrderAmbiguities)
        // .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(mouse::SimpleMouse)
        .add_plugin(bevy_arrow::ArrowPlugin)
        .add_plugin(default_arrows::DefaultArrows)
        .insert_non_send_resource(mesh)
        .add_plugin(mesh_draw::MeshDraw)
        .add_plugin(animate_mesh::AnimateMesh)
        .insert_resource(ClearColor(Color::WHITE))
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        .add_startup_system(setup_system)
        // .add_startup_system_to_stage(StartupStage::PostStartup, initial_events)
        .add_system(ui_system)
        .init_resource::<UiWindowRect>()
        .add_system(move_node_to_click)
        .run();
}

fn move_node_to_click(
    windows: Res<Windows>,
    ui_window_rect: Res<UiWindowRect>,
    // mut transform: Query<&mut Transform, With<NodeSprite>>,
    mouse_button: Res<Input<MouseButton>>,
    mouse_position: Res<mouse::MousePosition>,
    // mut mesh_events: EventWriter<mesh_draw::MeshEvent>,
    mut animate_events: EventWriter<animate_mesh::AnimateMeshEvent<'static>>,
) {
    let (window_min, window_max): (Vec2, Vec2) = ui_window_rect
        .0
        .map(|rect| {
            let min = rect.min;
            let max = rect.max;
            let a =
                mouse::inverted_screen_space_to_model_2d(windows.primary(), (min.x, min.y).into());
            let b =
                mouse::inverted_screen_space_to_model_2d(windows.primary(), (max.x, max.y).into());
            ((a.x, b.y).into(), (b.x, a.y).into())
        })
        .unwrap_or_default();

    if mouse_button.just_pressed(MouseButton::Left) {
        let pos = mouse_position.0;
        if pos.x > window_min.x
            && pos.y > window_min.y
            && pos.x < window_max.x
            && pos.y < window_max.y
        {
            info!("clicked inside window");
            return;
        }
        info!("mouse clicked outside window");
        // *transform = Transform::from_translation((mouse_position.0, 0.0).into());
        animate_events.send(animate_mesh::AnimateMeshEvent::SetTargetPosition(
            Some("Set target point from mouse click"),
            mouse_position.0,
        ))
        // mesh_events.send(mesh_draw::MeshEvent::Insert(mouse_position.0));
    }
}

fn initial_events(mut animate_events: EventWriter<animate_mesh::AnimateMeshEvent<'static>>) {
    use animate_mesh::AnimateMeshEvent::*;
    animate_events.send(SetTargetPosition(None, (70.0, 70.0).into()));
    animate_events.send(SetTargetVisibility(None, true));
}

#[derive(Default)]
struct UiWindowRect(Option<egui::Rect>);

fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut ui_window_rect: ResMut<UiWindowRect>,
    edges: Query<&mesh_draw::PDEdgeEntity>,
    active_dedge: Res<animate_mesh::ActiveDedge>,
    mut spread: ResMut<f32>,
    mut mesh: NonSendMut<DelaunayMesh>,
    target_point: Query<&Transform, With<PointTarget>>,
    mut mesh_events: EventWriter<mesh_draw::MeshEvent>,
    mut animate_events: EventWriter<animate_mesh::AnimateMeshEvent<'static>>,
) {
    let window = egui::Window::new("Primal DEdges");
    window.show(egui_context.ctx_mut(), |ui| {
        ui_window_rect.0.replace(ui.ctx().used_rect());

        ui.add(egui::Slider::new(&mut *spread, 0.0..=200.0).text("Spread"));
        ui.label(format!(
            "Selected Dedge: {}",
            active_dedge
                .0
                .map_or("None".to_string(), |e| e.0.to_string())
        ));
        if ui
            .add_enabled(
                active_dedge.0.is_some(),
                egui::widgets::Button::new("Swap"),
            )
            .clicked()
        {
            mesh_events.send(mesh_draw::MeshEvent::Swap(active_dedge.0.unwrap()));
        };
        if ui.button("locate").clicked() {
            let x = target_point.single().translation;
            let found = mesh.locate_point(Point2::new(x.x, x.y));
            animate_events.send(animate_mesh::AnimateMeshEvent::SetActiveDedge(Some("located"), Some(found.into())));
        }
        if ui.button("start animation").clicked() {
            animate_events.send(animate_mesh::AnimateMeshEvent::BeginLocateAnimation(Some(
                "Locate Test Point",
            )));
        }
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for i in edges.iter() {
                    let mut text = RichText::new(format!("{}", i.0));
                    if let Some(selected) = active_dedge.0 {
                        if selected == *i {
                            text = text.strong();
                        }
                    }
                    let label = Label::new(text).sense(Sense::click());
                    if ui.add(label).clicked() {
                        animate_events.send(animate_mesh::AnimateMeshEvent::SetActiveDedge(
                            Some("manually selected"),
                            Some(*i),
                        ));
                        animate_events.send(animate_mesh::AnimateMeshEvent::SetHighlightDedge(
                            Some("highlight test"),
                            Color::YELLOW,
                            Some(*i),
                        ));
                    }
                }
            });
    });
}

fn setup_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
