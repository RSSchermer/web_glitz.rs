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
    queue: VecDeque<(WebGlSync, Box<dyn ExecutorJob>)>,
    connection: Rc<RefCell<Connection>>
}

impl FencedTaskQueue {
    pub(crate) fn new(connection: Rc<RefCell<Connection>>) -> Self {
        FencedTaskQueue {
            queue: VecDeque::new(),
            connection
        }
    }

    pub(crate) fn push<T>(&mut self, job: T)
    where
        T: ExecutorJob + 'static,
    {
        let connection = self.connection.borrow_mut();
        let (gl, _) = unsafe { connection.unpack() };
        let fence = gl.fence_sync(Gl::SYNC_GPU_COMMANDS_COMPLETE, 0).unwrap();

        self.queue.push_back((fence, Box::new(job)));
    }

    pub(crate) fn run(&mut self) -> bool {
        // Make sure the connection reference goes out of scope, before we borrow it again below.
        let gl = {
            let connection = self.connection.borrow_mut();
            let (gl, _) = unsafe { connection.unpack() };

            gl.clone()
        };

        while let Some((fence, _)) = self.queue.front() {
            let sync_status = gl
                .get_sync_parameter(fence, Gl::SYNC_STATUS)
                .as_f64()
                .unwrap() as u32;

            if sync_status == Gl::SIGNALED {
                let (_, mut job) = self.queue.pop_front().unwrap();

                if JobState::ContinueFenced == job.progress(&mut self.connection.borrow_mut()) {
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
    queue: Rc<RefCell<FencedTaskQueue>>,
    loop_handle: Option<JsTimeoutFencedTaskLoopHandle>,
}

impl JsTimeoutFencedTaskRunner {
    pub(crate) fn new(connection: Rc<RefCell<Connection>>) -> Self {
        JsTimeoutFencedTaskRunner {
            queue: Rc::new(RefCell::new(FencedTaskQueue::new(connection))),
            loop_handle: None,
        }
    }

    pub(crate) fn schedule<T>(&mut self, job: T)
    where
        T: ExecutorJob + 'static,
    {
        self.queue
            .borrow_mut()
            .push(job);

        let loop_running = if let Some(handle) = &self.loop_handle {
            !handle.cancelled()
        } else {
            false
        };

        if !loop_running {
            self.loop_handle = Some(JsTimeoutFencedTaskLoop::init(
                self.queue.clone(),
            ));
        }
    }
}

#[derive(Clone)]
struct JsTimeoutFencedTaskLoop {
    queue: Rc<RefCell<FencedTaskQueue>>,
    closure: Weak<Option<Closure<dyn FnMut()>>>,
    handle: Rc<Cell<i32>>,
    cancelled: Rc<Cell<bool>>,
}

impl JsTimeoutFencedTaskLoop {
    fn init(
        queue: Rc<RefCell<FencedTaskQueue>>,
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
            closure: Rc::downgrade(&closure_container),
            handle: handle.clone(),
            cancelled: cancelled.clone(),
        }) as Box<dyn FnMut()>);

        let handle_id = window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1,
            )
            .unwrap();

        unsafe {
            // There's one alive weak pointer that we passed to the JsTimeoutFencedTaskLoop above,
            // but it's not currently dereferenced (the soonest it could be dereferenced is in the
            // next macro-task) so this should be safe.
            *Rc::get_mut_unchecked(&mut closure_container) = Some(closure);
        }

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
            .run();

        // If there are still unfinished jobs in the queue, schedule another callback; otherwise,
        // stop the loop (by not scheduling a new callback) and wait for a new job to be scheduled
        // with the task runner (which will kick of a new loop).
        if !is_empty {
            // This fails if the loop handle is dropped, in which case the loop is already cancelled
            // and we won't schedule a new timeout.
            if let Some(container) = self.closure.upgrade() {
                let closure = container
                    .deref()
                    .as_ref()
                    .expect("Uninitialized closure container.");

                let handle_id = window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        1,
                    )
                    .unwrap();

                self.handle.set(handle_id);
            }
        } else {

            self.cancelled.set(true);
        }
    }
}

struct JsTimeoutFencedTaskLoopHandle {
    inner: Rc<Cell<i32>>,
    #[allow(dead_code)] // Just holding onto this so it doesn't get dropped prematurely
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
