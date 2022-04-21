use bevy::{
    asset::HandleId,
    core::{FloatOrd, Pod, Zeroable},
    core_pipeline::Transparent2d,
    ecs::system::lifetimeless::{Read, SQuery, SRes},
    prelude::*,
    reflect::{TypeUuid, Uuid},
    render::{
        mesh::{GpuBufferInfo, Indices},
        render_asset::{RenderAsset, RenderAssets},
        render_component::{ExtractComponent, ExtractComponentPlugin},
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer,
            BufferBindingType, BufferInitDescriptor, BufferUsages, ColorTargetState, ColorWrites,
            FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineCache, RenderPipelineDescriptor, SamplerBindingType,
            ShaderStages, SpecializedPipeline, SpecializedPipelines, TextureFormat,
            TextureSampleType, TextureViewDimension, VertexAttribute, VertexBufferLayout,
            VertexFormat, VertexState, VertexStepMode, BindingResource,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::VisibleEntities,
        RenderApp, RenderStage,
    },
    sprite::{
        Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
    utils::HashMap,
};

// Randomly generated UUID
pub const TWO_TRANSFORM_INTER_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x97391c0926e9d409);

pub struct ArrowPlugin;
impl Plugin for ArrowPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            TWO_TRANSFORM_INTER_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../../assets/shaders/two_interp.wgsl")),
        );

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app.add_render_command::<Transparent2d, (
            SetItemPipeline,
            SetMesh2dViewBindGroup<0>,
            SetMesh2dBindGroup<1>,
            SetArrowTextureBindGroup,
            DrawArrowInstanced,
        )>();

        render_app
            .init_resource::<MyImageBindGroups>()
            .init_resource::<ArrowInstancePipeline>()
            .init_resource::<SpecializedPipelines<ArrowInstancePipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_arrow_instances)
            .add_system_to_stage(RenderStage::Queue, queue_arrow_instances)
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers);
    }
}

// name for vertex attribute. Bevy puts attributes in alphabetical order.
pub const ATTRIBUTE_WEIGHT: &'static str = "Vertex_Weight";

// Marker component for `ArrowsBundle`.
#[derive(Component, Default)]
pub struct ArrowFrame;

// Set of components that represent a 'frame' or 'stage' that all arrows of the same shape are drawn on.
// Moving the `ArrowsBundle` moves all the `Arrow`s that are linked to it as if they were in a Parent-Child relationship.
// So far this is the only way to parent an `Arrow`.
#[derive(Bundle, Default)]
pub struct ArrowsBundle {
    pub mesh: Mesh2dHandle,
    pub texture: Handle<Image>,

    pub local: Transform,
    pub global: GlobalTransform,
    pub visible: Visibility,
    pub computed_visibility: ComputedVisibility,

    // Marker Component
    pub arrow_frame_marker: ArrowFrame,
}

// A Render `World` component.
// First component is used to build the instance buffers in the `RenderStage::Prepare` phase.
// Second component holds the extracted texture.
#[derive(Component, Default, Debug, Clone)]
pub struct ExtractedArrowInstances(pub Vec<ExtractedArrowInstance>);

// Array of `ExtractedArrowInstances` are passed to GPU as an Instance Buffer
// TODO: reduce the size of this. Instances do not need the full transform matrix.
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct ExtractedArrowInstance {
    pub tail_global_transform: Mat4,
    pub head_global_transform: Mat4,
}

// Represents an arrow with tail transform, head transform, and `ArrowsFrame` entity.
// Example usage:
// fn add_arrow(mut commands: Commands, arrow_frame: Query<Entity, With<ArrowFrame>>) {
//     commands.spawn().insert(Arrow(
//        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
//        arrow_frame.single(),
//    ));
// }
#[derive(Component)]
pub struct Arrow(pub Transform, pub Transform, pub Entity);

#[derive(Component, Clone, Copy)]
struct ExtractedArrowTexture {
    image_handle_id: HandleId, // copied from bevy_sprite `ExtractedSprite`
}

// Extract user `Arrow` components into rendering `World`.
// Each 'arrow type' is represented by a different `ArrowsFrame` entity.
fn extract_arrow_instances(
    mut commands: Commands,
    arrows: Query<&Arrow>,
    image_handles: Query<&Handle<Image>>,
    global_transforms: Query<&GlobalTransform>,
) {
    let mut arrows_by_type = HashMap::default();

    for Arrow(tail, head, entity) in arrows.iter() {
        let mut arrows = arrows_by_type.entry(*entity).or_insert(Vec::new());

        if let Ok(transform) = global_transforms.get(*entity) {
            arrows.push(ExtractedArrowInstance {
                head_global_transform: transform.mul_transform(*head).compute_matrix(),
                tail_global_transform: transform.mul_transform(*tail).compute_matrix(),
            });
        }
    }

    // insert the collected arrow instances onto the ArrowFrame and mark as ready to be queued.
    for (arrow_type, arrows) in arrows_by_type.drain() {
        if let Ok(image_handle) = image_handles.get(arrow_type) {
            info!("inserting extracted arrow into render world");
            commands
                .get_or_spawn(arrow_type)
                .insert(ExtractedArrowInstances(arrows))
                .insert(ExtractedArrowTexture {
                    image_handle_id: image_handle.id,
                })
                .insert(QueueArrowInstanced);
        } else {
            warn!(
                "arrow_type: {:?}, did not have an Handle<Image>",
                arrow_type
            );
        }
    }
}

#[derive(Component)]
struct QueueArrowInstanced;

#[derive(Default)]
pub struct MyImageBindGroups {
    pub values: HashMap<Handle<Image>, BindGroup>,
}

// Queue up the custom EntityRenderCommand useing the specialized pipeline.
fn queue_arrow_instances(
    mut render_device: ResMut<RenderDevice>,
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    arrow_instance_pipeline: Res<ArrowInstancePipeline>,
    mut pipelines: ResMut<SpecializedPipelines<ArrowInstancePipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    msaa: Res<Msaa>,

    arrow_instances: Query<
        (&Mesh2dHandle, &Mesh2dUniform, &ExtractedArrowTexture),
        With<QueueArrowInstanced>,
    >,
    gpu_images: Res<RenderAssets<Image>>,
    mut image_bind_groups: ResMut<MyImageBindGroups>,

    render_meshes: Res<RenderAssets<Mesh>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if arrow_instances.is_empty() {
        return;
    }
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        info!("queue view");
        let draw_arrow_instanced = transparent_draw_functions
            .read()
            .get_id::<(
                SetItemPipeline,
                SetMesh2dViewBindGroup<0>,
                SetMesh2dBindGroup<1>,
                SetArrowTextureBindGroup,
                DrawArrowInstanced,
            )>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            info!("queue visible entity");

            if let Ok((mesh2d_handle, mesh2d_uniform, extracted_arrow_texture)) =
                arrow_instances.get(*visible_entity)
            {
                info!("queue visible arrow_instance: {:?}", visible_entity);

                if let Some(gpu_image) =
                    gpu_images.get(&Handle::weak(extracted_arrow_texture.image_handle_id))
                {
                    image_bind_groups
                        .values
                        .entry(Handle::weak(extracted_arrow_texture.image_handle_id))
                        .or_insert_with(|| {
                            render_device.create_bind_group(&BindGroupDescriptor {
                                label: Some("arrow texture bind group"),
                                layout: &arrow_instance_pipeline.texture_layout,
                                entries: &[
                                    BindGroupEntry {
                                        binding: 0,
                                        resource: BindingResource::TextureView(&gpu_image.texture_view),
                                    },
                                    BindGroupEntry {
                                        binding: 1,
                                        resource: BindingResource::Sampler(&gpu_image.sampler),
                                    },
                                ],
                            })
                        });
                    info!("gpu image texture is loaded");
                } else {
                    warn!("gpu image texture not loaded");
                }

                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &arrow_instance_pipeline, mesh2d_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_arrow_instanced,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: None,
                });
            } else {
                // info!("entity is not arrow_instances");
            }
        }
    }
}

// The render world component that holds GPU buffer of `ArrowInstances` data.
#[derive(Component, Debug)]
struct InstanceBuffer {
    // GPU buffer with data copied from `ArrowInstances`.
    buffer: Buffer,
    length: u32,
}

// Create the instance buffer on the GPU with copied data.
// Insert the `InstanceBuffer` into the source `ArrowInstances` entity.
// Bevy calls this before the EntityRenderCommands, `render`.
fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &ExtractedArrowInstances)>,
    render_device: Res<RenderDevice>,
) {
    // info!("Running prepare instance buffer system");
    for (entity, instance_data) in query.iter() {
        // info!("prepare instance buffer data");
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Instance buffer data"),
            contents: bytemuck::cast_slice(instance_data.0.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.0.len() as u32,
        });
        // info!("Added instance buffer to entity: {:?}", entity);
    }
}


struct SetArrowTextureBindGroup;
impl EntityRenderCommand for SetArrowTextureBindGroup {
    type Param = (SRes<MyImageBindGroups>, SQuery<Read<ExtractedArrowTexture>>);

    fn render<'w>(
        view: Entity,
        item: Entity,
        (image_bind_groups, extracted_arrow_texture_query): bevy::ecs::system::SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let extracted_arrow_texture = extracted_arrow_texture_query.get(item).unwrap();
        info!(
            "SetArrowTextureBindGroup.render( {:?} )",
            extracted_arrow_texture.image_handle_id
        );
        let bind_group = image_bind_groups
            .into_inner()
            .values
            .get(&Handle::weak(extracted_arrow_texture.image_handle_id))
            .unwrap();
        pass.set_bind_group(2, bind_group, &[]);
        RenderCommandResult::Success
    }
}

// The `EntityRenderCommand` that performs the actual draw calls.
struct DrawArrowInstanced;
impl EntityRenderCommand for DrawArrowInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Mesh2dHandle>>,
        SQuery<Read<InstanceBuffer>>,
    );

    fn render<'w>(
        view: Entity,
        item: Entity,
        (meshes, mesh2d_query, instance_buffer_query): bevy::ecs::system::SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let mesh_handle = &mesh2d_query.get(item).unwrap().0;
        // info!("DrawArrowInstanced#render({:?})", item);
        let instance_buffer = instance_buffer_query.get(item).unwrap();

        if let Some(gpu_mesh) = meshes.into_inner().get(mesh_handle) {
            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..instance_buffer.length);
                }
                GpuBufferInfo::NonIndexed { vertex_count } => {
                    pass.draw(0..*vertex_count, 0..1);
                }
            }
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

// Render pipeline for `ArrowInstances`.
struct ArrowInstancePipeline {
    mesh2d_pipeline: Mesh2dPipeline,
    texture_layout: BindGroupLayout,
}
impl FromWorld for ArrowInstancePipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh2d_pipeline = Mesh2dPipeline::from_world(world);
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let texture_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Arrow texture bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        Self {
            mesh2d_pipeline,
            texture_layout,
        }
    }
}

impl SpecializedPipeline for ArrowInstancePipeline {
    type Key = Mesh2dPipelineKey;

    // configure the buffer layouts, bind groups and shader modules.
    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        let vertex_attributes = vec![
            VertexAttribute {
                // color
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                //position
                format: VertexFormat::Float32x3,
                offset: 16, // size of color attribute. bevy stores MESH::*_ATTRIBUTES in alphabetical order.
                shader_location: 1, // location of the position attribute
            },
            VertexAttribute {
                // weight
                format: VertexFormat::Float32,
                offset: 28,
                shader_location: 2,
            },
        ];
        let vertex_array_stride = 3 * 4 + 4 * 4 + 4;

        let instance_vertex_attributes = vec![
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 3, // tail transform row 0
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 16,
                shader_location: 4, // tail transform 1
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 16 * 2,
                shader_location: 5, // tail transform 2
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 16 * 3,
                shader_location: 6, // tail transform 3
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 16 * 4,
                shader_location: 7, // head transform row 0
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 16 * 5,
                shader_location: 8, // head transform row 1
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 16 * 6,
                shader_location: 9, // head transform row 2
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 16 * 7,
                shader_location: 10, // head transform row 3
            },
        ];

        let vertex_state = VertexState {
            shader: TWO_TRANSFORM_INTER_SHADER_HANDLE.typed::<Shader>(),
            entry_point: "vs_main".into(),
            shader_defs: vec![],
            buffers: vec![
                VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                },
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<ExtractedArrowInstance>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: instance_vertex_attributes,
                },
            ],
        };
        let fragment_state = FragmentState {
            shader: TWO_TRANSFORM_INTER_SHADER_HANDLE.typed::<Shader>(),
            shader_defs: vec![],
            entry_point: "fs_main".into(),
            targets: vec![ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            }],
        };

        // the uniform buffers taken from wrapped mesh2d_pipeline
        let bind_groups_layout = vec![
            self.mesh2d_pipeline.view_layout.clone(),
            self.mesh2d_pipeline.mesh_layout.clone(),
            self.texture_layout.clone(),
        ];

        RenderPipelineDescriptor {
            label: Some("arrow instance pipeline".into()),
            layout: Some(bind_groups_layout),
            vertex: vertex_state,
            primitive: PrimitiveState {
                topology: key.primitive_topology(),
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(fragment_state),
        }
    }
}
