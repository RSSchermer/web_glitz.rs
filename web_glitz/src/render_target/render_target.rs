use crate::render_target::render_target_description::RenderTargetDescription;
use crate::render_target::render_target_attachment::{FloatAttachment, DepthStencilAttachment, DepthAttachment, StencilAttachment, IntegerAttachment, UnsignedIntegerAttachment, ColorAttachmentDescription, DepthStencilAttachmentDescription};
use crate::render_target::attachable_image_ref::AsAttachableImageRef;
use crate::image::format::{FloatRenderable, DepthRenderable, StencilRenderable, IntegerRenderable, DepthStencilRenderable, UnsignedIntegerRenderable};
use crate::render_pass::{Framebuffer, FloatBuffer, DepthStencilBuffer, DepthBuffer, StencilBuffer, IntegerBuffer, UnsignedIntegerBuffer};
use crate::render_target::render_target_encoding::{EncodingContext, RenderTargetEncoding, RenderTargetEncoder};

pub struct RenderTarget<C, Ds> {
    pub color: C,
    pub depth_stencil: Ds,
}

impl Default for RenderTarget<(), ()> {
    fn default() -> Self {
        RenderTarget {
            color: (),
            depth_stencil: (),
        }
    }
}

impl<'a, I, A, F> RenderTargetDescription for RenderTarget<I, ()>
    where
        I: IntoIterator<Item=&'a FloatAttachment<A>>,
        A: AsAttachableImageRef<'a, Format=F>,
        F: FloatRenderable,
{
    type Framebuffer = Framebuffer<Vec<FloatBuffer<F>>, ()>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding::from_float_colors(context, self.color)
    }
}

//impl<I, Ds, F0, F1> RenderTargetDescription
//for RenderTarget<FloatAttachments<I>, DepthStencilAttachment<Ds>>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F0>,
//        F0: FloatRenderable,
//        for<'a> Ds: IntoAttachableImageRef<'a, Format = F1>,
//        F1: DepthStencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<FloatBuffer<F0>>, DepthStencilBuffer<F1>>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self.color.images.into_iter().map(|image| FloatAttachment {
//            image,
//            load_op,
//            store_op,
//        });
//
//        RenderTargetEncoding::from_float_colors_and_depth_stencil(
//            context,
//            colors,
//            self.depth_stencil,
//        )
//    }
//}
//
//impl<I, Ds, F0, F1> RenderTargetDescription for RenderTarget<FloatAttachments<I>, DepthAttachment<Ds>>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F0>,
//        F0: FloatRenderable,
//        for<'a> Ds: IntoAttachableImageRef<'a, Format = F1>,
//        F1: DepthRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<FloatBuffer<F0>>, DepthBuffer<F1>>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self.color.images.into_iter().map(|image| FloatAttachment {
//            image,
//            load_op,
//            store_op,
//        });
//
//        RenderTargetEncoding::from_float_colors_and_depth(context, colors, self.depth_stencil)
//    }
//}
//
//impl<I, Ds, F0, F1> RenderTargetDescription for RenderTarget<FloatAttachments<I>, StencilAttachment<Ds>>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F0>,
//        F0: FloatRenderable,
//        for<'a> Ds: IntoAttachableImageRef<'a, Format = F1>,
//        F1: StencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<FloatBuffer<F0>>, StencilBuffer<F1>>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self.color.images.into_iter().map(|image| FloatAttachment {
//            image,
//            load_op,
//            store_op,
//        });
//
//        RenderTargetEncoding::from_float_colors_and_stencil(context, colors, self.depth_stencil)
//    }
//}
//
//impl<I, F> RenderTargetDescription for RenderTarget<IntegerAttachments<I>, ()>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F>,
//        F: IntegerRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<IntegerBuffer<F>>, ()>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self.color.images.into_iter().map(|image| IntegerAttachment {
//            image,
//            load_op,
//            store_op,
//        });
//
//        RenderTargetEncoding::from_integer_colors(context, colors)
//    }
//}
//
//impl<I, Ds, F0, F1> RenderTargetDescription
//for RenderTarget<IntegerAttachments<I>, DepthStencilAttachment<Ds>>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F0>,
//        F0: IntegerRenderable,
//        for<'a> Ds: IntoAttachableImageRef<'a, Format = F1>,
//        F1: DepthStencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<IntegerBuffer<F0>>, DepthStencilBuffer<F1>>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self.color.images.into_iter().map(|image| IntegerAttachment {
//            image,
//            load_op,
//            store_op,
//        });
//
//        RenderTargetEncoding::from_integer_colors_and_depth_stencil(
//            context,
//            colors,
//            self.depth_stencil,
//        )
//    }
//}
//
//impl<I, Ds, F0, F1> RenderTargetDescription for RenderTarget<IntegerAttachments<I>, DepthAttachment<Ds>>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F0>,
//        F0: IntegerRenderable,
//        for<'a> Ds: IntoAttachableImageRef<'a, Format = F1>,
//        F1: DepthRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<IntegerBuffer<F0>>, DepthBuffer<F1>>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self.color.images.into_iter().map(|image| IntegerAttachment {
//            image,
//            load_op,
//            store_op,
//        });
//
//        RenderTargetEncoding::from_integer_colors_and_depth(context, colors, self.depth_stencil)
//    }
//}
//
//impl<I, Ds, F0, F1> RenderTargetDescription for RenderTarget<IntegerAttachments<I>, StencilAttachment<Ds>>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F0>,
//        F0: IntegerRenderable,
//        for<'a> Ds: IntoAttachableImageRef<'a, Format = F1>,
//        F1: StencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<IntegerBuffer<F0>>, StencilBuffer<F1>>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self.color.images.into_iter().map(|image| IntegerAttachment {
//            image,
//            load_op,
//            store_op,
//        });
//
//        RenderTargetEncoding::from_integer_colors_and_stencil(context, colors, self.depth_stencil)
//    }
//}
//
//impl<I, F> RenderTargetDescription for RenderTarget<UnsignedIntegerAttachments<I>, ()>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F>,
//        F: UnsignedIntegerRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<UnsignedIntegerBuffer<F>>, ()>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self
//            .color
//            .images
//            .into_iter()
//            .map(|image| UnsignedIntegerAttachment {
//                image,
//                load_op,
//                store_op,
//            });
//
//        RenderTargetEncoding::from_unsigned_integer_colors(context, colors)
//    }
//}
//
//impl<I, Ds, F0, F1> RenderTargetDescription
//for RenderTarget<UnsignedIntegerAttachments<I>, DepthStencilAttachment<Ds>>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F0>,
//        F0: UnsignedIntegerRenderable,
//        for<'a> Ds: IntoAttachableImageRef<'a, Format = F1>,
//        F1: DepthStencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, DepthStencilBuffer<F1>>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self
//            .color
//            .images
//            .into_iter()
//            .map(|image| UnsignedIntegerAttachment {
//                image,
//                load_op,
//                store_op,
//            });
//
//        RenderTargetEncoding::from_unsigned_integer_colors_and_depth_stencil(
//            context,
//            colors,
//            self.depth_stencil,
//        )
//    }
//}
//
//impl<I, Ds, F0, F1> RenderTargetDescription
//for RenderTarget<UnsignedIntegerAttachments<I>, DepthAttachment<Ds>>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F0>,
//        F0: UnsignedIntegerRenderable,
//        for<'a> Ds: IntoAttachableImageRef<'a, Format = F1>,
//        F1: DepthRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, DepthBuffer<F1>>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self
//            .color
//            .images
//            .into_iter()
//            .map(|image| UnsignedIntegerAttachment {
//                image,
//                load_op,
//                store_op,
//            });
//
//        RenderTargetEncoding::from_unsigned_integer_colors_and_depth(
//            context,
//            colors,
//            self.depth_stencil,
//        )
//    }
//}
//
//impl<I, Ds, F0, F1> RenderTargetDescription
//for RenderTarget<UnsignedIntegerAttachments<I>, StencilAttachment<Ds>>
//    where
//        I: IntoIterator,
//        for<'a> I::Item: IntoAttachableImageRef<'a, Format = F0>,
//        F0: UnsignedIntegerRenderable,
//        for<'a> Ds: IntoAttachableImageRef<'a, Format = F1>,
//        F1: StencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, StencilBuffer<F1>>;
//
//    fn into_encoding(
//        self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        let load_op = self.color.load_op;
//        let store_op = self.color.store_op;
//
//        let colors = self
//            .color
//            .images
//            .into_iter()
//            .map(|image| UnsignedIntegerAttachment {
//                image,
//                load_op,
//                store_op,
//            });
//
//        RenderTargetEncoding::from_unsigned_integer_colors_and_stencil(
//            context,
//            colors,
//            self.depth_stencil,
//        )
//    }
//}

impl<C0> RenderTargetDescription for RenderTarget<C0, ()>
    where
        C0: ColorAttachmentDescription,
{
    type Framebuffer = Framebuffer<C0::Buffer, ()>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let encoder = RenderTargetEncoder::new(context);
        let encoder = self.color.encode(encoder).unwrap();

        encoder.finish()
    }
}

impl<C0, Ds> RenderTargetDescription for RenderTarget<C0, Ds>
    where
        C0: ColorAttachmentDescription,
        Ds: DepthStencilAttachmentDescription,
{
    type Framebuffer = Framebuffer<C0::Buffer, Ds::Buffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let encoder = RenderTargetEncoder::new(context);
        let encoder = self.color.encode(encoder).unwrap();
        let encoder = self.depth_stencil.encode(encoder);

        encoder.finish()
    }
}

macro_rules! impl_render_target_description {
    ($($C:ident),*) => {
        impl<$($C),*> RenderTargetDescription for RenderTarget<($($C),*), ()>
        where
            $($C: ColorAttachmentDescription),*
        {
            type Framebuffer = Framebuffer<($($C::Buffer),*), ()>;

            fn into_encoding(self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer> {
                let encoder = RenderTargetEncoder::new(context);

                #[allow(non_snake_case)]
                let ($($C),*) = self.color;

                $(
                    let encoder = $C.encode(encoder).unwrap();
                )*

                encoder.finish()
            }
        }

        impl<$($C),*, Ds> RenderTargetDescription for RenderTarget<($($C),*), Ds>
        where
            $($C: ColorAttachmentDescription),*,
            Ds: DepthStencilAttachmentDescription
        {
            type Framebuffer = Framebuffer<($($C::Buffer),*), Ds::Buffer>;

            fn into_encoding(self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer> {
                let encoder = RenderTargetEncoder::new(context);

                #[allow(non_snake_case)]
                let ($($C),*) = self.color;

                $(
                    let encoder = $C.encode(encoder).unwrap();
                )*

                let encoder = self.depth_stencil.encode(encoder);

                encoder.finish()
            }
        }
    }
}

impl_render_target_description!(C0, C1);
impl_render_target_description!(C0, C1, C2);
impl_render_target_description!(C0, C1, C2, C3);
impl_render_target_description!(C0, C1, C2, C3, C4);
impl_render_target_description!(C0, C1, C2, C3, C4, C5);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
impl_render_target_description!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15
);

