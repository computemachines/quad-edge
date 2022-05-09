use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::sprite::Mesh2dHandle;

use bevy_arrow::ATTRIBUTE_WEIGHT;

#[derive(SystemParam)]
pub struct DefaultArrowsParam<'w, 's> {
    pub white: Query<'w, 's, Entity, (With<WhiteArrowFrame>, Without<PulsingArrowFrame>)>,
    pub red: Query<'w, 's, Entity, (With<RedArrowFrame>, Without<PulsingArrowFrame>)>,
    pub scoop: Query<'w, 's, Entity, (With<ScoopArrowFrame>, Without<PulsingArrowFrame>)>,
    pub pulsing_white: Query<'w, 's, Entity, (With<WhiteArrowFrame>, With<PulsingArrowFrame>)>,
    pub pulsing_red: Query<'w, 's, Entity, (With<RedArrowFrame>, With<PulsingArrowFrame>)>,
    pub pulsing_scoop: Query<'w, 's, Entity, (With<ScoopArrowFrame>, With<PulsingArrowFrame>)>,
}

pub struct DefaultArrows;
impl Plugin for DefaultArrows {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup_arrow_frames)
            .add_system(animate_pulsing_arrow_frame);
    }
}

#[derive(Component)]
pub struct WhiteArrowFrame;

#[derive(Component)]
pub struct RedArrowFrame;

#[derive(Component)]
pub struct ScoopArrowFrame;

#[derive(Component)]
pub struct PulsingArrowFrame;

pub fn setup_arrow_frames(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_handle = Mesh2dHandle(meshes.add(bevy_arrow::build_arrow_strip_mesh()));
    let pulsing_mesh_handle = Mesh2dHandle(meshes.add(bevy_arrow::build_arrow_strip_mesh()));

    let white_texture_handle: Handle<Image> = asset_server.load("images/node_arrow_80x16.png");
    let red_texture_handle: Handle<Image> = asset_server.load("images/node_arrow_red_80x16.png");
    let scoop_texture_handle: Handle<Image> = asset_server.load("images/node_scoop_80x16.png");
    info!("laksjdf");

    // Static arrow frames
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: mesh_handle.clone(),
            texture: white_texture_handle.clone(),
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(WhiteArrowFrame);
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: mesh_handle.clone(),
            texture: red_texture_handle.clone(),
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(RedArrowFrame);

    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: mesh_handle,
            texture: scoop_texture_handle.clone(),
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(ScoopArrowFrame);

    // pulsing arrow frames
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: pulsing_mesh_handle.clone(),
            texture: white_texture_handle,
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(WhiteArrowFrame)
        .insert(PulsingArrowFrame);
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: pulsing_mesh_handle.clone(),
            texture: red_texture_handle,
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(RedArrowFrame)
        .insert(PulsingArrowFrame);
    commands
        .spawn_bundle(bevy_arrow::ArrowsBundle {
            mesh: pulsing_mesh_handle,
            texture: scoop_texture_handle,
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(ScoopArrowFrame)
        .insert(PulsingArrowFrame);
}

// TODO: This could be improved. Does not check for repeated mesh_handle.
fn animate_pulsing_arrow_frame(
    mesh_handles: Query<&bevy::sprite::Mesh2dHandle, With<PulsingArrowFrame>>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    for mesh_handle in mesh_handles.iter() {
        let mesh = meshes.get_mut(mesh_handle.clone().0).unwrap();
        let t = time.seconds_since_startup() * 2.0 % 1.0;
        let weights = mesh.attribute_mut(ATTRIBUTE_WEIGHT).unwrap();
        if let VertexAttributeValues::Float32(values) = weights {
            for i in 4..8 {
                values[i] = 0.3 - t as f32 * 0.2;
            }
        }
    }
}
