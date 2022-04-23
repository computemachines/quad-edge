mod arrow_instance;
mod arrow_shapes;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::{Mesh2dHandle, MaterialMesh2dBundle};
use bevy::{prelude::*, render::mesh::Indices};

// use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;

use self::arrow_instance::{Arrow, ArrowFrame, ArrowsBundle, ATTRIBUTE_WEIGHT};

#[derive(Component, Default)]
struct MousePosition(Vec2);

pub fn explore_mesh(_mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(arrow_instance::ArrowPlugin)
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        // .add_system(ui_example)
        // .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::TOMATO))
        .init_resource::<MousePosition>()
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_default_arrow_frame.label("default arrow frame"))
        .add_startup_system(setup_system.after("default arrow frame"))
        .add_system(animate_arrows)
        .add_system_to_stage(CoreStage::PreUpdate, update_mouse_position)
        .add_system(clicked)
        .run();
}

fn cursor_position_to_model_2d(window: &Window, position: Vec2) -> Vec2 {
    Vec2::new(
        position.x - 0.5 * window.width(),
        position.y - 0.5 * window.height(),
    )
}

fn update_mouse_position(
    windows: Res<Windows>,
    mut mouse_position: ResMut<MousePosition>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut arrows: Query<&mut Arrow>,
) {
    if let Some(cursor_moved) = cursor_moved_events.iter().last().take() {
        mouse_position.0 =
            cursor_position_to_model_2d(windows.get_primary().unwrap(), cursor_moved.position);
        for mut arrow in arrows.iter_mut() {
            arrow.1 = (mouse_position.0, 0.0).into();
        }
    }
}

fn clicked(
    mut commands: Commands,
    mouse_position: Res<MousePosition>,
    mouse_button_input: Res<Input<MouseButton>>,
    entity: Query<Entity, With<ArrowFrame>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        commands
            .spawn()
            .insert(Arrow((mouse_position.0, 0.0).into(), (mouse_position.0, 0.0).into(), entity.single()));
    }
}

pub fn setup_default_arrow_frame(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let mesh_handle = Mesh2dHandle(meshes.add(arrow_shapes::build_arrow_strip_mesh()));

    let texture_handle: Handle<Image> = asset_server.load("images/arrow_atlas.png");

    commands
        .spawn_bundle(ArrowsBundle {
            mesh: mesh_handle,
            arrow_frame_marker: ArrowFrame::default(),
            texture: texture_handle,
            ..Default::default()
        });
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    entity: Query<Entity, With<ArrowFrame>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let entity = entity.single();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(MaterialMesh2dBundle {
        material: materials.add(ColorMaterial::from(Color::WHITE)),
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::splat(100.0)),
        ..Default::default()
    });

    // commands.spawn().insert(Arrow(
    //     Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    //     Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
    //     entity,
    // ));
    // commands
    //     .spawn()
    //     .insert(Arrow(
    //         Transform::from_translation(Vec3::new(-100.0, 250.0, 0.0))
    //             .with_scale(Vec3::new(10.0, 10.0, 10.0)),
    //         Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
    //         entity,
    //     ))
    //     .insert(Animated);
    // commands.spawn().insert(Arrow(
    //     Transform::from_translation(Vec3::new(70.0, 70.0, 0.0)),
    //     Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
    //     entity,
    // ));
}

#[derive(Component)]
struct Animated;

fn animate_arrows(time: Res<Time>, mut arrows: Query<&mut Arrow, With<Animated>>) {
    for mut arrow in arrows.iter_mut() {
        arrow.0.y = 250.0 - 10.0 * time.seconds_since_startup() as f32;
    }
}
