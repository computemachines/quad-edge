use bevy::prelude::*;

use cgmath::Point2;
use quad_edge::{delaunay_voronoi::DelaunayMesh, geometry::ccw};

use super::{ActiveDedge, AnimateMeshEvent, AnimationState, HighlightedDedge, PointTarget};
use crate::gui::mesh_draw::PDEdgeEntity;

pub fn setup_animation_locate_point(
    mesh: NonSend<DelaunayMesh>,
    // mut highlighted_dedge: ResMut<HighlightedDedge>,
    active_dedge: Res<ActiveDedge>,
    // mut point_target_visibility: Query<&mut Visibility, With<PointTarget>>,
    mut animate_events: EventWriter<AnimateMeshEvent<'static>>,
) {
    // ensure that target point is visible
    animate_events.send(AnimateMeshEvent::SetTargetVisibility(None, true));
    animate_events.send(AnimateMeshEvent::SetHighlightDedge(None, None));

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
        Self(Timer::from_seconds(5.0, true))
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
    mut highlighted_dedge: ResMut<HighlightedDedge>,
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
                animate_events.send(SetText(None, Some("point is right of edge, flipping edge")))
            }
            Action => {
                animate_events.send(SetActiveDedge(
                    Some("point is now left of active edge"),
                    Some(e.sym().id().into()),
                ));
            }
        };
        // e.sym_mut();
        // continue;
    } else if ccw(x, *e.onext().org().borrow(), *e.onext().dest().borrow()) {
        info!("x is left of {:?}", e.onext().id());
        match *indicate_or_action {
            Indicate => animate_events.send(SetHighlightDedge(
                Some("point is left of active edge and left of highlighed edge"),
                Some(e.onext().id().into()),
            )),
            Action => {
                animate_events.send(SetActiveDedge(
                    Some("highlighed edge is now the active edge"),
                    Some(e.onext().id().into()),
                ));
                animate_events.send(SetHighlightDedge(None, None));
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
                    Some("point is left of active edge, e, right of e.Onext and left of highlighted edge"), 
                    Some(e.dprev().id().into())
                ));
            }
            Action => {
                animate_events.send(SetActiveDedge(
                    Some("highlighed edge is now the active edge"),
                    Some(e.dprev().id().into()),
                ));
                animate_events.send(SetHighlightDedge(None, None));
            }
        }
        // leftof x, e.dprev
        // e.dprev_mut();
        // continue;
    } else {
        info!("found face");
        match *indicate_or_action {
            Indicate => {
                animate_events.send(SetHighlightDedge(Some("point is left of active edge, right of e.Onext and right of e.dPrev"), None))
            },
            Action => {
                animate_events.send(SetText(Some("Found Edge"), Some("Point lies in face left of active edge!")));
                animation_state.set(AnimationState::Stopped).unwrap();
            },
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
