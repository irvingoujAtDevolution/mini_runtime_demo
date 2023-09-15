use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;
use crate::utils::PossibleFuture;

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

/// Shared state between the future and the waiting thread
struct SharedState {
    /// Whether or not the sleep time has elapsed
    completed: bool,

    /// The waker for the task that `TimerFuture` is running on.
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        dbg!("Entering poll of TimerFuture");

        // Look at the shared state to see if the timer has already completed.
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.completed {
            dbg!("Timer has completed");
            Poll::Ready(())
        } else {
            dbg!("Timer not yet completed, setting waker");

            // Set waker so that the thread can wake up the current task
            // when the timer has completed.
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        dbg!("Creating new TimerFuture");

        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        dbg!("Shared state created, spawning thread");

        // Spawn the new thread
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            dbg!("Inside spawned thread, going to sleep");
            thread::sleep(duration);

            dbg!("Thread awake, locking shared state");
            let mut shared_state = thread_shared_state.lock().unwrap();

            dbg!("Setting completed to true and waking up future if it has a waker");

            // Signal that the timer has completed and wake up the last task.
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                dbg!("Waker wake");
                waker.wake();
            }
        });

        TimerFuture { shared_state }
    }
}

pub fn possibly_return_a_future(num:u32) ->PossibleFuture<f64>{
    if num%2 == 0 {
        return PossibleFuture::Value(num.clone() as f64);
    }
    // let num_clone = num;
    dbg!("return a future");
    return PossibleFuture::Future(
        Box::pin(async move {
            TimerFuture::new(Duration::from_secs(3)).await;
            num as f64
        })
    )
}

pub fn possibly_return_another_future(num:u32) ->PossibleFuture<u32>{
    let num = num -1;
    if num%2 == 0 {
        return PossibleFuture::Value(num.clone());
    }
    return PossibleFuture::Future(
        Box::pin(async move {
            TimerFuture::new(Duration::from_secs(3)).await;
            num
        })
    )
}

pub async fn returns_a_future(num:u32) -> Result<u32,Box<dyn std::error::Error>> {
    dbg!("returns future is called");
    TimerFuture::new(Duration::from_secs(num as u64)).await;
    dbg!("returns future is done");
    Ok(num)
}
