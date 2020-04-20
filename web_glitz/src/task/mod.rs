//! Provides a framework for specifying units of work that are to be executed by the GPU driver.
//!
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
//! let texture = context.try_create_texture_2d(&Texture2DDescriptor {
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
//! A sequence may be created with functions such as [sequence], [sequence3], [sequence4],
//! [sequence5] or [sequence_iter], or the with [sequence_all] macro, which sequences any number of
//! sub-tasks. In all cases, the sub-tasks are considered to have an order that corresponds to the
//! order in which they were passed to these functions/the macro. A sub-task in a sequence only
//! begins executing after the previous sub-task in this ordering has finished executing completely.
//! The sequence is considered to be finished when the last sub-task has finished executing. The
//! following example expands on the previous example by adding a command that generates mipmap data
//! for the texture:
//!
//! ```rust
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
//! use web_glitz::image::{Image2DSource, MipmapLevels};
//! use web_glitz::image::format::RGB8;
//! use web_glitz::image::texture_2d::Texture2DDescriptor;
//! use web_glitz::task::sequence;
//!
//! let texture = context.try_create_texture_2d(&Texture2DDescriptor {
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
//! A join may be created with functions such as [join], [join3], [join4], [join5] or [join_iter],
//! or the with [join_all] macro, which joins any number of sub-tasks. Joining is similar to
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
//! use web_glitz::task::{join_all, sequence_all};
//!
//! // First we create the cube-map texture we are going to upload to:
//! let texture = context.try_create_texture_cube(&TextureCubeDescriptor {
//!     format: RGB8,
//!     width: 256,
//!     height: 256,
//!     levels: MipmapLevels::Complete
//! }).unwrap();
//!
//! // Then we define some data for each of the cube-map faces:
//! let positive_x_pixels: Vec<[u8; 3]> = vec![[255, 0, 0]; 256 * 256];
//! let positive_x_data = Image2DSource::from_pixels(positive_x_pixels, 256, 256).unwrap();
//! let negative_x_pixels: Vec<[u8; 3]> = vec![[0, 255, 0]; 256 * 256];
//! let negative_x_data = Image2DSource::from_pixels(negative_x_pixels, 256, 256).unwrap();
//! let positive_y_pixels: Vec<[u8; 3]> = vec![[0, 0, 255]; 256 * 256];
//! let positive_y_data = Image2DSource::from_pixels(positive_y_pixels, 256, 256).unwrap();
//! let negative_y_pixels: Vec<[u8; 3]> = vec![[255, 255, 0]; 256 * 256];
//! let negative_y_data = Image2DSource::from_pixels(negative_y_pixels, 256, 256).unwrap();
//! let positive_z_pixels: Vec<[u8; 3]> = vec![[255, 0, 255]; 256 * 256];
//! let positive_z_data = Image2DSource::from_pixels(positive_z_pixels, 256, 256).unwrap();
//! let negative_z_pixels: Vec<[u8; 3]> = vec![[0, 255, 255]; 256 * 256];
//! let negative_z_data = Image2DSource::from_pixels(negative_z_pixels, 256, 256).unwrap();
//!
//! // Finally, we define our task:
//! let task = sequence_all![
//!     join_all![
//!         texture.base_level().positive_x().upload_command(positive_x_data),
//!         texture.base_level().negative_x().upload_command(negative_x_data),
//!         texture.base_level().positive_y().upload_command(positive_y_data),
//!         texture.base_level().negative_y().upload_command(negative_y_data),
//!         texture.base_level().positive_z().upload_command(positive_z_data),
//!         texture.base_level().negative_z().upload_command(negative_z_data),
//!     ],
//!     texture.generate_mipmap_command()
//! ];
//! # }
//! ```
//!
//! In this case, we don't really care about the order in which the uploads to each of the cube's
//! faces finish, so we use the [join_all] macro to join them into a task. However, it is important
//! that we do not begin generating the mipmap data for the remaining levels before all uploads
//! have finished. We therefor use [sequence] to sequence our combined upload command with the
//! "generate mipmap" command.
//!
//! Note that all sequence and join functions and macros mentioned above also come in "left" and
//! "right" variants. For example, [sequence5] is accompanied by [sequence5_left] and
//! [sequence5_right]. The difference is in the combined task's output (the future result of the
//! task when it is submitted with [RenderingContext::submit], see below). [sequence5] outputs a
//! tuple of all 5 of the outputs of the 5 tasks it sequences. However, one is often only interested
//! in just the output of either the left-most (the first) or right-most (the last) task in the
//! sequence. This is where [sequence5_left] and [sequence5_right] come in: [sequence5_left] only
//! outputs the output of the left-most task and [sequence5_right] only outputs the output of the
//! right-most task. In all other aspects the behaviour of a task created with [sequence5_left] or
//! [sequence5_right] is identical to the behaviour of a task created with [sequence5].
//!
//! # Submitting tasks
//!
//! A task merely describes work for the GPU, it does not actually do anything until it is submitted
//! to a [RenderingContext] with [RenderingContext::submit]:
//!
//! ```rust
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
//! # use web_glitz::image::{Image2DSource, MipmapLevels};
//! # use web_glitz::image::format::RGB8;
//! # use web_glitz::image::texture_2d::Texture2DDescriptor;
//! # let texture = context.try_create_texture_2d(&Texture2DDescriptor {
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
//!
//! [Texture2D]: web_glitz::image::texture_2d::Texture2D
//! [RenderingContext]: web_glitz::runtime::RenderingContext
//! [TextureCube]: web_glitz::image::texture_cube::TextureCube
//! [Future]: std::future::Future

mod gpu_task;
pub use self::gpu_task::{ContextId, Empty, GpuTask, GpuTaskExt, Progress};

mod join;
pub use self::join::{
    join, join3, join3_left, join3_right, join4, join4_left, join4_right, join5, join5_left,
    join5_right, join_iter, join_left, join_right, Join, Join3, Join3Left, Join3Right, Join4,
    Join4Left, Join4Right, Join5, Join5Left, Join5Right, JoinIter, JoinLeft, JoinRight,
};

mod map;
pub use self::map::Map;

mod option_task;
pub use self::option_task::OptionTask;

mod sequence;
pub use self::sequence::{
    sequence, sequence3, sequence3_left, sequence3_right, sequence4, sequence4_left,
    sequence4_right, sequence5, sequence5_left, sequence5_right, sequence_iter, sequence_left,
    sequence_right, Sequence, Sequence3, Sequence3Left, Sequence3Right, Sequence4, Sequence4Left,
    Sequence4Right, Sequence5, Sequence5Left, Sequence5Right, SequenceIter, SequenceLeft,
    SequenceRight,
};

mod maybe_done;

/// Macro that joins all tasks.
pub use crate::join_all;

/// Macro that joins all tasks and returns only the output of the left-most task.
pub use crate::join_all_left;

/// Macro that joins all tasks and returns only the output of the right-most task.
pub use crate::join_all_right;

/// Macro that sequences all tasks and returns a tuple of all outputs.
pub use crate::sequence_all;

/// Macro that sequences all tasks and returns only the output of the left-most task.
pub use crate::sequence_all_left;

/// Macro that sequences all tasks and returns only the output of the right-most task.
pub use crate::sequence_all_right;
