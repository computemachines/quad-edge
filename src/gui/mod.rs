mod line_mesh;
mod shapes;

use std::time::Duration;

use bevy::app::{AppExit, ScheduleRunnerSettings};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle, SpecializedMaterial2d};
use bevy::utils::tracing::span::Entered;
use bevy::{prelude::*, render::mesh::Indices};

// use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;

#[derive(Default, Debug)]
struct MousePosition(Vec3);

#[derive(Debug, StageLabel, Hash, PartialEq, Eq, Clone)]
struct ArrowUpdateStage;

#[derive(Debug, Default)]
struct DebugTimer(Timer);

pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f32(
            1.0,
        )))
        .add_plugins(DefaultPlugins)
        // .add_plugin(line_mesh::LineMeshPlugin)
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        // .add_system(ui_example)
        .insert_resource(Msaa { samples: 1 })
        .init_resource::<MousePosition>()
        .insert_resource(DebugTimer(Timer::from_seconds(1.0, true)))
        .add_stage_before(
            CoreStage::PostUpdate,
            ArrowUpdateStage,
            SystemStage::parallel(),
        )
        .add_startup_system(setup_system)
        .add_startup_system_to_stage(StartupStage::PreStartup, arrow_setup)
        // .add_system(add_arrow_children)
        .add_system(update_mouse_position)
        .add_system_to_stage(ArrowUpdateStage, add_arrow_children)
        .add_system(exit_on_escape)
        .add_system(clicked)
        .add_system(debug)
        .run();
}

fn update_mouse_position(
    window: Res<Windows>,
    mut mouse_position: ResMut<MousePosition>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    cursor_moved_events.iter().last().map(|event| {
        mouse_position.0 = Vec3::new(
            event.position.x - window.primary().width() / 2.0,
            event.position.y - window.primary().height() / 2.0,
            0.0,
        )
    });
}

struct ArrowConfig<M: SpecializedMaterial2d> {
    start_cap: MaterialMesh2dBundle<M>,
    end_cap: MaterialMesh2dBundle<M>,
    line: MaterialMesh2dBundle<M>,
}

#[derive(Bundle)]
pub struct ArrowBundle {
    pub end: Arrow,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Component)]
pub struct Arrow(Vec3);

impl ArrowBundle {
    fn new(start: Vec3, end: Vec3) -> Self {
        Self {
            end: Arrow(end - start),
            transform: Transform::from_translation(start),
            global_transform: GlobalTransform::default(),
        }
    }
}

fn arrow_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let circle: Mesh2dHandle = meshes.add(Mesh::from(shapes::Circle::new(10.0, 16))).into();
    let arrow = meshes.add(Mesh::from(shapes::Circle::new(10.0, 3))).into();
    let line = meshes
        .add(Mesh::from(shape::Quad::new(Vec2::new(1.0, 1.0))))
        .into();

    let solid_black = materials.add(ColorMaterial::from(Color::BLACK));

    let arrow_config = ArrowConfig {
        end_cap: MaterialMesh2dBundle {
            mesh: arrow,
            material: solid_black.clone(),
            ..Default::default()
        },
        start_cap: MaterialMesh2dBundle {
            mesh: circle.clone(),
            material: solid_black.clone(),
            ..Default::default()
        },
        line: MaterialMesh2dBundle {
            mesh: line,
            material: solid_black.clone(),
            ..Default::default()
        },
    };
    commands.insert_resource(arrow_config);
}

fn clicked(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    arrow_config: Res<ArrowConfig<ColorMaterial>>,
    mouse_position: Res<MousePosition>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        commands.spawn_bundle(ArrowBundle::new(
            Vec3::new(30.0, 25.0, 0.0),
            Vec3::new(40.0, 15.0, 0.0),
        ));
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: arrow_config.end_cap.mesh.clone(),
                material: arrow_config.end_cap.material.clone(),
                transform: Transform::from_translation(Vec3::new(-50.0, -50.0, 0.0)),
                global_transform: GlobalTransform::from_translation(Vec3::default()),
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(MaterialMesh2dBundle {
                    mesh: arrow_config.end_cap.mesh.clone(),
                    material: arrow_config.end_cap.material.clone(),
                    global_transform: GlobalTransform::from_translation(Vec3::default()),
                    ..Default::default()
                });
            });
    }
}

fn add_arrow_children(
    mut commands: Commands,
    arrow_config: Res<ArrowConfig<ColorMaterial>>,
    new_arrows: Query<(Entity, &Arrow), Changed<Arrow>>,
) {
    for (entity, new_arrow) in new_arrows.iter() {
        info!("New arrow children");
        let start = commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: arrow_config.start_cap.mesh.clone(),
                material: arrow_config.start_cap.material.clone(),
                ..Default::default()
            })
            .insert(Parent(entity))
            .id();
        info!("start cap: {:?}", start);
        let end = commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: arrow_config.end_cap.mesh.clone(),
                material: arrow_config.end_cap.material.clone(),
                transform: Transform::from_translation(new_arrow.0),
                ..Default::default()
            })
            .insert(Parent(entity))
            .id();
        info!("end cap: {:?}", end);
    }
}

fn debug(
    time: Res<Time>,
    mut timer: ResMut<DebugTimer>,
    query_no_parent: Query<(Entity, &Transform, &GlobalTransform), Without<Parent>>,
    query_parents: Query<(Entity, &Transform, &GlobalTransform, &Parent)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        info!("roots -----------");
        for (entity, local, global) in query_no_parent.iter() {
            info!("{:?}.local = {}", entity, local.translation);
            info!("{:?}.global = {}", entity, global.translation);
        }
        info!("children ---------");
        for (entity, local, global, parent) in query_parents.iter() {
            info!("{:?}.parent = {:?}", entity, parent);
            info!("{:?}.local = {}", entity, local.translation);
            info!("{:?}.global = {}", entity, global.translation);
        }
    }
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn exit_on_escape(keyboard_input: Res<Input<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}
