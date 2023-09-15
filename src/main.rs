mod executor;
mod async_timer;
mod utils;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use futures::task::SpawnExt;
use crate::async_timer::{possibly_return_a_future, returns_a_future, TimerFuture};
use crate::executor::new_executor_and_spawner;
use crate::utils::{MockNetWorkClient, PossibleFuture};

fn main() {
    dbg!("Entering main");
    let (executor, spawner) = new_executor_and_spawner();

    dbg!("Executor and spawner created");

    let future = StateMachine::State1;  // Initial state

    dbg!("Initial future state created");


    let client = MockNetWorkClient;

    spawner.spawn(async move {
        dbg!("starting the task");
        let res = returns_a_future(1).await.unwrap();
        dbg!(res);
        let res2 = returns_a_future(2).await.unwrap();
        dbg!(res2);
        let res3= returns_a_future(5).await.unwrap();
        dbg!(res3);

    });  // Spawn a task
    drop(spawner);  // Drop the spawner
    dbg!("Executor run");
    executor.try_run().unwrap();
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)
        .ok()
        .expect("Failed to read line");
    executor.try_run().expect("run second time failed");

    io::stdin().read_line(&mut answer)
        .ok()
        .expect("Failed to read line");

    executor.try_run().unwrap();
}

enum StateMachine {
    State1,
    State2(TimerFuture),
}

impl Future for StateMachine {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

        loop {
            dbg!("Entering poll loop");
            match *self {
                StateMachine::State1 => {
                    dbg!("In State1");
                    println!("howdy!");
                    let timer_future = TimerFuture::new(Duration::new(2, 0));
                    dbg!("Timer future created");
                    self.set(StateMachine::State2(timer_future));
                    dbg!("Transitioned to State2");
                },
                StateMachine::State2(ref mut inner) => {
                    dbg!("In State2");
                    match Pin::new(inner).poll(cx) {
                        Poll::Ready(()) => {
                            dbg!("Timer future is ready");
                            println!("done!");
                            return Poll::Ready(());
                        },
                        Poll::Pending => {
                            dbg!("Timer future is pending");
                            return Poll::Pending;
                        },
                    }
                },
            }
        }
    }
}
