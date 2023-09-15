use std::future::Future;
use std::pin::Pin;
use crate::async_timer::possibly_return_a_future;

pub enum PossibleFuture<T: Send> {
    Value(T),
    Future(Pin<Box<dyn Future<Output=T> + Send>>),
}

impl<T: Send> PossibleFuture<T> {
    pub fn unwrap_value(self) -> T {
        match self {
            PossibleFuture::Value(val) => val,
            PossibleFuture::Future(_) => panic!("Called unwrap_value on a Future variant"),
        }
    }
}


fn handle_logic<T, U, F>(res: PossibleFuture<T>, logic: F) -> PossibleFuture<U>
    where
        F: FnOnce(T) -> PossibleFuture<U> + Send + 'static,
        T: Send + 'static,
        U: Send + 'static,
{
    match res {
        PossibleFuture::Value(v) => logic(v),
        PossibleFuture::Future(f) => PossibleFuture::Future(Box::pin(async move {
            let result = f.await;
            match logic(result) {
                PossibleFuture::Value(v) => v,
                PossibleFuture::Future(f) => f.await,
            }
        })),
    }
}


pub struct MockNetWorkClient;

impl MockNetWorkClient {
    pub fn send(&self,num:u32) -> PossibleFuture<i64> {
        handle_logic(possibly_return_a_future(num), |x| {
            handle_logic(possibly_return_a_future(2), |y| {
                PossibleFuture::Value(decrypt(y))
            })
        })
    }
}

fn decrypt(num:f64) -> i64{
    (num + 1.0) as i64
}