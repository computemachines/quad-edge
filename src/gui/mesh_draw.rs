use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use quad_edge::delaunay_voronoi::DelaunayMesh;
use quad_edge::mesh::quad::{PrimalDEdgeEntity, VertexEntity};

#[derive(Clone, Hash, Debug, PartialEq, Eq, StageLabel)]
pub enum MeshStage {
    DelaunayMeshUpdate,
    DelaunayMeshRead,
}
pub struct MeshDraw;
impl Plugin for MeshDraw {
    fn build(&self, app: &mut App) {
        app.insert_resource::<f32>(150.0)
            .add_event::<MeshEvent>()
            .add_event::<NotifyMeshEvent>()
            .add_stage(MeshStage::DelaunayMeshUpdate, SystemStage::parallel())
            .add_stage_after(
                MeshStage::DelaunayMeshUpdate,
                MeshStage::DelaunayMeshRead,
                SystemStage::parallel(),
            )
            .add_startup_system(insert_initial_mesh_into_world)
            // .add_system(insert_node.label("insert node"))
            .add_system_to_stage(MeshStage::DelaunayMeshUpdate, swap_mesh_dedges)
            .add_system_to_stage(MeshStage::DelaunayMeshUpdate, update_delaunay_spread)
            .add_system_to_stage(MeshStage::DelaunayMeshRead, update_mesh_positions.label("mesh positions"))
            .add_system_to_stage(MeshStage::DelaunayMeshRead, handle_notify_mesh_events);
    }
}

/// Component form of PrimalDEdgeEntity. Probably a better way to do this.
#[derive(Component, Clone, Copy, PartialEq)]
pub struct PDEdgeEntity(pub usize);
impl From<PrimalDEdgeEntity> for PDEdgeEntity {
    fn from(e: PrimalDEdgeEntity) -> Self {
        Self(e.0)
    }
}
impl From<PDEdgeEntity> for PrimalDEdgeEntity {
    fn from(e: PDEdgeEntity) -> Self {
        Self(e.0)
    }
}
pub enum MeshEvent {
    Swap(PDEdgeEntity),
    // Insert(Vec2),
}

fn swap_mesh_dedges(mut mesh_events: EventReader<MeshEvent>, mesh: NonSend<DelaunayMesh>) {
    for mesh_event in mesh_events.iter() {
        match mesh_event {
            MeshEvent::Swap(e) => mesh.swap_primal((*e).into()),
            _ => (),
        }
    }
}

// fn insert_node(
//     mut mesh_events: EventReader<MeshEvent>,
//     mut mesh: NonSendMut<DelaunayMesh>,
//     mut selected_dedge: ResMut<SelectedDedge>,
// ) {
//     for mesh_event in mesh_events.iter() {
//         match mesh_event {
//             MeshEvent::Insert(pos) => {
//                 selected_dedge.0 = Some(mesh.locate_point((pos.x, pos.y).into())).map(|e| e.into());
//             }
//             _ => (),
//         }
//     }
// }

fn set_mesh_vertex_spread(mesh: &mut DelaunayMesh, x: f32) {
    let mut v1 = mesh.get_vertex(VertexEntity(2)).borrow_mut();
    let mut v2 = mesh.get_vertex(VertexEntity(3)).borrow_mut();
    v1.x = x;
    v2.x = -x;
}

use super::default_arrows::{self, PulsingArrowFrame, RedArrowFrame, WhiteArrowFrame, DefaultArrowsParam};

fn insert_initial_mesh_into_world(
    mut commands: Commands,
    mesh: NonSend<DelaunayMesh>,
    red_arrow_frame: Query<
        Entity,
        (
            With<default_arrows::RedArrowFrame>,
            Without<default_arrows::PulsingArrowFrame>,
        ),
    >,
) {
    let red_arrow_frame = red_arrow_frame.single();
    // let white_arrow_frame = white_arrow_frame.single();

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

            // let arrow_frame = if !mesh.is_delaunay(ent) {
            //     red_arrow_frame
            // } else {
            //     white_arrow_frame
            // };

            commands
                .spawn()
                .insert(bevy_arrow::Arrow {
                    tail: Vec3::new(origin.x as f32, origin.y as f32, 0.0),
                    head: Vec3::new(dest.x as f32, dest.y as f32, 0.0),
                    arrow_frame: red_arrow_frame,
                    width: 16.0,
                })
                .insert(PDEdgeEntity::from(ent));
        }
    }
}

pub enum NotifyMeshEvent {
    DEdgeInserted(PDEdgeEntity),
    // EdgeRemoved(PDEdgeEntity),
}

fn handle_notify_mesh_events(
    mut commands: Commands,
    mesh: NonSend<DelaunayMesh>,
    red_arrow_frame: Query<
        Entity,
        (
            With<default_arrows::RedArrowFrame>,
            Without<default_arrows::PulsingArrowFrame>,
        ),
    >,
    mut notify_mesh_events: EventReader<NotifyMeshEvent>
) {
    for event in notify_mesh_events.iter() {
        match *event {
            NotifyMeshEvent::DEdgeInserted(pde) => {
                let cursor = mesh.primal(pde.into());
                let origin = cursor.org().borrow();
                let dest = cursor.dest().borrow();
                commands.spawn().insert(bevy_arrow::Arrow {
                    tail: Vec3::new(origin.x as f32, origin.y as f32, 0.0),
                    head: Vec3::new(dest.x as f32, dest.y as f32, 0.0),
                    arrow_frame: red_arrow_frame.single(),
                    width: 16.0,
                }).insert(pde);
            }
            // NotifyMeshEvent::EdgeRemoved(_) => todo!(),
        }
    }
}

fn update_delaunay_spread(mut mesh: NonSendMut<DelaunayMesh>, spread: Res<f32>) {
    // set_mesh_vertex_spread(&mut *mesh, *spread)
}

fn update_mesh_positions(
    mesh: NonSend<DelaunayMesh>,
    mut query: Query<(&mut bevy_arrow::Arrow, &PDEdgeEntity)>,
) {
    for (mut arrow, ent) in query.iter_mut() {
        let ent = PrimalDEdgeEntity::from(*ent);
        let dedge = mesh.get_primal(ent);

        let origin = dedge.borrow().org;
        let origin = mesh.get_vertex(origin).borrow().clone();

        let dest = mesh.get_primal(ent.sym()).borrow().org;
        let dest = mesh.get_vertex(dest).borrow().clone();

        arrow.tail.x = origin.x as f32;
        arrow.tail.y = origin.y as f32;
        arrow.head.x = dest.x as f32;
        arrow.head.y = dest.y as f32;
    }
}

// // swap the
// fn update_mesh_is_delaunay(
//     // arrow_frames: Query<&bevy_arrow::ArrowFrame>,
//     mesh: NonSend<DelaunayMesh>,
//     mut query: Query<(&mut bevy_arrow::Arrow, &PDEdgeEntity)>,
//     arrow_frames: DefaultArrowsParam,
// ) {
//     // This is should be rethought out. This isn't good.
//     let white = arrow_frames.white.single();
//     let red = arrow_frames.red.single();
//     let pulsing_white = arrow_frames.pulsing_white.single();
//     let pulsing_red = arrow_frames.pulsing_red.single();

//     for (mut arrow, ent) in query.iter_mut() {
//         let x = arrow.arrow_frame;
//         let is_delaunay = mesh.is_delaunay((*ent).into());
//         arrow.arrow_frame = if x == pulsing_white || x == pulsing_red {
//             if is_delaunay {
//                 pulsing_red
//             } else {
//                 pulsing_white
//             }
//         } else {
//             if is_delaunay {
//                 red
//             } else {
//                 white
//             }
//         };
//     }
// }
