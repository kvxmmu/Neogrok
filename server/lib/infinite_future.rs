use std::{
    future::Future,
    task::Poll,
};

pub enum Bottom {}
pub struct InfiniteFuture;

impl Future for InfiniteFuture {
    type Output = Bottom;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Pending
    }
}

pub const fn infinite_future() -> impl Future<Output = Bottom> {
    InfiniteFuture
}
