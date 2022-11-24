use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

pub enum Never {}

pub struct InfiniteFuture;
impl Future for InfiniteFuture {
    type Output = Never;

    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        Poll::Pending
    }
}

pub const fn wait_forever() -> InfiniteFuture {
    InfiniteFuture
}
