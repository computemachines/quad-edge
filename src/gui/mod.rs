mod line_mesh;
mod shapes;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::{prelude::*, render::mesh::Indices};

use bevy_egui::{egui, EguiContext, EguiPlugin};
use quad_edge::delaunay_voronoi::DelaunayMesh;

pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugin(line_mesh::LineMeshPlugin)
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        // .add_system(ui_example)
        // .insert_resource(Msaa { samples: 1 })
        .add_startup_system(setup_system)
        // .add_startup_system(add_lines)
        // .add_system(animate_view)
        .add_system(animate_line)
        // .add_system(debug)
        .run();
}

fn animate_view(mut transform: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    // transform.single_mut().translation.y = time.seconds_since_startup() as f32 * 10.;
}

#[derive(Component)]
struct AnimatedLine;

fn animate_line(mut query: Query<(Entity, &mut Transform), With<AnimatedLine>>, time: Res<Time>) {
    if let (entity, mut transform) = query.single_mut() {
        transform.translation.x = time.seconds_since_startup() as f32 * 10.0;
        // info!("Animate line called with: {}", entity.id());
    } else {
        error!("Animate line system called with bad data");
    }
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::ALICE_BLUE,
                custom_size: Some(Vec2::new(1., 100.)),
                ..Default::default()
            },
            transform: Transform::from_rotation(Quat::from_rotation_z(3.14 / 80.)),
            ..Default::default()
        })
        .insert(AnimatedLine);
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shapes::Circle::new(100., 16))).into(),
        material: materials.add(ColorMaterial::from(Color::GREEN)),
        ..Default::default()
    });
}

fn debug(query: Query<(Entity, &Mesh2dHandle)>, meshes: Res<Assets<Mesh>>) {
    for (entity, mesh) in query.iter() {
        info!("Entity({}).mesh = {:?}", entity.id(), meshes.get(&mesh.0));
    }
}

fn add_lines(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut lines = Mesh::new(PrimitiveTopology::LineList);
    let mut v_pos = vec![[0., 0., 0.], [100., 100., 0.]];
    lines.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

    let mut v_color = vec![[1., 0., 0., 1.], [0., 1., 0., 1.]];
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
        AnimatedLine,
    ));
}
