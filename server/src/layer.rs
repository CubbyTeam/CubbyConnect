//! This is a handler that can manipulate data through the handler.
//!
//! `Layer` helps you to build handler with previous handler.
//! You can only use `Handler` for start of the handler.
//!
//! # Examples
//!
//! ```
//! use cubby_connect_server::handler::Handler;
//! use cubby_connect_server::layer::Layer;
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
//! pub struct Echo<T, H>
//! where
//!     H: Handler<T>,
//! {
//!     prev: H,
//!     _marker: PhantomData<T>,
//! }
//!
//! impl<T, H> Layer<T, H> for EchoFactory
//! where
//!     H: Handler<T>,
//!     H::Future: 'static,
//! {
//!     type Next = T;
//!     type Error = P::Error;
//!     type Handler = Echo<T, H>;
//!     type InitError = ();
//!     type Future = Ready<Result<Self::Handler, Self::InitError>>;
//!
//!     fn new_handler(&self, prev: H) -> Self::Future {
//!         ok(Echo {
//!             prev,
//!             _marker: PhantomData::default(),
//!         })
//!     }
//! }
//!
//! impl<T, H> Handler<T> for Echo<T, H>
//! where
//!     H: Handler<T>,
//!     H::Future: 'static,
//! {
//!     type Error = H::Error;
//!     type Future = LocalBoxFuture<'static, Result<(), Self::Error>>;
//!
//!     fn call(&self, msg: T) -> Self::Future {
//!         let prev_call = self.prev.call(msg);
//!
//!         // this would act as same future of previous handler,
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
//! impl<S> Handler<S> for Print
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
//! // `e` would be the handler: `Echo` > `Print`
//! let e = ef.new_handler(p).await?;
//! // this would print "Hello, World!" to stdout
//! e.call("Hello, World!").await?;
//! # Ok(())
//! # }
//! ```

use std::future::Future;

use crate::handler::{Handler, IntoHandler};

/// This is a factory for `Handler`. Since `Handler` has chain connection,
/// it have to hold the previous `Pipe`. It would be provided in factory.
pub trait Layer<T, H>
where
    H: Handler<Self::Next>,
{
    /// data type that would send to previous handler
    type Next;

    /// error type that would emit when processing handler
    type Error;

    /// handler type to build
    type Handler: Handler<T, Error = Self::Error>;

    /// initial error that would emit when building handler
    type InitError;

    /// future when building handler
    type Future: Future<Output = Result<Self::Handler, Self::InitError>>;

    /// function to build a handler
    fn new_handler(&self, prev: H) -> Self::Future;
}

/// This trait can make into `Layer`
pub trait IntoLayer<L, T, H>
where
    L: Layer<T, H>,
    H: Handler<L::Next>,
{
    fn into_layer(self) -> L;
}

impl<L, T, H> IntoLayer<L, T, H> for L
where
    L: Layer<T, H>,
    H: Handler<L::Next>,
{
    /// `Layer` can be turn into `Layer` itself
    fn into_layer(self) -> L {
        self
    }
}

/// `Layer` and `Handler` connect function for simple use.
pub fn connect<IL, L, T, IH, H>(layer: IL, handler: IH) -> L::Future
where
    IL: IntoLayer<L, T, H>,
    L: Layer<T, H>,
    H: Handler<L::Next>,
    IH: IntoHandler<H, L::Next>,
{
    layer.into_layer().new_handler(handler.into_handler())
}

/// macro to use handler more simple
///
/// # Example
///
/// ```ignore
/// apply!(some_layer1, some_layer2, ... some_handler);
/// ```
#[macro_export]
macro_rules! apply {
    ($x:expr, $y:expr) => {
        $crate::layer::connect($x, $y).await?
    };
    ($x:expr, $($y:expr),+) => {
        $crate::layer::connect($x, apply!($( $y ),+)).await?
    };
}

#[cfg(test)]
mod test {
    use std::fmt::Display;
    use std::marker::PhantomData;

    use futures::future::{ok, LocalBoxFuture, Ready};
    use num_traits::PrimInt;

    use super::*;

    struct PlusOneFactory;

    struct PlusOne<T, H>
    where
        T: PrimInt,
        H: Handler<T>,
    {
        prev: H,
        _marker: PhantomData<T>,
    }

    impl<T, H> Layer<T, H> for PlusOneFactory
    where
        T: PrimInt,
        H: Handler<T>,
        H::Future: 'static,
    {
        type Next = T;
        type Error = H::Error;
        type Handler = PlusOne<T, H>;
        type InitError = ();
        type Future = Ready<Result<Self::Handler, ()>>;

        fn new_handler(&self, prev: H) -> Self::Future {
            ok(PlusOne {
                prev,
                _marker: PhantomData,
            })
        }
    }

    impl<T, H> Handler<T> for PlusOne<T, H>
    where
        T: PrimInt,
        H: Handler<T>,
        H::Future: 'static,
    {
        type Error = H::Error;
        type Future = LocalBoxFuture<'static, Result<(), Self::Error>>;

        fn call(&self, msg: T) -> Self::Future {
            let prev = self.prev.call(msg.add(T::one()));

            Box::pin(async move {
                prev.await?;
                Ok(())
            })
        }
    }

    struct Check {
        check: String,
    }

    impl Check {
        fn new<S: AsRef<str>>(s: S) -> Check {
            Check {
                check: s.as_ref().to_string(),
            }
        }
    }

    impl<T: Display> Handler<T> for Check {
        type Error = ();
        type Future = Ready<Result<(), ()>>;

        fn call(&self, msg: T) -> Self::Future {
            assert_eq!(msg.to_string(), self.check);
            ok(())
        }
    }

    #[tokio::test]
    async fn plus_one_test() -> Result<(), ()> {
        let handler = PlusOneFactory.new_handler(Check::new("2")).await?;
        handler.call(1).await?;
        Ok(())
    }

    #[tokio::test]
    async fn plus_multi_times_test() -> Result<(), ()> {
        let handler = PlusOneFactory
            .new_handler(
                PlusOneFactory
                    .new_handler(PlusOneFactory.new_handler(Check::new("8")).await?)
                    .await?,
            )
            .await?;
        handler.call(5).await?;
        Ok(())
    }

    #[tokio::test]
    async fn connect_plus_one_test() -> Result<(), ()> {
        let handler = connect(PlusOneFactory, Check::new("1")).await?;
        handler.call(0).await?;
        Ok(())
    }

    #[tokio::test]
    async fn connect_plus_multi_times_test() -> Result<(), ()> {
        let handler = connect(
            PlusOneFactory,
            connect(
                PlusOneFactory,
                connect(PlusOneFactory, Check::new("7")).await?,
            )
            .await?,
        )
        .await?;
        handler.call(4).await?;
        Ok(())
    }

    #[tokio::test]
    async fn handler_macro_test() -> Result<(), ()> {
        let handler = apply!(
            PlusOneFactory,
            PlusOneFactory,
            PlusOneFactory,
            Check::new("6")
        );
        handler.call(3).await?;
        Ok(())
    }
}
