use bevy::prelude::*;

use cgmath::Point2;
use quad_edge::{delaunay_voronoi::DelaunayMesh, geometry::ccw, mesh::quad::PrimalDEdgeEntity};

use super::{ActiveDedge, AnimateMeshEvent, AnimationState, PointTarget};
use crate::gui::mesh_draw::{NotifyMeshEvent, PDEdgeEntity};

pub fn setup_animation_locate_point(
    mesh: NonSend<DelaunayMesh>,
    // mut highlighted_dedge: ResMut<HighlightedDedge>,
    active_dedge: Res<ActiveDedge>,
    // mut point_target_visibility: Query<&mut Visibility, With<PointTarget>>,
    mut animate_events: EventWriter<AnimateMeshEvent<'static>>,
) {
    // ensure that target point is visible
    animate_events.send(AnimateMeshEvent::SetTargetVisibility(None, true));
    animate_events.send(AnimateMeshEvent::SetHighlightDedge(
        None,
        Color::YELLOW,
        None,
    ));
    animate_events.send(AnimateMeshEvent::SetHighlightDedge(
        None,
        Color::YELLOW_GREEN,
        None,
    ));

    // find an initial edge for edge walk
    if active_dedge.0.is_none() {
        let first_primal = mesh
            .primal_dedges
            .iter()
            .enumerate()
            .find(|(i, e)| e.is_some())
            .map(|(i, _)| PDEdgeEntity(i));
        // if first_primal is none then there are no dedges
        animate_events.send(AnimateMeshEvent::SetActiveDedge(
            Some("arbitrary initial directed edge"),
            first_primal,
        ));
    }
}

pub struct AnimationStep(pub Timer);
impl Default for AnimationStep {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, true))
    }
}

pub enum LocatePointAnimationState {
    Indicate,
    Action,
}
impl Default for LocatePointAnimationState {
    fn default() -> Self {
        LocatePointAnimationState::Indicate
    }
}

pub fn update_animation_locate_point(
    mut step_timer: Local<AnimationStep>,
    time: Res<Time>,
    mesh: NonSend<DelaunayMesh>,
    point_target: Query<&Transform, With<PointTarget>>,
    active_dedge: Res<ActiveDedge>,
    mut animation_state: ResMut<State<AnimationState>>,
    mut animate_events: EventWriter<AnimateMeshEvent<'static>>,
    mut indicate_or_action: Local<LocatePointAnimationState>,
) {
    if !step_timer.0.tick(time.delta()).just_finished() {
        return;
    }
    use AnimateMeshEvent::*;
    use LocatePointAnimationState::*;

    info!("animation step");
    let x: Vec3 = point_target.single().translation;
    let x = Point2::new(x.x, x.y);

    let e = mesh.primal(active_dedge.0.unwrap().into());
    if x == *e.org().borrow() || x == *e.dest().borrow() {
        animation_state.set(AnimationState::Stopped).unwrap();
        animate_events.send(SetText(
            Some("Found Edge"),
            Some("Coincides with existing vertex!"),
        ));
    } else if !ccw(x, *e.org().borrow(), *e.dest().borrow()) {
        // rightof x, e
        info!("x is right of {:?}", e.id());
        match *indicate_or_action {
            Indicate => {
                animate_events.send(SetMarked(
                    Some("point is right of edge, flipping edge"),
                    e.id().into(),
                    true,
                ));
            }
            Action => {
                animate_events.send(SetActiveDedge(
                    Some("e := e.Sym; point is now left of active edge"),
                    Some(e.sym().id().into()),
                ));
            }
        };
        // e.sym_mut();
        // continue;
    } else if e.left().borrow().is_infinite() {
        info!("left of boundary edge");
        dbg!(e.left().borrow());
        match *indicate_or_action {
            Indicate => animate_events.send(SetText(
                None,
                Some("point is left of active edge and left of Onext (yellow)"),
            )),
            Action => {
                animate_events.send(SetText(
                    Some("Found Edge"),
                    Some("Point lies in infinite face left of active edge!"),
                ));
                animate_events.send(BeginInsertExteriorAnimation(Some("Insert Exterior Vertex")));
                // animation_state.set(AnimationState::Stopped).unwrap();
            }
        }
    } else if ccw(x, *e.onext().org().borrow(), *e.onext().dest().borrow()) {
        info!("x is left of {:?}", e.onext().id());
        match *indicate_or_action {
            Indicate => animate_events.send(SetHighlightDedge(
                Some("point is left of active edge and left of Onext (yellow)"),
                Color::YELLOW,
                Some(e.onext().id().into()),
            )),
            Action => {
                animate_events.send(SetActiveDedge(
                    Some("e := e.Onext;"),
                    Some(e.onext().id().into()),
                ));
                animate_events.send(SetHighlightDedge(None, Color::YELLOW, None));
            }
        }
        // leftof x, e.onext
        // animate_events.send(SetHighlightDedge((), ()))
        // e.onext_mut();
        // continue;
    } else if ccw(x, *e.dprev().org().borrow(), *e.dprev().dest().borrow()) {
        info!("x is left of {:?}", e.dprev().id());
        match *indicate_or_action {
            Indicate => {
                animate_events.send(SetHighlightDedge(
                    None,
                    Color::YELLOW,
                    Some(e.onext().id().into()),
                ));
                animate_events.send(SetHighlightDedge(
                    Some(
                        "point is left of e, right of e.Onext (yellow) and left of e.Dprev (green)",
                    ),
                    Color::YELLOW_GREEN,
                    Some(e.dprev().id().into()),
                ));
            }
            Action => {
                animate_events.send(SetActiveDedge(
                    Some("highlighed edge is now the active edge"),
                    Some(e.dprev().id().into()),
                ));
                animate_events.send(SetHighlightDedge(None, Color::YELLOW, None));
            }
        }
        // leftof x, e.dprev
        // e.dprev_mut();
        // continue;
    } else {
        info!("found face");
        match *indicate_or_action {
            Indicate => {
                animate_events.send(SetHighlightDedge(
                    None,
                    Color::YELLOW,
                    Some(e.onext().id().into()),
                ));
                animate_events.send(SetHighlightDedge(
                Some("point is left of e, right of e.Onext (yellow) and right of e.Dprev (green)"),
                Color::YELLOW_GREEN,
                Some(e.dprev().id().into()),
            ));
            }
            Action => {
                animate_events.send(SetText(
                    Some("Found Edge"),
                    Some("Point lies in face left of active edge!"),
                ));
                animation_state.set(AnimationState::Stopped).unwrap();
            }
        }
        // break e.id();
    }

    // toggle indicate/action phase of animation step
    *indicate_or_action = match *indicate_or_action {
        Indicate => Action,
        Action => Indicate,
    };

    info!("stuff");
}

pub enum InsertExteriorState {
    FindFanStart,
    MakeDanglingEdge,
    CompleteFan(PrimalDEdgeEntity),
}
impl Default for InsertExteriorState {
    fn default() -> Self {
        InsertExteriorState::FindFanStart
    }
}

pub fn update_animation_insert_exterior(
    mut state: Local<InsertExteriorState>,
    mut step_timer: Local<AnimationStep>,
    time: Res<Time>,
    mut mesh: NonSendMut<DelaunayMesh>,
    point_target: Query<&Transform, With<PointTarget>>,
    active_dedge: Res<ActiveDedge>,
    mut highlights: Query<&mut super::Highlight>,
    mut animate_events: EventWriter<AnimateMeshEvent<'static>>,
    mut mesh_events: EventWriter<NotifyMeshEvent>,
    mut animation_state: ResMut<State<AnimationState>>,
) {
    if !step_timer.0.tick(time.delta()).just_finished() {
        return;
    }
    use AnimateMeshEvent::*;

    info!("animation step");
    let x: Vec3 = point_target.single().translation;
    let x = Point2::new(x.x, x.y);

    let mut e = mesh.primal(active_dedge.0.unwrap().into());
    *state = match *state {
        InsertExteriorState::FindFanStart => {
            // state machine form of:
            // while ccw(x, *e.org().borrow(), *e.dest().borrow()) {
            //     e.lnext_mut();
            // }
            if ccw(x, *e.org().borrow(), *e.dest().borrow()) {
                animate_events.send(SetActiveDedge(
                    Some("e := e.Lnext; "),
                    Some(e.lnext().id().into()),
                ));

                InsertExteriorState::FindFanStart
            } else {
                animate_events.send(SetMarked(Some("Found Fan Start"), e.id().into(), true));
                InsertExteriorState::MakeDanglingEdge
            }
        }
        InsertExteriorState::MakeDanglingEdge => {
            info!("make dangling edge");

            let v = mesh.insert_vertex(x);
            let edge =
                mesh.connect_vertex(PrimalDEdgeEntity::from(active_dedge.0.unwrap()).sym(), v);

            animate_events.send(SetHighlightDedge(
                Some("Make dangling edge at e.sym"),
                Color::ORANGE_RED,
                Some(edge.sym().into()),
            ));
            animate_events.send(SetActiveDedge(
                None,
                Some(mesh.primal(edge).sym().rprev().id().into()),
            ));
            mesh_events.send(NotifyMeshEvent::DEdgeInserted(edge.into()));
            mesh_events.send(NotifyMeshEvent::DEdgeInserted(edge.sym().into()));

            InsertExteriorState::CompleteFan(edge.sym())
        }
        InsertExteriorState::CompleteFan(fan_start) => {
            if !ccw(x, *e.org().borrow(), *e.dest().borrow()) {
                info!("x rightof e");
                let e_rnext_id = e.rnext().id();
                let e_id = e.id();
                let e_rprev_id = e.rprev().id();
                let new_face =
                    mesh.insert_face(quad_edge::delaunay_voronoi::VoronoiVertex::Finite(0.0, 0.0));

                let new_edge = mesh.connect_primal(e_rprev_id.sym(), fan_start);
                mesh.get_dual(e_id.rot()).borrow_mut().org = new_face; // set e.right
                mesh.get_dual(e_rnext_id.rot()).borrow_mut().org = new_face; // set e.rnext.right
                mesh.get_dual(new_edge.rot()).borrow_mut().org = new_face; // set new_edge.right

                mesh_events.send(NotifyMeshEvent::DEdgeInserted(new_edge.into()));
                mesh_events.send(NotifyMeshEvent::DEdgeInserted(new_edge.sym().into()));
                animate_events.send(SetActiveDedge(
                    Some("Create new fanned edge"),
                    Some(e_rprev_id.into()),
                ));
                animate_events.send(SetHighlightDedge(
                    None,
                    Color::YELLOW,
                    Some(e_rprev_id.sym().into()),
                ));
                animate_events.send(SetHighlightDedge(
                    None,
                    Color::YELLOW_GREEN,
                    Some(fan_start.into()),
                ));
                // animation_state.set(AnimationState::Stopped).unwrap();

                InsertExteriorState::CompleteFan(fan_start)
            } else {
                animate_events.send(SetText(
                    Some("Inserted Exterior Node"),
                    Some("Graph should be convex"),
                ));
                animation_state.set(AnimationState::Stopped).unwrap();
                InsertExteriorState::FindFanStart // reset state
            }
        }
    }
}
