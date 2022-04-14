mod arrow_instance;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::Mesh2dHandle;
use bevy::{prelude::*, render::mesh::Indices};

// use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;

use self::arrow_instance::{Arrow, ArrowInstances, ArrowsBundle, ATTRIBUTE_WEIGHT};

pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(arrow_instance::ArrowPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        // .add_system(ui_example)
        // .insert_resource(Msaa { samples: 1 })
        .add_startup_system(setup_system)
        .add_system(animate_arrows)
        .run();
}

fn setup_system(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let mut lines = Mesh::new(PrimitiveTopology::TriangleList);

    let v_color = vec![
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    lines.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    let v_pos = vec![
        [0.0, 2.0, 0.0],
        [0.0, -2.0, 0.0],
        [0.0, 2.0, 0.0],
        [0.0, -2.0, 0.0],
    ];
    lines.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

    let indices: Vec<u32> = vec![0, 2, 1, 2, 1, 3];
    lines.set_indices(Some(Indices::U32(indices)));

    lines.set_attribute(ATTRIBUTE_WEIGHT, vec![0.2, 0.2, 1.0, 1.0]);

    let mesh_handle = Mesh2dHandle(meshes.add(lines));

    let entity = commands
        .spawn_bundle(ArrowsBundle {
            mesh: mesh_handle,
            ..Default::default()
        })
        .id();

    commands.spawn().insert(Arrow(
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        entity,
    ));
    commands
        .spawn()
        .insert(Arrow(
            Transform::from_translation(Vec3::new(-100.0, 250.0, 0.0))
                .with_scale(Vec3::new(10.0, 10.0, 10.0)),
            Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
            entity,
        ))
        .insert(Animated);
    commands.spawn().insert(Arrow(
        Transform::from_translation(Vec3::new(70.0, 70.0, 0.0)),
        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        entity,
    ));
}

#[derive(Component)]
struct Animated;

fn animate_arrows(time: Res<Time>, mut arrows: Query<&mut Arrow, With<Animated>>) {
    for mut arrow in arrows.iter_mut() {
        arrow.0.translation.y = 250.0 - 10.0* time.seconds_since_startup() as f32;
    }
}
