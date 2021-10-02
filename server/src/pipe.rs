//! This is a pipe that can manipulate data through the pipe.
//!
//! # Examples
//!
//! ```
//! use std::fmt::Display;
//! use std::future::Future;
//! use std::marker::PhantomData;
//! use std::task::{Context, Poll};
//! use futures::future::{ok, Ready, LocalBoxFuture};
//! use cubby_connect_server::pipe::{Pipe, PipeFactory};
//!
//! // Factory of Echo.
//! pub struct EchoFactory;
//!
//! // Pipe that sends the message to next as is
//! pub struct Echo<T, P>
//! where
//!     P: Pipe<T>
//! {
//!     prev: P,
//!     _marker: PhantomData<T>,
//! }
//!
//! impl<T, P> PipeFactory<T, P> for EchoFactory
//! where
//!     P: Pipe<T>,
//!     P::Future: 'static,
//! {
//!     type Next = T;
//!     type Error = P::Error;
//!     type Pipe = Echo<T, P>;
//!     type InitError = ();
//!     type Future = Ready<Result<Self::Pipe, Self::InitError>>;
//!
//!     fn new_layer(&self, prev: P) -> Self::Future {
//!         ok(Echo {
//!             prev,
//!             _marker: PhantomData::default()
//!         })
//!     }
//! }
//!
//! impl<T, P> Pipe<T> for Echo<T, P>
//! where
//!     P: Pipe<T>,
//!     P::Future: 'static,
//! {
//!     type Error = P::Error;
//!     type Future = LocalBoxFuture<'static, Result<(), Self::Error>>;
//!
//!     fn call(&self, msg: T) -> Self::Future {
//!         let prev_call = self.prev.call(msg);
//!
//!         Box::pin(async move {
//!             prev_call.await?;
//!             Ok(())
//!         })
//!     }
//! }
//!
//! // print to stdout that got
//! pub struct Print;
//!
//! impl<S> Pipe<S> for Print
//! where
//!     S: Display
//! {
//!     type Error = ();
//!     type Future = Ready<Result<(), Self::Error>>;
//!
//!     fn call(&self, msg: S) -> Self::Future {
//!         print!("{}", msg);
//!         ok(())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), ()>{
//!     let p = Print;
//!     let ef = EchoFactory;
//!     let e = ef.new_layer(p).await?;
//!     // this would print "Hello, World!" to stdout
//!     e.call("Hello, World!").await?;
//!     Ok(())
//! }
//! ```
//!

use std::future::Future;

/// This is a factory for `Pipe`. Since `Pipe` has chain connection,
/// it have to hold the previous `Pipe`. It would be provided in factory.
pub trait PipeFactory<M, P>
where
    P: Pipe<Self::Next>,
{
    type Next;
    type Error;
    type Pipe: Pipe<M, Error = Self::Error>;
    type InitError;
    type Future: Future<Output = Result<Self::Pipe, Self::InitError>>;

    fn new_layer(&self, prev: P) -> Self::Future;
}

pub trait Pipe<M> {
    type Error;
    type Future: Future<Output = Result<(), Self::Error>>;

    fn call(&self, msg: M) -> Self::Future;
}
