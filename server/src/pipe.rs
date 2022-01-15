//! This is a pipe that can manipulate data through the pipe.
//!
//! `PipeFactory` helps you to build pipe with previous pipe.
//! You can only use `Pipe` for start of the pipe.
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
//!     fn new_pipe(&self, prev: P) -> Self::Future {
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
//!         // this would act as same future of previous pipe,
//!         // but type of `Ok` is `()`
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
//!     // just calls `print!`
//!     // then just be ready
//!     fn call(&self, msg: S) -> Self::Future {
//!         // This should equal to "Hello, World!" in this example
//!         assert_eq!(msg.to_string(), "Hello, World!");
//!         print!("{}", msg);
//!         ok(())
//!     }
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), ()>{
//! let p = Print;
//! let ef = EchoFactory;
//! // `e` would be the pipe: `Echo` > `Print`
//! let e = ef.new_pipe(p).await?;
//! // this would print "Hello, World!" to stdout
//! e.call("Hello, World!").await?;
//! # Ok(())
//! # }
//! ```
//!

use futures::future::{ok, LocalBoxFuture, Ready};
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

/// This is a factory for `Pipe`. Since `Pipe` has chain connection,
/// it have to hold the previous `Pipe`. It would be provided in factory.
pub trait PipeFactory<M, P>
where
    P: Pipe<Self::Next>,
{
    /// data type that would send to previous pipe
    type Next;

    /// error type that would emit when processing pipe
    type Error;

    /// pipe type to build
    type Pipe: Pipe<M, Error = Self::Error>;

    /// initial error that would emit when building pipe
    type InitError;

    /// future when building pipe
    type Future: Future<Output = Result<Self::Pipe, Self::InitError>>;

    /// function to build a pipe
    fn new_pipe(&'static self, prev: P) -> Self::Future;
}

/// This is a pipe to send data easily using future
pub trait Pipe<M> {
    /// error when processing
    type Error;

    /// future when building pipe
    type Future: Future<Output = Result<(), Self::Error>>;

    fn call(&self, msg: M) -> Self::Future;
}

pub trait IntoPipe<P, M>
where
    P: Pipe<M>,
{
    fn into_pipe(self) -> P;
}

impl<P, M> IntoPipe<P, M> for P
where
    P: Pipe<M>,
{
    fn into_pipe(self) -> P {
        self
    }
}

pub struct FnPipe<F, M, Fut, Err>
where
    F: Fn(M) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    f: F,
    _data: PhantomData<fn(M)>,
}

impl<F, M, Fut, Err> FnPipe<F, M, Fut, Err>
where
    F: Fn(M) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    fn new(f: F) -> Self {
        Self {
            f,
            _data: PhantomData,
        }
    }
}

impl<F, M, Fut, Err> Clone for FnPipe<F, M, Fut, Err>
where
    F: Fn(M) -> Fut + Clone,
    Fut: Future<Output = Result<(), Err>>,
{
    fn clone(&self) -> Self {
        Self::new(self.f.clone())
    }
}

impl<F, M, Fut, Err> Pipe<M> for FnPipe<F, M, Fut, Err>
where
    F: Fn(M) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    type Error = Err;
    type Future = Fut;

    fn call(&self, msg: M) -> Self::Future {
        (self.f)(msg)
    }
}

impl<F, M, Fut, Err> IntoPipe<FnPipe<F, M, Fut, Err>, M> for F
where
    F: Fn(M) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    fn into_pipe(self) -> FnPipe<F, M, Fut, Err> {
        FnPipe::new(self)
    }
}

pub fn fn_pipe<F, M, Fut, Err>(f: F) -> FnPipe<F, M, Fut, Err>
where
    F: Fn(M) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    f.into_pipe()
}

pub trait IntoPipeFactory<PF, M, P>
where
    PF: PipeFactory<M, P>,
    P: Pipe<PF::Next>,
{
    fn into_pipe_factory(self) -> PF;
}

impl<PF, M, P> IntoPipeFactory<PF, M, P> for PF
where
    PF: PipeFactory<M, P>,
    P: Pipe<PF::Next>,
{
    fn into_pipe_factory(self) -> PF {
        self
    }
}

pub struct FnPipeFactory<'a, F, M1, M2, Fut, Err>
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
{
    f: F,
    _data: PhantomData<&'a fn(M1) -> M2>,
}

impl<'a, F, M1, M2, Fut, Err> FnPipeFactory<'a, F, M1, M2, Fut, Err>
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
{
    fn new(f: F) -> Self {
        Self {
            f,
            _data: PhantomData,
        }
    }
}

impl<'a, F, M1, M2, Fut, Err, P> PipeFactory<M1, P> for FnPipeFactory<'a, F, M1, M2, Fut, Err>
where
    F: Fn(M1) -> Fut,
    Fut: Future<Output = Result<M2, Err>>,
    P: Pipe<M2, Error = Err> + 'a,
{
    type Next = M2;
    type Error = Err;
    type Pipe = FnPipe<
        Box<dyn Fn(M1) -> LocalBoxFuture<'a, Result<(), Err>> + 'a>,
        M1,
        LocalBoxFuture<'a, Result<(), Err>>,
        Err,
    >;
    type InitError = Err;
    type Future = Ready<Result<Self::Pipe, Err>>;

    fn new_pipe(&'a self, prev: P) -> Self::Future {
        let prev = Arc::new(prev);
        ok(fn_pipe(Box::new(move |msg| {
            let prev_ = prev.clone();
            Box::pin(async move {
                prev_.call((self.f)(msg).await?).await?;
                Ok(())
            })
        })))
    }
}

impl<'a, F, M1, M2, Fut, Err, P> IntoPipeFactory<FnPipeFactory<'a, F, M1, M2, Fut, Err>, M1, P> for F
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
    P: Pipe<M2, Error = Err> + 'a,
{
    fn into_pipe_factory(self) -> FnPipeFactory<'a, F, M1, M2, Fut, Err> {
        FnPipeFactory::new(self)
    }
}
