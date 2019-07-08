use std::marker;

use serde_derive::Serialize;

use crate::render_pass::{
    DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultRGBABuffer, DefaultRGBBuffer,
    DefaultStencilBuffer,
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

pub struct ContextOptions<C, Ds> {
    color: marker::PhantomData<C>,
    depth_stencil: marker::PhantomData<Ds>,
    antialias: bool,
    preserve_drawing_buffer: bool,
    fail_if_major_performance_caveat: bool,
    premultiplied_alpha: bool,
    power_preference: PowerPreference,
}

impl<C, Ds> ContextOptions<C, Ds> {
    pub fn antialias(&self) -> bool {
        self.antialias
    }

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

impl Default for ContextOptions<DefaultRGBABuffer, ()> {
    fn default() -> Self {
        ContextOptions {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: false,
            antialias: true,
            preserve_drawing_buffer: false,
            premultiplied_alpha: true,
            power_preference: PowerPreference::default(),
        }
    }
}

impl ContextOptions<DefaultRGBABuffer, ()> {
    pub fn begin() -> ContextOptionsBuilder<DefaultRGBABuffer, ()> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: false,
            antialias: true,
            preserve_drawbuffer: false,
            premultiplied_alpha: true,
            power_preference: PowerPreference::default(),
        }
    }
}

pub struct ContextOptionsBuilder<C, Ds> {
    color: marker::PhantomData<C>,
    depth_stencil: marker::PhantomData<Ds>,
    fail_if_major_performance_caveat: bool,
    antialias: bool,
    preserve_drawbuffer: bool,
    premultiplied_alpha: bool,
    power_preference: PowerPreference,
}

impl<C, Ds> ContextOptionsBuilder<C, Ds> {
    pub fn antialias(mut self, antialias: bool) -> Self {
        self.antialias = antialias;

        self
    }

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

    pub fn enable_alpha(self) -> ContextOptionsBuilder<DefaultRGBABuffer, Ds> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn disable_alpha(self) -> ContextOptionsBuilder<DefaultRGBBuffer, Ds> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn finish(self) -> ContextOptions<C, Ds> {
        ContextOptions {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawing_buffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<C> ContextOptionsBuilder<C, ()> {
    pub fn enable_depth(self) -> ContextOptionsBuilder<C, DefaultDepthBuffer> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn disable_depth(self) -> Self {
        self
    }

    pub fn enable_stencil(self) -> ContextOptionsBuilder<C, DefaultStencilBuffer> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn disable_stencil(self) -> Self {
        self
    }
}

impl<C> ContextOptionsBuilder<C, DefaultDepthBuffer> {
    pub fn enable_depth(self) -> Self {
        self
    }

    pub fn disable_depth(self) -> ContextOptionsBuilder<C, ()> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn enable_stencil(self) -> ContextOptionsBuilder<C, DefaultDepthStencilBuffer> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn disable_stencil(self) -> Self {
        self
    }
}

impl<C> ContextOptionsBuilder<C, DefaultStencilBuffer> {
    pub fn enable_depth(self) -> ContextOptionsBuilder<C, DefaultDepthStencilBuffer> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn disable_depth(self) -> Self {
        self
    }

    pub fn enable_stencil(self) -> Self {
        self
    }

    pub fn disable_stencil(self) -> ContextOptionsBuilder<C, ()> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}

impl<C> ContextOptionsBuilder<C, DefaultDepthStencilBuffer> {
    pub fn enable_depth(self) -> Self {
        self
    }

    pub fn disable_depth(self) -> ContextOptionsBuilder<C, DefaultStencilBuffer> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }

    pub fn enable_stencil(self) -> Self {
        self
    }

    pub fn disable_stencil(self) -> ContextOptionsBuilder<C, DefaultDepthBuffer> {
        ContextOptionsBuilder {
            color: marker::PhantomData,
            depth_stencil: marker::PhantomData,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat,
            antialias: self.antialias,
            preserve_drawbuffer: self.preserve_drawbuffer,
            premultiplied_alpha: self.premultiplied_alpha,
            power_preference: self.power_preference,
        }
    }
}
