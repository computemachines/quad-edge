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
            .add_system_to_stage(
                MeshStage::DelaunayMeshRead,
                update_mesh_positions.label("mesh positions"),
            )
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

use super::default_arrows::{self, DefaultArrowsParam};

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
    mut notify_mesh_events: EventReader<NotifyMeshEvent>,
) {
    for event in notify_mesh_events.iter() {
        match *event {
            NotifyMeshEvent::DEdgeInserted(pde) => {
                let cursor = mesh.primal(pde.into());
                let origin = cursor.org().borrow();
                let dest = cursor.dest().borrow();
                commands
                    .spawn()
                    .insert(bevy_arrow::Arrow {
                        tail: Vec3::new(origin.x as f32, origin.y as f32, 0.0),
                        head: Vec3::new(dest.x as f32, dest.y as f32, 0.0),
                        arrow_frame: red_arrow_frame.single(),
                        width: 16.0,
                    })
                    .insert(pde);
            } // NotifyMeshEvent::EdgeRemoved(_) => todo!(),
        }
    }
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
