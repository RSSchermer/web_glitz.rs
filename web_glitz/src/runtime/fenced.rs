use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{window, WebGl2RenderingContext as Gl, WebGlSync};

use crate::runtime::executor_job::{ExecutorJob, JobState};
use crate::runtime::Connection;

pub(crate) struct FencedTaskQueue {
    queue: VecDeque<(WebGlSync, Box<ExecutorJob>)>,
}

impl FencedTaskQueue {
    pub(crate) fn new() -> Self {
        FencedTaskQueue {
            queue: VecDeque::new(),
        }
    }

    pub(crate) fn push<T>(&mut self, job: T, connection: &mut Connection)
    where
        T: ExecutorJob + 'static,
    {
        let Connection(gl, _) = connection;
        let fence = gl.fence_sync(Gl::SYNC_GPU_COMMANDS_COMPLETE, 0).unwrap();

        self.queue.push_back((fence, Box::new(job)));
    }

    pub(crate) fn run(&mut self, connection: &mut Connection) -> bool {
        let Connection(gl, _) = connection;
        let gl = gl.clone();

        while let Some((fence, _)) = self.queue.front() {
            let sync_status = gl.clone()
                .get_sync_parameter(fence, Gl::SYNC_STATUS)
                .as_f64()
                .unwrap() as u32;

            if sync_status == Gl::SIGNALED {
                let (_, mut job) = self.queue.pop_front().unwrap();

                if JobState::ContinueFenced == job.progress(connection) {
                    let new_fence = gl.fence_sync(Gl::SYNC_GPU_COMMANDS_COMPLETE, 0).unwrap();

                    self.queue.push_back((new_fence, job));
                }
            } else {
                // We can break immediately after we encounter the first sync object that isn't
                // signalled, without having to check the remaining sync objects: the queue contains
                // the sync objects in order of creation, and sync objects can only become signalled
                // in the order in which they were created.

                break;
            }
        }

        self.queue.is_empty()
    }
}

pub(crate) struct JsTimeoutFencedTaskRunner {
    connection: Rc<RefCell<Connection>>,
    queue: Rc<RefCell<FencedTaskQueue>>,
    loop_handle: Option<JsTimeoutFencedTaskLoopHandle>,
}

impl JsTimeoutFencedTaskRunner {
    pub(crate) fn new(connection: Rc<RefCell<Connection>>) -> Self {
        JsTimeoutFencedTaskRunner {
            connection,
            queue: Rc::new(RefCell::new(FencedTaskQueue::new())),
            loop_handle: None,
        }
    }

    pub(crate) fn schedule<T>(&mut self, job: T)
    where
        T: ExecutorJob + 'static,
    {
        self.queue.borrow_mut().push(job, &mut self.connection.borrow_mut());

        let loop_running = if let Some(handle) = &self.loop_handle {
            !handle.cancelled()
        } else {
            false
        };

        if !loop_running {
            self.loop_handle = Some(JsTimeoutFencedTaskLoop::init(
                self.queue.clone(),
                self.connection.clone(),
            ));
        }
    }
}

#[derive(Clone)]
struct JsTimeoutFencedTaskLoop {
    queue: Rc<RefCell<FencedTaskQueue>>,
    connection: Rc<RefCell<Connection>>,
    closure: Weak<Option<Closure<dyn FnMut()>>>,
    handle: Rc<Cell<i32>>,
    cancelled: Rc<Cell<bool>>
}

impl JsTimeoutFencedTaskLoop {
    fn init(
        queue: Rc<RefCell<FencedTaskQueue>>,
        connection: Rc<RefCell<Connection>>,
    ) -> JsTimeoutFencedTaskLoopHandle {
        let handle = Rc::new(Cell::new(0));
        let cancelled = Rc::new(Cell::new(false));

        // We need to hold on to the closure for the duration of the loop, otherwise the callback
        // will become invalid, so we store in this Rc, which we'll give to the loop handle. The
        // loop handle will make sure to cancel the timeout first, before the Rc gets dropped, so
        // the callback should never become invalid while the loop is still running.
        let mut closure_container = Rc::new(None);

        let closure = Closure::wrap(Box::new(JsTimeoutFencedTaskLoop {
            queue,
            connection,
            closure: Rc::downgrade(&closure_container),
            handle: handle.clone(),
            cancelled: cancelled.clone()
        }) as Box<FnMut()>);

        let handle_id = window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1,
            )
            .unwrap();

        *Rc::get_mut(&mut closure_container).unwrap() = Some(closure);
        handle.set(handle_id);

        JsTimeoutFencedTaskLoopHandle {
            inner: handle,
            closure_container,
            cancelled,
        }
    }
}

impl FnOnce<()> for JsTimeoutFencedTaskLoop {
    type Output = ();

    extern "rust-call" fn call_once(mut self, _: ()) -> () {
        self.call_mut(())
    }
}

impl FnMut<()> for JsTimeoutFencedTaskLoop {
    extern "rust-call" fn call_mut(&mut self, _: ()) -> () {
        let is_empty = self
            .queue
            .borrow_mut()
            .run(&mut self.connection.borrow_mut());

        // If there are still unfinished jobs in the queue, schedule another callback; otherwise,
        // stop the loop (by not scheduling a new callback) and wait for a new job to be scheduled
        // with the task runner (which will kick of a new loop).
        if !is_empty {
            if let Some(container) = self.closure.upgrade() {
                if let Some(closure) = container.deref() {
                    let handle_id = window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            1,
                        )
                        .unwrap();

                    self.handle.set(handle_id);
                }
            }
        } {
            self.cancelled.set(true);
        }
    }
}

struct JsTimeoutFencedTaskLoopHandle {
    inner: Rc<Cell<i32>>,
    closure_container: Rc<Option<Closure<dyn FnMut()>>>,
    cancelled: Rc<Cell<bool>>,
}

impl JsTimeoutFencedTaskLoopHandle {
    fn cancelled(&self) -> bool {
        self.cancelled.get()
    }

    fn cancel(&mut self) -> bool {
        if !self.cancelled.get() {
            window()
                .unwrap()
                .clear_timeout_with_handle(self.inner.get());

            self.cancelled.set(true);

            true
        } else {
            false
        }
    }
}

impl Drop for JsTimeoutFencedTaskLoopHandle {
    fn drop(&mut self) {
        self.cancel();
    }
}
