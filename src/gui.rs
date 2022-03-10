use bevy::{
    ecs::system::lifetimeless::SRes,
    pbr::MaterialPipeline,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAsset,
        render_resource::{
            std140::{AsStd140, Std140},
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferInitDescriptor,
            BufferSize, BufferUsages, ShaderStages,
        },
        renderer::RenderDevice,
    },
};
use quad_edge::delaunay_voronoi::DelaunayMesh;

#[derive(Component)]
struct MovingLight;

pub fn explore_mesh(mesh: DelaunayMesh) {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        // .insert_resource(Msaa { samples: 1 })
        .add_startup_system(setup_system)
        .add_system(animate_light)
        .run();
}

pub fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..Default::default()
        }),
        ..Default::default()
    });

    // left wall
    let mut transform = Transform::from_xyz(2.5, 2.5, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    commands.spawn_bundle(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
        transform,
        material: custom_materials.add(CustomMaterial {
            color: Color::GOLD,
        }),
        ..Default::default()
    });
    // back (right) wall
    let mut transform = Transform::from_xyz(0.0, 2.5, -2.5);
    transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
        transform,
        material: materials.add(StandardMaterial {
            base_color: Color::INDIGO,
            perceptual_roughness: 1.0,
            ..Default::default()
        }),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule {
            depth: 1.0,
            radius: 0.25,
            ..Default::default()
        })),
        material: materials.add(Color::rgb(0.7, 0.3, 0.2).into()),
        ..Default::default()
    });
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 500.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(1.0, 2.0, 0.0),
            ..Default::default()
        })
        .insert(MovingLight);

    commands.insert_resource(AmbientLight {
        color: Color::GREEN,
        brightness: 0.02,
    });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    //     commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    //     let x = commands
    //         .spawn_bundle(SpriteBundle {
    //             sprite: Sprite {
    //                 color: Color::rgb(0.25, 0.5, 0.75),
    //                 custom_size: Some(Vec2::new(50.0, 50.0)),
    //                 ..Default::default()
    //             },
    //             transform: Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
    //             ..Default::default()
    //         });
}

fn animate_light(time: Res<Time>, mut query: Query<&mut Transform, With<MovingLight>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x = (time.seconds_since_startup().sin() * 5.0 - 5.5) as f32;
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "c6ed61a8-d517-4c20-b735-2c62adb4b8f8"]
pub struct CustomMaterial {
    color: Color,
}

#[derive(Clone)]
pub struct GpuCustomMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

impl RenderAsset for CustomMaterial {
    type ExtractedAsset = CustomMaterial;

    type PreparedAsset = GpuCustomMaterial;

    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<CustomMaterial>>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        let color = Vec4::from_slice(&extracted_asset.color.as_linear_rgba_f32());
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: color.as_std140().as_bytes(),
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &material_pipeline.material_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Ok(GpuCustomMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl Material for CustomMaterial {
    fn bind_group(material: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &material.bind_group
    }

    fn bind_group_layout(
        render_device: &RenderDevice,
    ) -> bevy::render::render_resource::BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(Vec4::std140_size_static() as u64),
                },
                count: None,
            }],
        })
    }

    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        None
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/custom_material.wgsl"))
    }

    fn alpha_mode(material: &<Self as RenderAsset>::PreparedAsset) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn dynamic_uniform_indices(material: &<Self as RenderAsset>::PreparedAsset) -> &[u32] {
        &[]
    }
}
