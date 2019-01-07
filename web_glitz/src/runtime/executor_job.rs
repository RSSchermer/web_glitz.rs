use futures::sync::oneshot::{channel, Sender};

use crate::runtime::{Connection, Execution};
use crate::task::{GpuTask, Progress};

pub(crate) trait ExecutorJob {
    fn progress(&mut self, connection: &mut Connection) -> JobState;
}

#[derive(PartialEq)]
pub(crate) enum JobState {
    Finished,
    ContinueFenced,
}

pub(crate) struct Job<T>
where
    T: GpuTask<Connection>,
{
    task: T,
    result_tx: Option<Sender<T::Output>>,
}

impl<T> ExecutorJob for Job<T>
where
    T: GpuTask<Connection>,
{
    fn progress(&mut self, connection: &mut Connection) -> JobState {
        match self.task.progress(connection) {
            Progress::Finished(res) => {
                self.result_tx
                    .take()
                    .expect("Cannot make progress on a Job after it has finished")
                    .send(res)
                    .unwrap_or(());

                JobState::Finished
            }
            Progress::ContinueFenced => JobState::ContinueFenced,
        }
    }
}

pub(crate) fn job<T>(task: T) -> (Job<T>, Execution<T::Output>)
where
    T: GpuTask<Connection>,
{
    let (tx, rx) = channel();
    let job = Job {
        task,
        result_tx: Some(tx),
    };

    (job, Execution::Pending(rx))
}
