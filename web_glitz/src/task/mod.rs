//! WebGlitz's execution model centers around the concepts of "tasks" and "commands". A "task" is a
//! unit of work that is to be executed on a graphical processing unit (GPU). A "command" is an
//! atomic task; a task may composed of multiple commands. There is not a specific type that
//! represents a command in this library, both tasks and commands are represented by the [GpuTask]
//! trait. The term "command" is only used by convention for the atomic task building blocks
//! provided by WebGlitz that you may combine into more complex tasks.
//!
//! The following example shows how we may create a task consists of a single command that uploads
//! image data to the base level of a [Texture2D]:
//!
//! ```rust
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
//! use web_glitz::image::{Image2DSource, MipmapLevels};
//! use web_glitz::image::format::RGB8;
//! use web_glitz::image::texture_2d::Texture2DDescriptor;
//!
//! // First we create the texture that we are going to upload to:
//! let texture = context.create_texture_2d(&Texture2DDescriptor {
//!     format: RGB8,
//!     width: 256,
//!     height: 256,
//!     levels: MipmapLevels::Complete
//! }).unwrap();
//!
//! // Next, we define some data we wish to upload
//! let pixels: Vec<[u8; 3]> = vec![[255, 0, 0]; 256 * 256];
//! let data = Image2DSource::from_pixels(pixels, 256, 256).unwrap();
//!
//! // Finally, we define a task that consist of a single upload command that will upload our data:
//! let task = texture.base_level().upload_command(data);
//! # }
//! ```
//!
//! Here `context` is a [RenderingContext]. For more information on textures, please refer to the
//! documentation for the [image] module.
//!
//! # Task combinators
//!
//! Combining tasks and commands into more complex tasks is done by "sequencing" or "joining". Both
//! sequencing and joining involve combining 2 or more sub-tasks into 1 new combined task, but they
//! have different guarantees about the order in which these sub-tasks are executed.
//!
//! A sequence may be created with functions such as [sequence], [sequence3], [sequence4] or
//! [sequence5] or the with [sequence_all] macro, which sequences any number of sub-tasks. In all
//! cases, the sub-tasks are considered to have an order that corresponds to the order in which they
//! were passed to these functions/the macro. A sub-task in a sequence only begins executing after
//! the previous sub-task in this ordering has finished executing completely. The sequence is
//! considered to be finished when the last sub-task has finished executing. The following example
//! expands on the previous example by adding a command that generates mipmap data for the texture:
//!
//! ```rust
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
//! use web_glitz::image::{Image2DSource, MipmapLevels};
//! use web_glitz::image::format::RGB8;
//! use web_glitz::image::texture_2d::Texture2DDescriptor;
//!
//! let texture = context.create_texture_2d(&Texture2DDescriptor {
//!     format: RGB8,
//!     width: 256,
//!     height: 256,
//!     levels: MipmapLevels::Complete
//! }).unwrap();
//!
//! let pixels: Vec<[u8; 3]> = vec![[255, 0, 0]; 256 * 256];
//! let data = Image2DSource::from_pixels(pixels, 256, 256).unwrap();
//!
//! // This time our tasks consists of a sequence of two commands:
//! let task = sequence(
//!     texture.base_level().upload_command(data),
//!     texture.generate_mipmap_command()
//! );
//! # }
//! ```
//!
//! This example will first upload our data to the texture's base level. Then, after the data has
//! finished uploading, the data for the remaining texture levels is generated with a "generate
//! mipmap" command. For details on mipmapping and mipmap levels, see the documentation for the
//! [image] module.
//!
//! A join may be created with functions such as [join], [join3], [join4] or [join5] functions or
//! the with [join_all] macro, which joins any number of sub-tasks. Joining is similar to
//! sequencing, except for that joining does not give any guarantees about the order in which the
//! sub-tasks begin executing. The join is considered to have finished when all sub-tasks have
//! finished executing. A join may be faster than a sequence.
//!
//! The following example shows how we might upload data to the base level for all faces of a
//! [TextureCube], before generating the data for the remaining levels with a "generate mipmap"
//! command:
//!
//! ```rust
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
//! use web_glitz::image::{Image2DSource, MipmapLevels};
//! use web_glitz::image::format::RGB8;
//! use web_glitz::image::texture_cube::TextureCubeDescriptor;
//!
//! // First we create the cube-map texture we are going to upload to:
//! let texture = context.create_texture_cube(&TextureCubeDescriptor {
//!     format: RGB8,
//!     width: 256,
//!     height: 256,
//!     levels: MipmapLevels::Complete
//! }).unwrap();
//!
//! // Then we define some data for each of the cube-map faces:
//! let positive_x_pixels: Vec<[u8; 3]> = vec![[255, 0, 0]; 256 * 256];
//! let positive_x_data = Image2DSource::from_pixels(pixels, 256, 256).unwrap();
//! let negative_x_pixels: Vec<[u8; 3]> = vec![[0, 255, 0]; 256 * 256];
//! let negative_x_data = Image2DSource::from_pixels(pixels, 256, 256).unwrap();
//! let positive_y_pixels: Vec<[u8; 3]> = vec![[0, 0, 255]; 256 * 256];
//! let positive_y_data = Image2DSource::from_pixels(pixels, 256, 256).unwrap();
//! let negative_y_pixels: Vec<[u8; 3]> = vec![[255, 255, 0]; 256 * 256];
//! let negative_y_data = Image2DSource::from_pixels(pixels, 256, 256).unwrap();
//! let positive_z_pixels: Vec<[u8; 3]> = vec![[255, 0, 255]; 256 * 256];
//! let positive_z_data = Image2DSource::from_pixels(pixels, 256, 256).unwrap();
//! let negative_z_pixels: Vec<[u8; 3]> = vec![[0, 255, 255]; 256 * 256];
//! let negative_z_data = Image2DSource::from_pixels(pixels, 256, 256).unwrap();
//!
//! // Finally, we define our task:
//! let task = sequence(
//!     join_all![
//!         texture.base_level().positive_x().upload_command(positive_x_data),
//!         texture.base_level().negative_x().upload_command(negative_x_data),
//!         texture.base_level().positive_y().upload_command(positive_y_data),
//!         texture.base_level().negative_y().upload_command(negative_y_data),
//!         texture.base_level().positive_z().upload_command(positive_z_data),
//!         texture.base_level().negative_z().upload_command(negative_z_data),
//!     ],
//!     texture.generate_mipmap_command()
//! );
//! # }
//! ```
//!
//! In this case, we don't really care about the order in which the uploads to each of the cube's
//! faces finish, so we use the [join_all] macro to join them into a task. However, it is important
//! that we do not begin generating the mipmap data for the remaining levels before all uploads
//! have finished. We therefor use [sequence] to sequence our combined upload command with the
//! "generate mipmap" command.
//!
//! # Submitting tasks
//!
//! A task does not actually do anything until it is submitted to a [RenderingContext] with
//! [RenderingContext::submit]:
//!
//! ```rust
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
//! # use web_glitz::image::{Image2DSource, MipmapLevels};
//! # use web_glitz::image::format::RGB8;
//! # use web_glitz::image::texture_2d::Texture2DDescriptor;
//! # let texture = context.create_texture_2d(&Texture2DDescriptor {
//! #     format: RGB8,
//! #     width: 256,
//! #     height: 256,
//! #     levels: MipmapLevels::Complete
//! # }).unwrap();
//! # let pixels: Vec<[u8; 3]> = vec![[255, 0, 0]; 256 * 256];
//! # let data = Image2DSource::from_pixels(pixels, 256, 256).unwrap();
//! # let task = texture.base_level().upload_command(data);
//! let future = context.submit(task);
//! # }
//! ```
//!
//! This will return a [Future] that will resolve with the task's output (see [GpuTask::Output])
//! after the task has finished executing.

mod gpu_task;
pub use self::gpu_task::{ContextId, GpuTask, GpuTaskExt, Progress};

mod join;
pub use self::join::{Join, Join3, Join4, Join5};

mod sequence;
pub use self::sequence::{Sequence, Sequence3, Sequence4, Sequence5};

mod maybe_done;
