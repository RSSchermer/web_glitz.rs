mod resources;
pub use self::resources::{
    BindGroup, BindGroupLayoutBuilder, EncodeBindableResourceGroup, IncompatibleResources,
    InvalidBindGroupSequence, InvalidResourceSlotSequence, LayoutAllocationHint, Resource,
    ResourceBindings, ResourceBindingsLayoutBuilder, ResourceBindingsLayoutBuilderError,
    ResourceBindingsLayoutDescriptor, ResourceSlotDescriptor, ResourceSlotIdentifier,
    ResourceSlotKind, ResourceSlotType, Resources, SampledTextureType, TypedBindableResourceGroup,
    TypedResourceBindings, TypedResourceBindingsLayout, TypedResourceBindingsLayoutDescriptor,
    TypedResourceSlotDescriptor,
};

pub(crate) mod resource_bindings_encoding;
pub use self::resource_bindings_encoding::{
    BindGroupDescriptor, BindGroupEncoder, BindGroupEncoding, BindGroupEncodingContext,
    ResourceBindingsEncoding, ResourceBindingsEncodingContext, StaticResourceBindingsEncoder,
};

pub(crate) mod resource_slot;
pub use self::resource_slot::IncompatibleInterface;
