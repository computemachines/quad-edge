mod arrow_instance;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::Mesh2dHandle;
use bevy::{prelude::*, render::mesh::Indices};

// use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;

use self::arrow_instance::{Arrow, ArrowInstances, ArrowsBundle};

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
        .add_system(debug)
        .run();
}

fn setup_system(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let mut lines = Mesh::new(PrimitiveTopology::LineList);
    let v_pos = vec![[0., 0., 0.], [100., 100., 0.]];
    lines.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

    let v_color = vec![[1., 0., 0., 1.], [0., 1., 0., 1.]];
    lines.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    let indices: Vec<u32> = vec![0, 1];
    lines.set_indices(Some(Indices::U32(indices)));

    let mesh_handle = Mesh2dHandle(meshes.add(lines));

    let entity = commands.spawn_bundle(ArrowsBundle {
        mesh: mesh_handle,
        instances: ArrowInstances(Vec::new()),
        ..Default::default()
    }).id();
    
    commands.spawn().insert(Arrow(
        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
        entity,
    ));
}

fn debug(query: Query<(Entity, &ArrowInstances)>){
    info!("debug");
    for a in query.iter() {
        info!("{:?}", a);
    }
}