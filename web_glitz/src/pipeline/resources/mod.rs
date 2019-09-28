mod resources;
pub use self::resources::{
    BufferResource, IncompatibleResources, ResourceBindings, ResourceBindingsLayoutDescriptor,
    ResourceSlotDescriptor, ResourceSlotIdentifier, ResourceSlotKind, ResourceSlotType, Resources,
    SampledTextureType, TextureResource, TypedResourceBindings, TypedResourceBindingsLayout,
    TypedResourceBindingsLayoutDescriptor, TypedResourceSlotDescriptor,
};

mod resource_bindings_encoding;
pub use self::resource_bindings_encoding::{
    BindingDescriptor, ResourceBindingsEncoding, ResourceBindingsEncodingContext,
    StaticResourceBindingsEncoder,
};

pub(crate) mod resource_slot;
pub use self::resource_slot::IncompatibleInterface;
