use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::HashMap;
use bevy_arrow::ArrowFrame;
use quad_edge::delaunay_voronoi::DelaunayMesh;

use super::default_arrows::DefaultArrowsParam;
use super::mesh_draw::{MeshStage, PDEdgeEntity};
use super::{default_arrows, shapes};

mod algorithm_animate;

#[derive(Debug, Clone, Eq, PartialEq, Hash, SystemLabel)]
pub enum AnimationState {
    Stopped,
    LocatePoint,
    InsertInterior,
    InsertExterior,
}

pub struct AnimateMesh;
impl Plugin for AnimateMesh {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveDedge>()
            .insert_resource(HighlightColors(vec![Color::YELLOW, Color::YELLOW_GREEN, Color::ORANGE_RED]))
            // .insert_resource(AnimationStep(Timer::from_seconds(1.0, true)))
            .add_event::<AnimateMeshEvent>()
            .add_state(AnimationState::Stopped)
            .add_startup_system(setup_dedge_highlights)
            .add_startup_system(setup_target_sprite)
            .add_startup_system(setup_text)
            .add_system_to_stage(CoreStage::Update, handle_animation_events)
            .add_system_to_stage(MeshStage::DelaunayMeshRead, update_arrow_frames)
            .add_system_to_stage(
                MeshStage::DelaunayMeshRead,
                update_highlights_to_follow_mesh.after("mesh positions"),
            )
            .add_system_set(
                SystemSet::on_enter(AnimationState::LocatePoint)
                    .with_system(algorithm_animate::setup_animation_locate_point),
            )
            .add_system_set(
                // MeshStage::DelaunayMeshRead,
                SystemSet::on_update(AnimationState::LocatePoint)
                    .with_system(algorithm_animate::update_animation_locate_point),
            )
            .add_system_set(
                SystemSet::on_update(AnimationState::InsertExterior)
                    .with_system(algorithm_animate::update_animation_insert_exterior),
            )
            .add_system_set(SystemSet::on_enter(AnimationState::Stopped).with_system(debug));
    }
}

fn debug(
    mesh: NonSend<DelaunayMesh>,
) {
    for (i, edge) in mesh.primal_dedges.iter().enumerate() {
        println!("{}, {:?}", i, edge);
    }
    for (i, edge) in mesh.dual_dedges.iter().enumerate() {
        println!("{}, {:?}", i, edge);
    }
}


#[derive(Component)]
struct DescriptionText;

fn setup_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut text = Text::with_section(
        "Phase\n",
        TextStyle {
            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
            font_size: 60.0,
            color: Color::BLACK,
        },
        TextAlignment {
            vertical: VerticalAlign::Bottom,
            horizontal: HorizontalAlign::Center,
        },
    );
    text.sections.push(TextSection {
        value: "Last Action".into(),
        style: TextStyle {
            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
            font_size: 30.0,
            color: Color::BLACK,
        },
    });
    commands
        .spawn_bundle(Text2dBundle {
            text,
            transform: Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
            ..Default::default()
        })
        .insert(DescriptionText);
}

struct HighlightColors(Vec<Color>);

#[derive(Component)]
pub struct Highlight(pub Color, pub Option<PDEdgeEntity>);

fn setup_dedge_highlights(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    colors: Res<HighlightColors>,
) {
    let mesh_handle: Mesh2dHandle = meshes
        .add(Mesh::from(shapes::build_rect(
            (1.0, -1.0).into(),
            (2.0, 1.0).into(),
        )))
        .into();

    for color in colors.0.iter() {
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: mesh_handle.clone(),
                material: materials.add(ColorMaterial::from(*color)),
                visibility: Visibility { is_visible: false },
                ..Default::default()
            })
            .insert(Highlight(*color, None));
    }
}

fn update_highlights_to_follow_mesh(
    mut commands: Commands,
    mesh: NonSend<DelaunayMesh>,
    mut query: Query<(&mut Transform, &mut Visibility, &Highlight)>,
) {
    for (mut transform, mut visibility, highlight) in query.iter_mut() {
        if let Some(entity) = highlight.1 {
            let dedge = mesh.primal(entity.into());
            let org = dedge.org().borrow();
            let org = Vec2::new(org.x, org.y);
            let dest = dedge.dest().borrow();
            let dest = Vec2::new(dest.x, dest.y);
            let r = dest - org;

            let angle = -r.angle_between(Vec2::X);
            let midpoint = org + r * 0.5;

            *transform = Transform {
                translation: (org, 0.0).into(),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::new(r.length() * 0.5, 8.0, 1.0),
            };
            visibility.is_visible = true;
        } else {
            visibility.is_visible = false;
        }
    }
}

#[derive(Default)]
pub struct ActiveDedge(pub Option<PDEdgeEntity>);

fn update_arrow_frames(
    selected_dedge: Res<ActiveDedge>,
    arrow_frames: default_arrows::DefaultArrowsParam,
    mesh: NonSend<DelaunayMesh>,
    mut query: Query<(&mut bevy_arrow::Arrow, &PDEdgeEntity)>,
) {
    let white = arrow_frames.white.single();
    let red = arrow_frames.red.single();
    let scoop = arrow_frames.scoop.single();
    let pulsing_white = arrow_frames.pulsing_white.single();
    let pulsing_red = arrow_frames.pulsing_red.single();
    let pulsing_scoop = arrow_frames.pulsing_scoop.single();

    let set_pulsing_from_colored = |old_frame: Entity, pulsing: bool| -> Entity {
        if old_frame == red || old_frame == pulsing_red {
            match pulsing {
                true => pulsing_red,
                false => red,
            }
        } else if old_frame == white || old_frame == pulsing_white {
            match pulsing {
                true => pulsing_white,
                false => white,
            }
        } else {
            match pulsing {
                true => pulsing_scoop,
                false => scoop,
            }
        }
    };

    for (mut arrow, dedge) in query.iter_mut() {
        let mut color = if mesh.is_delaunay((*dedge).into()) {
            red
        } else {
            white
        };
        if arrow.arrow_frame == pulsing_scoop || arrow.arrow_frame == scoop {
            color = scoop;
        }
        arrow.arrow_frame = set_pulsing_from_colored(color, selected_dedge.0 == Some(*dedge));
    }
}

#[derive(Component)]
pub struct PointTarget;

fn setup_target_sprite(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("images/node-open.png"),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(PointTarget);
}

pub enum AnimateMeshEvent<'a> {
    SetActiveDedge(Option<&'a str>, Option<PDEdgeEntity>),
    SetHighlightDedge(Option<&'a str>, Color, Option<PDEdgeEntity>),
    SetMarked(Option<&'a str>, PDEdgeEntity, bool),
    SetText(Option<&'a str>, Option<&'a str>),
    SetTargetPosition(Option<&'a str>, Vec2),
    SetTargetVisibility(Option<&'a str>, bool),
    BeginLocateAnimation(Option<&'a str>),
    BeginInsertExteriorAnimation(Option<&'a str>),
}

fn handle_animation_events(
    mut events: EventReader<AnimateMeshEvent<'static>>,
    mut active_dedge: ResMut<ActiveDedge>,
    mut point_target_query: Query<(&mut Transform, &mut Visibility), With<PointTarget>>,
    mut arrows_query: Query<(&mut bevy_arrow::Arrow, &PDEdgeEntity)>,
    mut highlights: Query<&mut Highlight>,
    mut text: Query<&mut Text, With<DescriptionText>>,
    mut animation_state: ResMut<State<AnimationState>>,
    arrow_frames: DefaultArrowsParam,
) {
    let mut desc_text = text.single_mut();
    let (mut phase_text, rest) = desc_text.sections.split_first_mut().unwrap();
    let action_text = rest.get_mut(0).unwrap();

    for event in events.iter() {
        match event {
            AnimateMeshEvent::SetActiveDedge(action, e) => {
                if let Some(action) = *action {
                    action_text.value = action.to_string();
                }
                active_dedge.0 = e.clone();
            }
            AnimateMeshEvent::SetHighlightDedge(action, color, e) => {
                if let Some(action) = *action {
                    action_text.value = action.to_string();
                }
                for mut highlight in highlights.iter_mut() {
                    if *color == highlight.0 {
                        highlight.1 = *e;
                    }
                }
            }
            AnimateMeshEvent::SetText(phase, action) => {
                if let Some(phase) = phase {
                    phase_text.value = format!("{}\n", *phase);
                }
                if let Some(action) = action {
                    action_text.value = action.to_string();
                }
            }
            AnimateMeshEvent::SetTargetPosition(action, position) => {
                if let Some(action) = *action {
                    action_text.value = action.to_string();
                }
                let (mut point_transform, _) = point_target_query.single_mut();
                point_transform.translation = (*position, 0.0).into();

                let (_, mut point_visibility) = point_target_query.single_mut();
                point_visibility.is_visible = true;
            }
            AnimateMeshEvent::SetTargetVisibility(action, is_visible) => {
                if let Some(action) = *action {
                    action_text.value = action.to_string();
                }
                let (_, mut point_visibility) = point_target_query.single_mut();
                point_visibility.is_visible = *is_visible;
            }
            AnimateMeshEvent::BeginLocateAnimation(phase) => {
                if let Some(phase) = phase {
                    phase_text.value = format!("{}\n", phase);
                }
                animation_state.set(AnimationState::LocatePoint).unwrap();
            }
            AnimateMeshEvent::BeginInsertExteriorAnimation(phase) => {
                if let Some(phase) = phase {
                    phase_text.value = format!("{}\n", phase);
                }
                animation_state.set(AnimationState::InsertExterior).unwrap();
            }
            AnimateMeshEvent::SetMarked(action, pde, set) => {
                if let Some(action) = *action {
                    action_text.value = action.to_string();
                }
                for (mut arrow, entity) in arrows_query.iter_mut() {
                    if pde == entity {
                        arrow.arrow_frame = if *set {
                            arrow_frames.scoop.single()
                        } else {
                            arrow_frames.red.single()
                        };
                    }
                }
            }
        }
    }
}
