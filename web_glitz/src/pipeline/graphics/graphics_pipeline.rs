

pub struct ResourceBindingsEncoding<T> {
    bindings: T
}

pub trait ConfirmableBindingsEncoding {
    fn
}

impl<T> ResourceBindingsEncoding<T> where T: Borrow<[ResourceBinding]> {

}



pub struct GraphicsPipelineDescriptor

pub unsafe trait ResourceBindingsLayout {
    type BindGroup;

    fn confirm_bindings(pipeline_slots: &[PipelineResourceSlotDescriptor]) -> Compatible;

    fn encode_bindings<R>(resources: R) -> BindingsEncoding<Self::BindGroup> where R: ResourceBindings<Self::BindGroup>;
}

pub trait PipelineResources {
    type BindGroup;


}

pub trait ResourceBindings<L> where L: ResourceBindingsLayout {
    fn encode_bindings<R>(&self, context: &mut EncodingContext) -> BindingsEncoding<L::BindGroup>;
}

pub struct ActiveGraphicsPipeline<Rl> {

}

impl<Rl> ActiveGraphicsPipeline<Rl> where Rl: ResourceBindingsLayout {
    fn draw_command<R>(&self, resources: R) -> DrawCommand where R: ResourceBindings<Rl::BindGroup> {
        unimplemented!()
    }
}

impl<B0, B1, B2, R0, R1, R2> ResourceBindings<(B0, B1, B2)> for (R0, R1, R2) where
    R0: Resource<Binding=B0>,
    R1: Resource<Binding=B1>,
    R2: Resource<Binding=B2>, {

    fn encode_bindings(&self, context: &mut EncodingContext) -> BindingsEncoding<(B0, B1, B2)> {
        BindingsEncoding::begin(context)
            .add(self.0)
            .add(self.1)
            .add(self.2)
            .finish()
    }
}

pub unsafe trait Binding {

}

pub struct BufferBinding {

}

pub struct FloatSampler2DBinding {

}

pub trait Resource {
    type Binding: Binding;

    fn binding(&self, slot: u32) -> Self::Binding;
}

pub trait PipelineResources {

}

impl PipelineResources for (R0, R1, R2, R3) {
    type Encoding;

    fn encode_bindings(&self) -> BindingsEncoding<Self::Encoding>;
}

pub trait LayoutCompatible {

}
