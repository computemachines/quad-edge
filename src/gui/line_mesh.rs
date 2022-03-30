use bevy::{
    core_pipeline::Transparent2d,
    ecs::system::lifetimeless::{Read, SQuery, SRes},
    prelude::*,
    reflect::{TypeUuid, Uuid},
    render::{
        texture::BevyDefault,
        mesh::GpuBufferInfo,
        render_asset::{RenderAsset, RenderAssets},
        render_phase::{
            AddRenderCommand, EntityRenderCommand, RenderCommandResult, SetItemPipeline,
        },
        render_resource::{
            SpecializedPipeline, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode, FragmentState, ColorTargetState, BlendState, ColorWrites, TextureFormat, RenderPipelineDescriptor, PrimitiveState, PrimitiveTopology, FrontFace, PolygonMode, MultisampleState,
        },
        RenderApp,
    },
    sprite::{
        Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
};

#[derive(Component, Default)]
pub struct LineMesh;

pub const LINES_MESH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x97391c0926e9d409);
pub struct LineMeshPlugin;
impl Plugin for LineMeshPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            LINES_MESH_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../../assets/shaders/line_mesh.wgsl")),
        );
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app.add_render_command::<Transparent2d, (
            SetItemPipeline,
            SetMesh2dViewBindGroup<0>,
            SetMesh2dBindGroup<1>,
            DrawLines2d,
        )>();
    }
}

struct DrawLines2d;
impl EntityRenderCommand for DrawLines2d {
    type Param = (SRes<RenderAssets<Mesh>>, SQuery<Read<Mesh2dHandle>>);

    fn render<'w>(
        view: Entity,
        item: Entity,
        (meshes, mesh2d_query): bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let mesh_handle = &mesh2d_query.get(item).unwrap().0;
        if let Some(gpu_mesh) = meshes.into_inner().get(mesh_handle) {
            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..1);
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

struct LinesMeshPipeline {
    mesh2d_pipeline: Mesh2dPipeline,
}
impl FromWorld for LinesMeshPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

impl SpecializedPipeline for LinesMeshPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        let vertex_attributes = vec![
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 16, // size of color attribute. bevy stores attributes in alphabetical order.
                shader_location: 0, // location of the position attribute
            },
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 1,
            },
        ];
        let vertex_array_stride = 28; // 3*4 + 4*4

        let vertex_state = VertexState {
            shader: LINES_MESH_SHADER_HANDLE.typed::<Shader>(),
            entry_point: "vs_main".into(),
            shader_defs: vec![],
            buffers: vec![VertexBufferLayout {
                array_stride: vertex_array_stride,
                step_mode: VertexStepMode::Vertex,
                attributes: vertex_attributes,
            }],
        };
        let fragment_state = FragmentState {
            shader: LINES_MESH_SHADER_HANDLE.typed::<Shader>(),
            shader_defs: vec![],
            entry_point: "fs_main".into(),
            targets: vec![
                ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }
            ]
        };

        let bind_groups_layout = vec![
            self.mesh2d_pipeline.view_layout.clone(),
            self.mesh2d_pipeline.mesh_layout.clone(),
        ];

        RenderPipelineDescriptor {
            label: Some("Line mesh pipeline".into()),
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
