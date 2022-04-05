use bevy::{
    core::{FloatOrd, Pod, Zeroable},
    core_pipeline::Transparent2d,
    ecs::system::lifetimeless::{Read, SQuery, SRes},
    prelude::*,
    reflect::{TypeUuid, Uuid},
    render::{
        mesh::GpuBufferInfo,
        render_asset::{RenderAsset, RenderAssets},
        render_component::{ExtractComponent, ExtractComponentPlugin},
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline,
        },
        render_resource::{
            BlendState, Buffer, BufferInitDescriptor, BufferUsages, ColorTargetState, ColorWrites,
            FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineCache, RenderPipelineDescriptor, SpecializedPipeline,
            SpecializedPipelines, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
            VertexState, VertexStepMode,
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
};

#[derive(Bundle, Debug, Clone)]
pub struct ArrowBundle {
    pub arrow: Arrow,
    pub origin: Transform,
    pub global: GlobalTransform,
    pub visible: Visibility,
    pub computed_visibility: ComputedVisibility,
    // pub local_head: ArrowHead
    pub instances: InstanceMaterialData,
}

#[derive(Component, Default, Debug, Clone)]
pub struct Arrow;

#[derive(Component, Debug, Clone)]
pub struct ArrowHead(pub Transform);

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
        app.add_plugin(ExtractComponentPlugin::<InstanceMaterialData>::default());
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app.add_render_command::<Transparent2d, (
            SetItemPipeline,
            SetMesh2dViewBindGroup<0>,
            SetMesh2dBindGroup<1>,
            DrawArrowInstanced,
        )>();
        render_app
            .init_resource::<ArrowInstancePipeline>()
            .init_resource::<SpecializedPipelines<ArrowInstancePipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_arrow_instances)
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers);
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct InstanceMaterialData(pub Vec<InstanceData>);
impl ExtractComponent for InstanceMaterialData {
    type Query = &'static InstanceMaterialData;
    type Filter = ();

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        InstanceMaterialData(item.0.clone())
    }
}

#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub color: [f32; 4],
}

fn queue_arrow_instances(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    arrow_instance_pipeline: Res<ArrowInstancePipeline>,
    mut pipelines: ResMut<SpecializedPipelines<ArrowInstancePipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    arrow_instances: Query<
        (Entity, &Mesh2dHandle, &Mesh2dUniform),
        (With<Mesh2dHandle>, With<InstanceMaterialData>),
    >,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    // if arrow_instances.is_empty() {
    // return;
    // }
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_arrow_instanced = transparent_draw_functions
            .read()
            .get_id::<(
                SetItemPipeline,
                SetMesh2dViewBindGroup<0>,
                SetMesh2dBindGroup<1>,
                DrawArrowInstanced,
            )>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((entity, mesh2d_handle, mesh2d_uniform)) =
                arrow_instances.get(*visible_entity)
            {
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
            }
        }
    }
}

#[derive(Component, Debug)]
struct InstanceBuffer {
    buffer: Buffer,
    length: u32,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in query.iter() {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Instance buffer data"),
            contents: bytemuck::cast_slice(instance_data.0.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.0.len() as u32,
        });
    }
}

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

struct ArrowInstancePipeline {
    mesh2d_pipeline: Mesh2dPipeline,
}
impl FromWorld for ArrowInstancePipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

impl SpecializedPipeline for ArrowInstancePipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        let vertex_attributes = vec![
            VertexAttribute { //position
                format: VertexFormat::Float32x3,
                offset: 16, // size of color attribute. bevy stores attributes in alphabetical order.
                shader_location: 0, // location of the position attribute
            },
            VertexAttribute { // color
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 1,
            },
        ];
        let vertex_array_stride = 28; // 3*4 + 4*4

        let instance_vertex_attributes = vec![
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 2,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: VertexFormat::Float32x3.size(),
                shader_location: 3,
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
                    array_stride: std::mem::size_of::<InstanceData>() as u64,
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

        let bind_groups_layout = vec![
            self.mesh2d_pipeline.view_layout.clone(),
            self.mesh2d_pipeline.mesh_layout.clone(),
        ];

        RenderPipelineDescriptor {
            label: Some("arrow instance pipeline".into()),
            layout: Some(bind_groups_layout),
            vertex: vertex_state,
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList, //key.primitive_topology(),
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Line,
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
