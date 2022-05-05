use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use quad_edge::delaunay_voronoi::DelaunayMesh;
use quad_edge::mesh::quad::{PrimalDEdgeEntity, VertexEntity};

pub struct MeshDraw;
impl Plugin for MeshDraw {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedDedge>()        
        .insert_resource::<f32>(100.0)
        .add_event::<MeshEvent>()
        .add_startup_system(insert_initial_mesh)
        .add_startup_system(init_circles)
        .add_system(swap_mesh_dedges)
        .add_system(update_delaunay_spread)
        .add_system(update_mesh_positions.label("mesh position"))
        .add_system(update_mesh_selected.after("mesh position"))
        .add_system(debug_in_circle_test);
    }
}


#[derive(Component, Clone, Copy, PartialEq)]
pub struct PDEdgeEntity( pub usize);
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

#[derive(Default)]
pub struct SelectedDedge(pub Option<PDEdgeEntity>);


pub enum MeshEvent {
    Swap(PDEdgeEntity),
}

fn swap_mesh_dedges(mut mesh_events: EventReader<MeshEvent>, mesh: NonSend<DelaunayMesh>) {
    for mesh_event in mesh_events.iter() {
        match mesh_event {
            MeshEvent::Swap(e) => mesh.swap_primal((*e).into())
        }
    }
}

fn set_mesh_vertex_spread(mesh: &mut DelaunayMesh, x: f32) {
    let mut v1 = mesh.get_vertex(VertexEntity(2)).borrow_mut();
    let mut v2 = mesh.get_vertex(VertexEntity(3)).borrow_mut();
    v1.x = x as f64;
    v2.x = -x as f64;
}

#[derive(Component)]
struct A;

#[derive(Component)]
struct X;

#[derive(Component)]
struct Y;

#[derive(Component)]
struct B;

fn init_circles(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mesh = super::shapes::build_circle(2); //Mesh::from(Quad::default());
    let mesh_handle = Mesh2dHandle(meshes.add(mesh));

    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            material: materials.add(ColorMaterial::from(Color::Rgba {
                red: 0.0,
                green: 0.0,
                blue: 1.0,
                alpha: 0.5,
            })),
            visibility: Visibility { is_visible: true },
            ..Default::default()
        })
        .insert(A);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            material: materials.add(ColorMaterial::from(Color::Rgba {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.5,
            })),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(X);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            material: materials.add(ColorMaterial::from(Color::Rgba {
                red: 1.0,
                green: 1.0,
                blue: 0.0,
                alpha: 0.5,
            })),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Y);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            material: materials.add(ColorMaterial::from(Color::Rgba {
                red: 0.0,
                green: 1.0,
                blue: 0.0,
                alpha: 0.5,
            })),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(B);
}

fn debug_in_circle_test(
    mesh: NonSend<DelaunayMesh>,
    selected_dedge: Res<SelectedDedge>,
    mut visibility: Query<&mut Visibility>,
    mut transform: Query<&mut Transform>,
    entity_a: Query<Entity, With<A>>,
    entity_x: Query<Entity, With<X>>,
    entity_y: Query<Entity, With<Y>>,
    entity_b: Query<Entity, With<B>>,
) {
    if selected_dedge.0.is_none() {
        return;
    }

    let xy = selected_dedge.0.unwrap().into();

    let xy = mesh.primal(xy);
    let a = xy.onext().dest().borrow().clone();
    let x = xy.org().borrow().clone();
    let y = xy.dest().borrow().clone();
    let b = xy.oprev().dest().borrow().clone();

    *transform
        .get_component_mut::<Transform>(entity_a.single())
        .unwrap() = Transform {
        translation: ((a.x as f32, a.y as f32, 0.0).into()),
        rotation: Default::default(),
        scale: Vec3::splat(20.0),
    };
    visibility
        .get_component_mut::<Visibility>(entity_a.single())
        .unwrap()
        .is_visible = true;

    *transform
        .get_component_mut::<Transform>(entity_x.single())
        .unwrap() = Transform {
        translation: ((x.x as f32, x.y as f32, 0.0).into()),
        rotation: Default::default(),
        scale: Vec3::splat(20.0),
    };
    visibility
        .get_component_mut::<Visibility>(entity_x.single())
        .unwrap()
        .is_visible = true;

    *transform
        .get_component_mut::<Transform>(entity_y.single())
        .unwrap() = Transform {
        translation: ((y.x as f32, y.y as f32, 0.0).into()),
        rotation: Default::default(),
        scale: Vec3::splat(20.0),
    };
    visibility
        .get_component_mut::<Visibility>(entity_y.single())
        .unwrap()
        .is_visible = true;

    *transform
        .get_component_mut::<Transform>(entity_b.single())
        .unwrap() = Transform {
        translation: ((b.x as f32, b.y as f32, 0.0).into()),
        rotation: Default::default(),
        scale: Vec3::splat(20.0),
    };
    visibility
        .get_component_mut::<Visibility>(entity_b.single())
        .unwrap()
        .is_visible = true;
}

use super::default_arrows;

fn insert_initial_mesh(
    mut commands: Commands,
    mesh: NonSend<DelaunayMesh>,
    red_arrow_frame: Query<Entity, (With<default_arrows::RedArrowFrame>, Without<default_arrows::PulsingArrowFrame>)>,
    white_arrow_frame: Query<Entity, (With<default_arrows::WhiteArrowFrame>, Without<default_arrows::PulsingArrowFrame>)>,
) {
    let red_arrow_frame = red_arrow_frame.single();
    let white_arrow_frame = white_arrow_frame.single();

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

            let arrow_frame = if !mesh.is_delaunay(ent) {
                red_arrow_frame
            } else {
                white_arrow_frame
            };

            commands
                .spawn()
                .insert(bevy_arrow::Arrow {
                    tail: Vec3::new(origin.x as f32, origin.y as f32, 0.0),
                    head: Vec3::new(dest.x as f32, dest.y as f32, 0.0),
                    arrow_frame,
                    width: 16.0,
                })
                .insert(PDEdgeEntity::from(ent));
        }
    }
}

fn update_delaunay_spread(mut mesh: NonSendMut<DelaunayMesh>, spread: Res<f32>) {
    set_mesh_vertex_spread(&mut *mesh, *spread)
}

fn update_mesh_positions(
    mesh: NonSend<DelaunayMesh>,
    mut query: Query<(&mut bevy_arrow::Arrow, &PDEdgeEntity)>,

    red_arrow_frame: Query<Entity, (With<default_arrows::RedArrowFrame>, Without<default_arrows::PulsingArrowFrame>)>,
    white_arrow_frame: Query<Entity, (With<default_arrows::WhiteArrowFrame>, Without<default_arrows::PulsingArrowFrame>)>,
) {
    let red_arrow_frame = red_arrow_frame.single();
    let white_arrow_frame = white_arrow_frame.single();

    for (mut arrow, ent) in query.iter_mut() {
        let ent = PrimalDEdgeEntity::from(*ent);
        let dedge = mesh.get_primal(ent);

        let origin = dedge.borrow().org;
        let origin = mesh.get_vertex(origin).borrow().clone();

        let dest = mesh.get_primal(ent.sym()).borrow().org;
        let dest = mesh.get_vertex(dest).borrow().clone();

        let arrow_frame = if !mesh.is_delaunay(ent) {
            red_arrow_frame
        } else {
            white_arrow_frame
        };

        arrow.tail.x = origin.x as f32;
        arrow.tail.y = origin.y as f32;
        arrow.head.x = dest.x as f32;
        arrow.head.y = dest.y as f32;
        arrow.arrow_frame = arrow_frame;
    }
}

// fn update_mesh_is_delaunay() {}

fn update_mesh_selected(
    selected_dedge: Res<SelectedDedge>,
    red_arrow_frame: Query<Entity, (With<default_arrows::RedArrowFrame>, Without<default_arrows::PulsingArrowFrame>)>,
    white_arrow_frame: Query<Entity, (With<default_arrows::WhiteArrowFrame>, Without<default_arrows::PulsingArrowFrame>)>,
    pulsing_red_arrow_frame: Query<Entity, (With<default_arrows::RedArrowFrame>, With<default_arrows::PulsingArrowFrame>)>,
    pulsing_white_arrow_frame: Query<Entity, (With<default_arrows::WhiteArrowFrame>, With<default_arrows::PulsingArrowFrame>)>,
    mut query: Query<(&mut bevy_arrow::Arrow, &PDEdgeEntity)>,
) {
    let red_arrow_frame = red_arrow_frame.single();
    let white_arrow_frame = white_arrow_frame.single();
    let pulsing_red_arrow_frame = pulsing_red_arrow_frame.single();
    let pulsing_white_arrow_frame = pulsing_white_arrow_frame.single();

    let set_pulsing_from = |old_frame: Entity, pulsing: bool| -> Entity {
        if old_frame == red_arrow_frame || old_frame == pulsing_red_arrow_frame {
            match pulsing {
                true => pulsing_red_arrow_frame,
                false => red_arrow_frame,
            }
        } else {
            match pulsing {
                true => pulsing_white_arrow_frame,
                false => white_arrow_frame,
            }
        }
    };

    for (mut arrow, dedge) in query.iter_mut() {
        arrow.arrow_frame = set_pulsing_from(arrow.arrow_frame, selected_dedge.0 == Some(*dedge));
    }
}