use std::future::Future;
use std::task::{self, Poll};

pub trait LayerFactory<R> {
    type Response;
    type Error;
    type Layer: Layer<R, Response = Self::Response, Error = Self::Error>;
    type InitError;
    type Future: Future<Output = Result<Self::Layer, Self::InitError>>;

    fn new_layer(&self, inner: L) -> Self::Future;
}

pub trait Layer<R> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn call(&self, req: R) -> Self::Future;
}
