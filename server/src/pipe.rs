//! This is a pipe that can manipulate data through the pipe.
//!
//! `PipeFactory` helps you to build pipe with previous pipe.
//! You can only use `Pipe` for start of the pipe.
//!
//! # Examples
//!
//! ```
//! use cubby_connect_server::pipe::{Pipe, PipeFactory};
//! use futures::future::{ok, LocalBoxFuture, Ready};
//! use std::fmt::Display;
//! use std::future::Future;
//! use std::marker::PhantomData;
//! use std::task::{Context, Poll};
//!
//! // Factory of Echo.
//! pub struct EchoFactory;
//!
//! // Pipe that sends the message to next as is
//! pub struct Echo<T, P>
//! where
//!     P: Pipe<T>,
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
//!             _marker: PhantomData::default(),
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
//!     S: Display,
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

use std::future::Future;

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
    fn new_pipe(&self, prev: P) -> Self::Future;
}

/// This is a pipe to send data easily using future
pub trait Pipe<M> {
    /// error when processing
    type Error;

    /// future when building pipe
    type Future: Future<Output = Result<(), Self::Error>>;

    fn call(&self, msg: M) -> Self::Future;
}

/// This is a trait that can make into `Pipe`
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
    /// `Pipe` can be turn into `Pipe` itself
    fn into_pipe(self) -> P {
        self
    }
}

/// This trait can make into `PipeFactory`
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
    /// `PipeFactory` can be turn into `PipeFactory` itself
    fn into_pipe_factory(self) -> PF {
        self
    }
}

/// `PipeFactory` and `Pipe` connect function for simple use.
pub fn connect<IPF, PF, M, IP, P>(fac: IPF, pipe: IP) -> PF::Future
where
    IPF: IntoPipeFactory<PF, M, P>,
    PF: PipeFactory<M, P>,
    P: Pipe<PF::Next>,
    IP: IntoPipe<P, PF::Next>,
{
    fac.into_pipe_factory().new_pipe(pipe.into_pipe())
}

/// macro to use pipe more simple
///
/// # Example
///
/// ```ignore
/// pipe!(some_pipe_factory1, some_pipe_factory2, ..., some_pipe);
/// ```
#[macro_export]
macro_rules! pipe {
    ($x:expr, $y:expr) => {
        connect($x, $y).await?
    };
    ($x:expr, $($y:expr),+) => {
        connect($x, pipe!($( $y ),+)).await?
    };
}
