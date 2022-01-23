//! Function adapter for `Layer`
//!
//! # Examples
//!
//! ```
//! use cubby_connect_server_core::fn_handler::fn_handler;
//! use cubby_connect_server_core::fn_layer::fn_layer;
//! use cubby_connect_server_core::handler::{self, Handler};
//! use cubby_connect_server_core::layer::{connect, Layer};
//! use cubby_connect_server_core::apply;
//! use std::fmt::Display;
//!
//! async fn echo<T>(t: T) -> Result<T, ()> {
//!     Ok(t)
//! }
//!
//! async fn print<T: Display>(t: T) -> Result<(), ()> {
//!     assert_eq!(t.to_string(), "Hello, World!");
//!     print!("{t}");
//!     Ok(())
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), ()> {
//! let p = fn_handler(print);
//! let ef = fn_layer(echo);
//! // `e` would be the handler: `Echo` > `Print`
//! let e = ef.new_handler(p).await?;
//! // this would print "Hello, World" to stdout
//! e.call("Hello, World!").await?;
//!
//! // or
//!
//! let e = connect(echo, print).await?;
//! e.call("Hello, World!").await?;
//!
//! // or
//!  
//! let e = apply!(echo to print);
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
pub struct FnLayer<'a, F, T1, T2, Fut, Err>
where
    F: Fn(T1) -> Fut + 'a,
    Fut: Future<Output = Result<T2, Err>>,
{
    f: Arc<F>,
    _marker: PhantomData<&'a fn(T1) -> T2>,
}

impl<'a, F, T1, T2, Fut, Err> FnLayer<'a, F, T1, T2, Fut, Err>
where
    F: Fn(T1) -> Fut + 'a,
    Fut: Future<Output = Result<T2, Err>>,
{
    fn new(f: F) -> Self {
        Self {
            f: Arc::new(f),
            _marker: PhantomData,
        }
    }
}

impl<'a, F, T1, T2, Fut, Err, H> Layer<T1, H> for FnLayer<'a, F, T1, T2, Fut, Err>
where
    F: Fn(T1) -> Fut,
    Fut: Future<Output = Result<T2, Err>>,
    H: Handler<T2, Error = Err> + 'a,
{
    type Next = T2;
    type Error = Err;
    #[allow(clippy::type_complexity)]
    type Handler = FnHandler<
        Box<dyn Fn(T1) -> LocalBoxFuture<'a, Result<(), Err>> + 'a>,
        T1,
        LocalBoxFuture<'a, Result<(), Err>>,
        Err,
    >;
    type InitError = Err;
    type Future = Ready<Result<Self::Handler, Err>>;

    fn new_handler(&self, prev: H) -> Self::Future {
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

impl<'a, F, T1, T2, Fut, Err, H> IntoLayer<FnLayer<'a, F, T1, T2, Fut, Err>, T1, H> for F
where
    F: Fn(T1) -> Fut + 'a,
    Fut: Future<Output = Result<T2, Err>>,
    H: Handler<T2, Error = Err> + 'a,
{
    fn into_layer(self) -> FnLayer<'a, F, T1, T2, Fut, Err> {
        FnLayer::new(self)
    }
}

/// public function wrapper of `FnPipeFactory`
/// use this to change function to `PipeFactory`
pub fn fn_layer<'a, F, T1, T2, Fut, Err>(f: F) -> FnLayer<'a, F, T1, T2, Fut, Err>
where
    F: Fn(T1) -> Fut + 'a,
    Fut: Future<Output = Result<T2, Err>>,
{
    FnLayer::new(f)
}

#[cfg(test)]
mod test {
    use num_traits::PrimInt;

    use crate::layer::connect;

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
        let handler = connect(plus_one, check).await?;
        handler.call(1).await?;
        Ok(())
    }

    #[tokio::test]
    async fn plus_multi_times_test() -> Result<(), ()> {
        make_check!("5");
        let handler = connect(
            plus_one,
            connect(plus_one, connect(plus_one, check).await?).await?,
        )
        .await?;
        handler.call(2).await?;
        Ok(())
    }
}
