//! Function adapter for `Handler`
//!
//! # Examples
//!
//! ```
//! use cubby_connect_server::fn_handler::fn_handler;
//! use cubby_connect_server::handler::Handler;
//! use std::fmt::Display;
//!
//! async fn hello<S: Display>(s: S) -> Result<(), ()> {
//!     println!("Hello {s}");
//!     Ok(())
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), ()> {
//! let handler = fn_handler(hello);
//! // it would print "Hello World"
//! handler.call("World");
//! # Ok(())
//! # }
//! ```

use std::future::Future;
use std::marker::PhantomData;

use crate::handler::{Handler, IntoHandler};

/// `Handler` for closures/functions for simple definition of use.
/// The type of function would be as: `async fn<T>(T) -> Result<(), Err>`
pub struct FnHandler<F, T, Fut, Err>
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<F, T, Fut, Err> FnHandler<F, T, Fut, Err>
where
    F: Fn(T) -> Fut,
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
impl<F, T, Fut, Err> Handler<T> for FnHandler<F, T, Fut, Err>
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    type Error = Err;
    type Future = Fut;

    fn call(&self, msg: T) -> Self::Future {
        (self.f)(msg)
    }
}

impl<F, T, Fut, Err> IntoHandler<FnHandler<F, T, Fut, Err>, T> for F
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    fn into_handler(self) -> FnHandler<F, T, Fut, Err> {
        FnHandler::new(self)
    }
}

/// public function wrapper of `FnPipe`
/// use this to change function into `Pipe`
pub fn fn_handler<F, T, Fut, Err>(f: F) -> FnHandler<F, T, Fut, Err>
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = Result<(), Err>>,
{
    FnHandler::new(f)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn fn_handler_test() -> Result<(), ()> {
        async fn hello<S: AsRef<str>>(name: S) -> Result<(), ()> {
            let name = name.as_ref();
            if name == "None" {
                Err(())
            } else {
                println!("Hello, {name}");
                Ok(())
            }
        }

        fn_handler(hello).call("World").await?;
        assert!(fn_handler(hello).call("None").await.is_err());

        Ok(())
    }
}
