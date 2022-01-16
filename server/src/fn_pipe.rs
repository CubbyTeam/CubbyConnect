//! Function adapter for `Pipe` and `PipeFactory`
//!
//! # Examples
//!
//! ```
//! use cubby_connect_server::fn_pipe::{fn_pipe, fn_pipe_factory};
//! use cubby_connect_server::pipe;
//! use cubby_connect_server::pipe::{connect, Pipe, PipeFactory};
//! use std::fmt::Display;
//!
//! async fn echo<T>(t: T) -> Result<T, ()> {
//!     Ok(t)
//! }
//!
//! async fn print<T: Display>(t: T) -> Result<(), ()> {
//!     assert_eq!(t.to_string(), "Hello, World!");
//!     print!("{}", t);
//!     Ok(())
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), ()> {
//! let p = fn_pipe(print);
//! let ef = fn_pipe_factory(echo);
//! // `e` would be the pipe: `Echo` > `Print`
//! let e = ef.new_pipe(p).await?;
//! // this would print "Hello, World" to stdout
//! e.call("Hello, World!").await?;
//!
//! // or
//!
//! let e = connect(fn_pipe_factory(echo), fn_pipe(print)).await?;
//! e.call("Hello, World!").await?;
//!
//! // or
//!
//! let e = pipe!(echo, print);
//! e.call("Hello, World!").await?;
//! # Ok(())
//! # }
//! ```

use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use futures::future::{ok, LocalBoxFuture, Ready};

use crate::pipe::{IntoPipe, IntoPipeFactory, Pipe, PipeFactory};

/// `Pipe` for closures/functions for simple definition of use.
/// The type of function would be as: `async fn<T>(T) -> Result<(), Err>`
pub struct FnPipe<F, M, Fut, Err>
where
    F: Fn(M) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    f: F,
    _marker: PhantomData<fn(M)>,
}

impl<F, M, Fut, Err> FnPipe<F, M, Fut, Err>
where
    F: Fn(M) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    fn new(f: F) -> Self {
        Self {
            f,
            _marker: PhantomData,
        }
    }
}

/// This would simply call the function
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

/// public function wrapper of `FnPipe`
/// use this to change function into `Pipe`
pub fn fn_pipe<F, M, Fut, Err>(f: F) -> FnPipe<F, M, Fut, Err>
where
    F: Fn(M) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    FnPipe::new(f)
}

/// `PipeFactory` for closures/functions for simple definition of use.
/// The type of function would be as: `async fn<T, U>(T) -> Result<U, Err>`
/// This would be connected to other `Pipe` as: `async fn<U>(U) -> Result<(), Err>`
/// It would be easier to know the data flow.
///
/// The lifetime is same as the closure.
///
/// *a little overhead due to lifetime problem*
/// function should go into `Arc` because it is multi-thread
pub struct FnPipeFactory<'a, F, M1, M2, Fut, Err>
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
{
    f: Arc<F>,
    _marker: PhantomData<&'a fn(M1) -> M2>,
}

impl<'a, F, M1, M2, Fut, Err> FnPipeFactory<'a, F, M1, M2, Fut, Err>
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
{
    fn new(f: F) -> Self {
        Self {
            f: Arc::new(f),
            _marker: PhantomData,
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
    #[allow(clippy::type_complexity)]
    type Pipe = FnPipe<
        Box<dyn Fn(M1) -> LocalBoxFuture<'a, Result<(), Err>> + 'a>,
        M1,
        LocalBoxFuture<'a, Result<(), Err>>,
        Err,
    >;
    type InitError = Err;
    type Future = Ready<Result<Self::Pipe, Err>>;

    fn new_pipe(&self, prev: P) -> Self::Future {
        // a little overhead due to lifetime problem
        // -> `prev` is captured in closure but it cannot be borrowed into async
        //    block because closure's lifetime cannot be set.
        // this should go into `Arc` because we are running this in multi-thread
        // TODO: think of a better way (maybe unsafe?)
        let prev = Arc::new(prev);
        let f = self.f.clone();

        ok(fn_pipe(Box::new(move |msg| {
            let prev_ = prev.clone();
            let f_ = f.clone();
            Box::pin(async move {
                prev_.call(f_(msg).await?).await?;
                Ok(())
            })
        })))
    }
}

impl<'a, F, M1, M2, Fut, Err, P> IntoPipeFactory<FnPipeFactory<'a, F, M1, M2, Fut, Err>, M1, P>
    for F
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
    P: Pipe<M2, Error = Err> + 'a,
{
    fn into_pipe_factory(self) -> FnPipeFactory<'a, F, M1, M2, Fut, Err> {
        FnPipeFactory::new(self)
    }
}

/// public function wrapper of `FnPipeFactory`
/// use this to change function to `PipeFactory`
pub fn fn_pipe_factory<'a, F, M1, M2, Fut, Err>(f: F) -> FnPipeFactory<'a, F, M1, M2, Fut, Err>
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
{
    FnPipeFactory::new(f)
}
