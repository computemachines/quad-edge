mod line_mesh;

use bevy::sprite::Mesh2dHandle;
use bevy::{prelude::*, render::mesh::Indices};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::render::render_resource::PrimitiveTopology;

use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;


pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(line_mesh::LineMeshPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        // .add_system(ui_example)
        // .insert_resource(Msaa { samples: 1 })
        .add_startup_system(setup_system)
        .run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn add_lines(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut lines = Mesh::new(PrimitiveTopology::LineList);
    let mut v_pos = vec![
        [0., 0., 0.],
        [1., 1., 0.],
    ];
    lines.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

    let mut v_color = vec![
        [1., 0., 0., 1.],
        [0., 1., 0., 1.],
    ];
    lines.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    let indices: Vec<u32> = vec![0, 1];
    lines.set_indices(Some(Indices::U32(indices)));

    commands.spawn_bundle((
        line_mesh::LineMesh::default(),
        Mesh2dHandle(meshes.add(lines)),

        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    ));
}