use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use quad_edge::delaunay_voronoi::DelaunayMesh;

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
            .init_resource::<HighlightedDedge>()
            // .insert_resource(AnimationStep(Timer::from_seconds(1.0, true)))
            .add_event::<AnimateMeshEvent>()
            .add_state(AnimationState::Stopped)
            .add_startup_system(setup_dedge_highlight)
            .add_startup_system(setup_target_sprite)
            .add_startup_system(setup_text)
            .add_system_to_stage(CoreStage::Update, handle_animation_events)
            .add_system_to_stage(MeshStage::DelaunayMeshRead, update_arrow_frames)
            .add_system_to_stage(
                MeshStage::DelaunayMeshRead,
                update_highlight_to_follow_mesh.after("mesh positions"),
            )
            .add_system_set(
                SystemSet::on_enter(AnimationState::LocatePoint)
                    .with_system(algorithm_animate::setup_animation_locate_point),
            )
            .add_system_set(
                // MeshStage::DelaunayMeshRead,
                SystemSet::on_update(AnimationState::LocatePoint)
                    .with_system(algorithm_animate::update_animation_locate_point),
            );
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

#[derive(Component)]
struct HighlightRect;

#[derive(Default)]
pub struct HighlightedDedge(pub Option<PDEdgeEntity>);

fn setup_dedge_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shapes::build_rect(
                    (1.0, -1.0).into(),
                    (2.0, 1.0).into(),
                )))
                .into(),
            material: materials.add(ColorMaterial::from(Color::YELLOW)),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(HighlightRect);
}

fn update_highlight_to_follow_mesh(
    mesh: NonSend<DelaunayMesh>,
    mut query: Query<(&mut Transform, &mut Visibility), With<HighlightRect>>,
    highlighted_dedge: Res<HighlightedDedge>,
) {
    let (mut transform, mut visibility) = query.single_mut();
    if let Some(entity) = highlighted_dedge.0 {
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
    let pulsing_white = arrow_frames.pulsing_white.single();
    let pulsing_red = arrow_frames.pulsing_red.single();

    let set_pulsing_from_colored = |old_frame: Entity, pulsing: bool| -> Entity {
        if old_frame == red || old_frame == pulsing_red {
            match pulsing {
                true => pulsing_red,
                false => red,
            }
        } else {
            match pulsing {
                true => pulsing_white,
                false => white,
            }
        }
    };

    for (mut arrow, dedge) in query.iter_mut() {
        let color = if mesh.is_delaunay((*dedge).into()) {
            red
        } else {
            white
        };
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
    SetHighlightDedge(Option<&'a str>, Option<PDEdgeEntity>),
    SetText(Option<&'a str>, Option<&'a str>),
    SetTargetPosition(Option<&'a str>, Vec2),
    SetTargetVisibility(Option<&'a str>, bool),
    BeginLocateAnimation(Option<&'a str>),
}

fn handle_animation_events(
    mut events: EventReader<AnimateMeshEvent<'static>>,
    mut active_dedge: ResMut<ActiveDedge>,
    mut highlighted_dedge: ResMut<HighlightedDedge>,
    mut point_target_query: Query<(&mut Transform, &mut Visibility), With<PointTarget>>,
    mut text: Query<&mut Text, With<DescriptionText>>,
    mut animation_state: ResMut<State<AnimationState>>,
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
            AnimateMeshEvent::SetHighlightDedge(action, e) => {
                if let Some(action) = *action {
                    action_text.value = action.to_string();
                }
                highlighted_dedge.0 = e.clone();
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
        }
    }
}
