mod arrow_instance;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::Mesh2dHandle;
use bevy::{prelude::*, render::mesh::Indices};

// use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;

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
        .add_startup_system(add_lines)
        .run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn add_lines(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut lines = Mesh::new(PrimitiveTopology::LineList);
    let mut v_pos = vec![[0., 0., 0.], [100., 100., 0.]];
    lines.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

    let mut v_color = vec![[1., 0., 0., 1.], [0., 1., 0., 1.]];
    lines.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    let indices: Vec<u32> = vec![0, 1];
    lines.set_indices(Some(Indices::U32(indices)));

    let instances = vec![
        arrow_instance::InstanceData {
            position: Vec3::new(0.0, 0.0, 0.0),
            color: [1.0, 0.0, 0.0, 1.0],
        },
        arrow_instance::InstanceData {
            position: Vec3::new(100.0, 0.0, 0.0),
            color: [1.0, 0.0, 1.0, 1.0],
        },
    ];

    println!("{:?}", instances);

    commands.spawn_bundle((
        // arrow_instance::ArrowHead(Transform::from_translation(Vec3::X)),
        arrow_instance::Arrow,
        Mesh2dHandle(meshes.add(lines)),
        arrow_instance::InstanceMaterialData(instances),
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    ));
}
