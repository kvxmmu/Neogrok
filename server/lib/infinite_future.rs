use std::{
    convert::Infallible,
    future::Future,
    task::Poll,
};

pub struct InfiniteFuture;

impl Future for InfiniteFuture {
    type Output = Infallible;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Pending
    }
}

pub const fn infinite_future() -> impl Future<Output = Infallible> {
    InfiniteFuture
}
