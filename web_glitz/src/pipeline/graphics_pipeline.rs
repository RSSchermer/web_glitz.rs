pub struct Program<Fs, Tf> {
    data: Arc<ProgramData<Fs, Tf>>,
}

impl<Fs, Tf> Program<Fs, Tf> {
    pub(crate) fn bind_uniforms<T>(
        &mut self,
        connection: &mut Connection,
        uniforms: T,
    ) -> Result<(), BindingError>
        where
            T: Uniform,
    {
        let mut data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        for (identifier, slot) in data.active_uniforms.iter_mut() {
            uniforms.bind(identifier.as_tail(), &mut slot.as_bindable(connection))?;
        }

        Ok(())
    }
}

impl Program<FragmentShaderMarker, ()> {
    pub(crate) fn new<Rc>(
        context: &Rc,
        dropper: RefCountedDropper,
        descriptor: &ProgramDescriptor<FragmentShaderMarker, ()>,
    ) -> Self
        where
            Rc: RenderingContext,
    {
        let data = Arc::new(ProgramData {
            id: None,
            dropper,
            vertex_shader: descriptor.vertex_shader.clone(),
            fragment_shader: descriptor.fragment_shader.clone(),
            active_uniforms: Vec::new(),
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        });

        context.submit(ProgramAllocateTask { data: data.clone() });

        Program { data }
    }
}

struct ProgramData<Fs, Tf> {
    id: Option<JsId>,
    dropper: RefCountedDropper,
    vertex_shader: Option<Arc<ShaderData>>,
    fragment_shader: Option<Arc<ShaderData>>,
    active_uniforms: Vec<(UniformIdentifier, UniformSlot)>,
    _fragment_shader_marker: marker::PhantomData<Fs>,
    _transform_feedback_marker: marker::PhantomData<Tf>,
}

impl<Fs, Tf> Drop for ProgramData<Fs, Tf> {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_gl_object(DropObject::Program(id));
        }
    }
}

pub struct ProgramDescriptor<Fs, Tf> {
    vertex_shader: Option<Arc<ShaderData>>,
    fragment_shader: Option<Arc<ShaderData>>,
    transform_feedback_varyings: Option<TransformFeedbackVaryings>,
    _fragment_shader_marker: marker::PhantomData<Fs>,
    _transform_feedback_marker: marker::PhantomData<Tf>,
}

impl<Fs, Tf> ProgramDescriptor<Fs, Tf> {
    pub fn begin() -> ProgramDescriptorBuilder<(), (), ()> {
        ProgramDescriptorBuilder {
            vertex_shader: None,
            fragment_shader: None,
            transform_feedback_varyings: None,
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

pub struct ProgramDescriptorBuilder<Vs, Fs, Tf> {
    vertex_shader: Option<Arc<ShaderData>>,
    fragment_shader: Option<Arc<ShaderData>>,
    transform_feedback_varyings: Option<TransformFeedbackVaryings>,
    _vertex_shader_marker: marker::PhantomData<Vs>,
    _fragment_shader_marker: marker::PhantomData<Fs>,
    _transform_feedback_marker: marker::PhantomData<Tf>,
}

impl<Vs, Fs, Tf> ProgramDescriptorBuilder<Vs, Fs, Tf> {
    pub fn vertex_shader(
        self,
        vertex_shader: &VertexShader,
    ) -> ProgramDescriptorBuilder<VertexShaderMarker, Fs, Tf> {
        ProgramDescriptorBuilder {
            vertex_shader: Some(vertex_shader.data.clone()),
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }

    pub fn fragment_shader(
        self,
        fragment_shader: &FragmentShader,
    ) -> ProgramDescriptorBuilder<Vs, FragmentShaderMarker, Tf> {
        ProgramDescriptorBuilder {
            vertex_shader: self.vertex_shader,
            fragment_shader: Some(fragment_shader.data.clone()),
            transform_feedback_varyings: self.transform_feedback_varyings,
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }

    pub fn transform_feedback<V>(
        self,
        varyings: V,
    ) -> ProgramDescriptorBuilder<Vs, FragmentShaderMarker, TransformFeedbackMarker>
        where
            V: Into<TransformFeedbackVaryings>,
    {
        ProgramDescriptorBuilder {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: Some(varyings.into()),
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

impl ProgramDescriptorBuilder<VertexShaderMarker, FragmentShaderMarker, ()> {
    pub fn finish(self) -> ProgramDescriptor<FragmentShaderMarker, ()> {
        ProgramDescriptor {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

impl ProgramDescriptorBuilder<VertexShaderMarker, (), TransformFeedbackMarker> {
    pub fn finish(self) -> ProgramDescriptor<(), TransformFeedbackMarker> {
        ProgramDescriptor {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

impl ProgramDescriptorBuilder<VertexShaderMarker, FragmentShaderMarker, TransformFeedbackMarker> {
    pub fn finish(self) -> ProgramDescriptor<FragmentShaderMarker, TransformFeedbackMarker> {
        ProgramDescriptor {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

pub struct TransformFeedbackVaryings {
    names: Vec<String>,
}

impl TransformFeedbackVaryings {
    pub fn new<I, S>(names: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: Borrow<str>,
    {
        let names = names.into_iter().map(|n| n.borrow().to_string()).collect();

        TransformFeedbackVaryings { names }
    }
}

impl<I, S> From<I> for TransformFeedbackVaryings
    where
        I: IntoIterator<Item = S>,
        S: Borrow<str>,
{
    fn from(names: I) -> TransformFeedbackVaryings {
        TransformFeedbackVaryings::new(names)
    }
}

pub struct VertexShaderMarker;
pub struct FragmentShaderMarker;
pub struct TransformFeedbackMarker;

struct ProgramAllocateTask<Fs, Tf> {
    data: Arc<ProgramData<Fs, Tf>>,
}

impl GpuTask<Connection> for ProgramAllocateTask<FragmentShaderMarker, ()> {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let program_object = gl.create_program().unwrap();

        state
            .set_active_program(Some(&program_object))
            .apply(gl)
            .unwrap();

        unsafe {
            if let Some(ref shader_data) = data.vertex_shader {
                shader_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|shader_object| {
                        gl.attach_shader(&program_object, &shader_object);
                    });
            }

            if let Some(ref shader_data) = data.fragment_shader {
                shader_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|shader_object| {
                        gl.attach_shader(&program_object, &shader_object);
                    });
            }
        }

        gl.link_program(&program_object);

        data.active_uniforms = active_uniforms(&gl, &program_object);
        data.id = Some(JsId::from_value(program_object.into()));

        Progress::Finished(())
    }
}

// TODO: implement GpuTask for the other 2 combinations once web_sys supports transform_feedback_varyings

fn active_uniforms(gl: &Gl, program: &WebGlProgram) -> Vec<(UniformIdentifier, UniformSlot)> {
    let active_uniform_count = gl
        .get_program_parameter(program, Gl::ACTIVE_UNIFORMS)
        .as_f64()
        .unwrap() as u32;
    let active_block_count = gl
        .get_program_parameter(program, Gl::ACTIVE_UNIFORM_BLOCKS)
        .as_f64()
        .unwrap() as u32;
    let mut result = Vec::with_capacity((active_uniform_count + active_block_count) as usize);

    for i in 0..active_block_count {
        let name = gl.get_active_uniform_block_name(program, i).unwrap();
        let identifier = UniformIdentifier::from_string(&name);
        let slot = UniformSlot::new_block(program, i);

        result.push((identifier, slot));
    }

    for i in 0..active_uniform_count {
        let info = gl.get_active_uniform(program, i).unwrap();
        let name = info.name();

        // Even though the way in which we obtain the name guarantees that a uniform with the name
        // exists, the uniform may still not be associated with a uniform location. This is because
        // the list of active uniforms also includes uniforms that belong to uniform blocks.
        if let Some(location) = gl.get_uniform_location(&program, &name) {
            let identifier = UniformIdentifier::from_string(&name);
            let slot = UniformSlot::new_value(name, info.type_(), info.size() as usize, location);

            result.push((identifier, slot));
        }
    }

    result
}
