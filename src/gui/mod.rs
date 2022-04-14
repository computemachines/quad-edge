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
        .add_system(debug)
        .run();
}

fn setup_system(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    
    let mut lines = Mesh::new(PrimitiveTopology::TriangleList);

    let v_color = vec![[0.1, 0.2, 0.3, 0.4], [0.5, 0.6, 0.7, 0.8], [1.0, 0.0, 0.0, 1.0]];
    lines.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    let v_pos = vec![[10.0, 11.0, 12.0], [20.0, 21.0, 22.0], [-100.0, 0.0, 0.0]];
    lines.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

    let indices: Vec<u32> = vec![0, 1, 2];
    lines.set_indices(Some(Indices::U32(indices)));

    lines.set_attribute(ATTRIBUTE_WEIGHT, vec![0.15625, 0.99, 0.5]);

    {
        let data = lines.get_vertex_buffer_data();
        let mut iter = data.iter();
        let mut count = 0;
        while let Some(val) = iter.next() {
            if count % 4 == 0 {
                println!("");
            }
            if count % 16 == 0 {
                println!("");
                println!("");
            }
            print!("{:02X?} ", val);
            count += 1;
        }
    }
    println!("");
    println!("");
    let mesh_handle = Mesh2dHandle(meshes.add(lines));
    


    let entity = commands.spawn_bundle(ArrowsBundle {
        mesh: mesh_handle,
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