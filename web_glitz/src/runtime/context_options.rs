use std::marker;

use serde_derive::Serialize;

use crate::rendering::{
    DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultMultisampleRenderTarget,
    DefaultRGBABuffer, DefaultRGBBuffer, DefaultRenderTarget, DefaultStencilBuffer,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PowerPreference {
    Default,
    HighPerformance,
    LowPower,
}

impl Default for PowerPreference {
    fn default() -> Self {
        PowerPreference::Default
    }
}

pub struct ContextOptions<T> {
    render_target: marker::PhantomData<T>,
    preserve_drawing_buffer: bool,
    fail_if_major_performance_caveat: bool,
    premultiplied_alpha: bool,
    power_preference: PowerPreference,
}

impl<T> ContextOptions<T> {
    pub fn preserve_drawing_buffer(&self) -> bool {
        self.preserve_drawing_buffer
    }

    pub fn fail_if_major_performance_caveat(&self) -> bool {
        self.fail_if_major_performance_caveat
    }

    pub fn premultiplied_alpha(&self) -> bool {
        self.premultiplied_alpha
    }

    pub fn power_preference(&self) -> PowerPreference {
        self.power_preference
    }
}

impl Default for ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBABuffer, ()>> {
    fn default() -> Self {
        ContextOptions {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: false,
            preserve_drawing_buffer: false,
            premultiplied_alpha: true,
            power_preference: PowerPreference::default(),
        }
    }
}

impl ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBABuffer, ()>> {
    pub fn begin() -> ContextOptionsBuilder<DefaultMultisampleRenderTarget<DefaultRGBABuffer, ()>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: false,
            preserve_drawbuffer: false,
            premultiplied_alpha: true,
            power_preference: PowerPreference::default(),
        }
    }
}

pub struct ContextOptionsBuilder<T> {
    render_target: marker::PhantomData<T>,
    fail_if_major_performance_caveat: bool,
    preserve_drawbuffer: bool,
    premultiplied_alpha: bool,
    power_preference: PowerPreference,
}

impl<T> ContextOptionsBuilder<T> {
    pub fn fail_if_major_performance_caveat(
        mut self,
        fail_if_major_performance_caveat: bool,
    ) -> Self {
        self.fail_if_major_performance_caveat = fail_if_major_performance_caveat;

        self
    }

    pub fn preserve_drawbuffer(mut self, preserve_drawbuffer: bool) -> Self {
        self.preserve_drawbuffer = preserve_drawbuffer;

        self
    }

    pub fn premultiplied_alpha(mut self, premultiplied_alpha: bool) -> Self {
        self.premultiplied_alpha = premultiplied_alpha;

        self
    }

    pub fn power_preference(mut self, power_preference: PowerPreference) -> Self {
        self.power_preference = power_preference;

        self
    }

    pub fn finish(self) -> ContextOptions<T> {
        ContextOptions {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawing_buffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<C, Ds> ContextOptionsBuilder<DefaultMultisampleRenderTarget<C, Ds>> {
    pub fn disable_antialias(self) -> ContextOptionsBuilder<DefaultRenderTarget<C, Ds>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<Ds> ContextOptionsBuilder<DefaultMultisampleRenderTarget<DefaultRGBABuffer, Ds>> {
    pub fn disable_alpha(
        self,
    ) -> ContextOptionsBuilder<DefaultMultisampleRenderTarget<DefaultRGBBuffer, Ds>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<C> ContextOptionsBuilder<DefaultMultisampleRenderTarget<C, ()>> {
    pub fn enable_depth(
        self,
    ) -> ContextOptionsBuilder<DefaultMultisampleRenderTarget<C, DefaultDepthBuffer>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn enable_stencil(
        self,
    ) -> ContextOptionsBuilder<DefaultMultisampleRenderTarget<C, DefaultStencilBuffer>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<C> ContextOptionsBuilder<DefaultMultisampleRenderTarget<C, DefaultDepthBuffer>> {
    pub fn enable_stencil(
        self,
    ) -> ContextOptionsBuilder<DefaultMultisampleRenderTarget<C, DefaultDepthStencilBuffer>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<C> ContextOptionsBuilder<DefaultMultisampleRenderTarget<C, DefaultStencilBuffer>> {
    pub fn enable_depth(
        self,
    ) -> ContextOptionsBuilder<DefaultMultisampleRenderTarget<C, DefaultDepthStencilBuffer>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<Ds> ContextOptionsBuilder<DefaultRenderTarget<DefaultRGBABuffer, Ds>> {
    pub fn disable_alpha(self) -> ContextOptionsBuilder<DefaultRenderTarget<DefaultRGBBuffer, Ds>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<C> ContextOptionsBuilder<DefaultRenderTarget<C, ()>> {
    pub fn enable_depth(self) -> ContextOptionsBuilder<DefaultRenderTarget<C, DefaultDepthBuffer>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn enable_stencil(
        self,
    ) -> ContextOptionsBuilder<DefaultRenderTarget<C, DefaultStencilBuffer>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<C> ContextOptionsBuilder<DefaultRenderTarget<C, DefaultDepthBuffer>> {
    pub fn enable_stencil(
        self,
    ) -> ContextOptionsBuilder<DefaultRenderTarget<C, DefaultDepthStencilBuffer>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<C> ContextOptionsBuilder<DefaultRenderTarget<C, DefaultStencilBuffer>> {
    pub fn enable_depth(
        self,
    ) -> ContextOptionsBuilder<DefaultRenderTarget<C, DefaultDepthStencilBuffer>> {
        ContextOptionsBuilder {
            render_target: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}
