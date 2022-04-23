use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;

// use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;
use quad_edge::mesh::quad::PrimalDEdgeEntity;

mod mouse;

pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(mouse::SimpleMouse)
        .add_plugin(bevy_arrow::ArrowPlugin)
        .insert_resource(ClearColor(Color::WHITE))
        .insert_non_send_resource(mesh)
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        // .add_system(ui_example)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_arrow_frames)
        .add_startup_system(draw_mesh)
        .add_startup_system(setup_system)
        .run();
}

fn draw_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mesh: NonSend<DelaunayMesh>,
    red_arrow_frame: Query<Entity, With<RedArrowFrame>>,
    white_arrow_frame: Query<Entity, With<WhiteArrowFrame>>,
) {
    // let mesh_handle = Mesh2dHandle(meshes.add(bevy_arrow::build_arrow_strip_mesh()));

    // let texture_handle: Handle<Image> = asset_server.load("images/node_arrow_80x16.png");

    // let arrow_frame = commands
    //     .spawn_bundle(bevy_arrow::ArrowsBundle {
    //         mesh: mesh_handle,
    //         texture: texture_handle,
    //         local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
    //         ..Default::default()
    //     })
    //     .id();

    info!("setting up mesh");
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

            commands.spawn().insert(bevy_arrow::Arrow {
                tail: Vec3::new(origin.x as f32, origin.y as f32, 0.0),
                head: Vec3::new(dest.x as f32, dest.y as f32, 0.0),
                arrow_frame: red_arrow_frame.single(),
                width: 16.0,
            });
        }
    }
}

#[derive(Component)]
struct WhiteArrowFrame;

#[derive(Component)]
struct RedArrowFrame;

pub fn setup_arrow_frames(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let mesh_handle = Mesh2dHandle(meshes.add(bevy_arrow::build_arrow_strip_mesh()));

    let white_texture_handle: Handle<Image> = asset_server.load("images/node_arrow_80x16.png");
    let red_texture_handle: Handle<Image> = asset_server.load("images/node_arrow_red_80x16.png");

    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: mesh_handle.clone(),
            texture: white_texture_handle,
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(WhiteArrowFrame);
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: mesh_handle,
            texture: red_texture_handle,
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(RedArrowFrame);
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    entity: Query<Entity, With<WhiteArrowFrame>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let entity = entity.single();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // commands.spawn_bundle(MaterialMesh2dBundle {
    //     material: materials.add(ColorMaterial::from(Color::WHITE)),
    //     mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
    //     transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
    //         .with_scale(Vec3::splat(9999.0)),
    //     ..Default::default()
    // });
    commands.spawn().insert(bevy_arrow::Arrow {
        arrow_frame: entity,
        tail: Vec3::new(-100.0, -100.0, -1.0),
        head: Vec3::new(100.0, -100.0, -1.0),
        width: 16.0,
    });
}
