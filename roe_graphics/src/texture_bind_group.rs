use super::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Instance, Sampler, ShaderStage,
    TextureSampleType, TextureView, TextureViewDimension,
};

// TODO: unit tests.
#[derive(Debug)]
pub struct TextureBindGroup {
    bind_group: BindGroup,
}

impl TextureBindGroup {
    pub fn new(instance: &Instance, texture_view: &TextureView, sampler: &Sampler) -> Self {
        let bind_group = BindGroup::new(
            instance,
            &BindGroupDescriptor {
                label: None,
                layout: &BindGroupLayout::new(
                    instance,
                    &BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            BindGroupLayoutEntry {
                                binding: 0,
                                visibility: ShaderStage::FRAGMENT,
                                ty: BindingType::Texture {
                                    multisampled: false,
                                    sample_type: TextureSampleType::Float { filterable: true },
                                    view_dimension: TextureViewDimension::D2Array,
                                },
                                count: None,
                            },
                            BindGroupLayoutEntry {
                                binding: 1,
                                visibility: ShaderStage::FRAGMENT,
                                ty: BindingType::Sampler {
                                    filtering: true,
                                    comparison: false,
                                },
                                count: None,
                            },
                        ],
                    },
                ),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(sampler),
                    },
                ],
            },
        );
        Self { bind_group }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
