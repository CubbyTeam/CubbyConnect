//! Function adapter for `Layer`
//!
//! # Examples
//!
//! ```
//! use cubby_connect_server::apply;
//! use cubby_connect_server::fn_handler::fn_handler;
//! use cubby_connect_server::fn_layer::fn_layer;
//! use cubby_connect_server::handler::{self, Handler};
//! use cubby_connect_server::layer::{connect, Layer};
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
//! let p = fn_handler(print);
//! let ef = fn_layer(echo);
//! // `e` would be the pipe: `Echo` > `Print`
//! let e = ef.new_handler(p).await?;
//! // this would print "Hello, World" to stdout
//! e.call("Hello, World!").await?;
//!
//! // or
//!
//! let e = connect(fn_layer(echo), fn_handler(print)).await?;
//! e.call("Hello, World!").await?;
//!
//! // or
//!
//! let e = apply!(echo, print);
//! e.call("Hello, World!").await?;
//! # Ok(())
//! # }
//! ```

use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use futures::future::{ok, LocalBoxFuture, Ready};

use crate::fn_handler::{fn_handler, FnHandler};
use crate::handler::Handler;
use crate::layer::{IntoLayer, Layer};

/// `PipeFactory` for closures/functions for simple definition of use.
/// The type of function would be as: `async fn<T, U>(T) -> Result<U, Err>`
/// This would be connected to other `Pipe` as: `async fn<U>(U) -> Result<(), Err>`
/// It would be easier to know the data flow.
///
/// The lifetime is same as the closure.
///
/// *a little overhead due to lifetime problem*
/// function should go into `Arc` because it is multi-thread
pub struct FnLayer<'a, F, M1, M2, Fut, Err>
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
{
    f: Arc<F>,
    _marker: PhantomData<&'a fn(M1) -> M2>,
}

impl<'a, F, M1, M2, Fut, Err> FnLayer<'a, F, M1, M2, Fut, Err>
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

impl<'a, F, M1, M2, Fut, Err, P> Layer<M1, P> for FnLayer<'a, F, M1, M2, Fut, Err>
where
    F: Fn(M1) -> Fut,
    Fut: Future<Output = Result<M2, Err>>,
    P: Handler<M2, Error = Err> + 'a,
{
    type Next = M2;
    type Error = Err;
    #[allow(clippy::type_complexity)]
    type Handler = FnHandler<
        Box<dyn Fn(M1) -> LocalBoxFuture<'a, Result<(), Err>> + 'a>,
        M1,
        LocalBoxFuture<'a, Result<(), Err>>,
        Err,
    >;
    type InitError = Err;
    type Future = Ready<Result<Self::Handler, Err>>;

    fn new_handler(&self, prev: P) -> Self::Future {
        // a little overhead due to lifetime problem
        // -> `prev` is captured in closure but it cannot be borrowed into async
        //    block because closure's lifetime cannot be set.
        // this should go into `Arc` because we are running this in multi-thread
        // TODO: think of a better way (maybe unsafe?)
        let prev = Arc::new(prev);
        let f = self.f.clone();

        ok(fn_handler(Box::new(move |msg| {
            let prev_ = prev.clone();
            let f_ = f.clone();
            Box::pin(async move {
                prev_.call(f_(msg).await?).await?;
                Ok(())
            })
        })))
    }
}

impl<'a, F, M1, M2, Fut, Err, P> IntoLayer<FnLayer<'a, F, M1, M2, Fut, Err>, M1, P> for F
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
    P: Handler<M2, Error = Err> + 'a,
{
    fn into_layer(self) -> FnLayer<'a, F, M1, M2, Fut, Err> {
        FnLayer::new(self)
    }
}

/// public function wrapper of `FnPipeFactory`
/// use this to change function to `PipeFactory`
pub fn fn_layer<'a, F, M1, M2, Fut, Err>(f: F) -> FnLayer<'a, F, M1, M2, Fut, Err>
where
    F: Fn(M1) -> Fut + 'a,
    Fut: Future<Output = Result<M2, Err>>,
{
    FnLayer::new(f)
}

#[cfg(test)]
mod test {
    use num_traits::PrimInt;

    use crate::apply;

    use super::*;

    async fn plus_one<I: PrimInt>(i: I) -> Result<I, ()> {
        Ok(i.add(I::one()))
    }

    macro_rules! make_check {
        ($check:expr) => {
            use std::fmt::Display;

            async fn check<S: Display>(s: S) -> Result<(), ()> {
                assert_eq!(s.to_string(), $check);
                Ok(())
            }
        };
    }

    #[tokio::test]
    async fn plus_one_test() -> Result<(), ()> {
        make_check!("2");
        let pipe = apply!(plus_one, check);
        pipe.call(1).await?;
        Ok(())
    }

    #[tokio::test]
    async fn plus_multi_times_test() -> Result<(), ()> {
        make_check!("5");
        let pipe = apply!(plus_one, plus_one, plus_one, check);
        pipe.call(2).await?;
        Ok(())
    }

    #[tokio::test]
    async fn plus_a_lot_of_times_test() -> Result<(), ()> {
        make_check!("15");
        let handler = apply!(
            plus_one, plus_one, plus_one, plus_one, plus_one, plus_one, plus_one, plus_one,
            plus_one, plus_one, plus_one, plus_one, plus_one, check
        );
        handler.call(2).await?;
        Ok(())
    }
}