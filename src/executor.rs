use futures::{future::{BoxFuture}, FutureExt};
use std::{
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex},
    task::Context,
    time::Duration,
};
use std::fmt::Error;
use std::future::Future;
use futures::task::{ArcWake, waker_ref};
use lazy_static::lazy_static;
// The timer we wrote in the previous section:

/// Task executor that receives tasks off of a channel and runs them.
pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

impl Executor {

    pub(crate) fn try_run(&self) -> Result<(),Error>{
        if let Ok(task) = self.ready_queue.recv() {
            dbg!("polling task ",*task.count.clone());
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if future.as_mut().poll(context).is_pending() {
                    dbg!("polled, is pending");
                    *future_slot = Some(future);
                }else {
                    dbg!("polled, is done");
                }
            }
            return Ok(());
        }
        Err(Error::default())
    }
    pub(crate) fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            dbg!("polling task ",*task.count.clone());
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if future.as_mut().poll(context).is_pending() {
                    dbg!("polled, is pending");
                    *future_slot = Some(future);
                }
            }
        }
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        dbg!("waker by ref, adding to queue");
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

lazy_static!{
    static ref COUNT:Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output=()> + 'static + Send) {
        let future = future.boxed();
        unsafe {
            let task = Arc::new(Task {
                count: Arc::new(COUNT.lock().unwrap().clone()),
                future: Mutex::new(Some(future)),
                task_sender: self.task_sender.clone(),
            });
            self.task_sender.send(task).expect("too many tasks queued");
        }
    }
}

/// A future that can reschedule itself to be polled by an `Executor`.
pub struct Task {
    count: Arc<u32>,
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>,
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}