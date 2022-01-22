//! This is a handler trait to handle asynchronously
//!
//! # Examples
//!
//! ```
//! use cubby_connect_server::handler::Handler;
//! use futures::future::{ok, Ready};
//! use std::fmt::Display;
//!
//! struct Hello;
//!
//! impl<S: Display> Handler<S> for Hello {
//!     type Error = ();
//!     type Future = Ready<Result<(), ()>>;
//!
//!     fn call(&self, msg: S) -> Self::Future {
//!         println!("Hello {}", msg);
//!         ok(())
//!     }
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), ()> {
//! let handler = Hello;
//! // this would print "Hello World"
//! handler.call("World");
//! # Ok(())
//! # }
//! ```

use std::future::Future;

/// This is a handler to send data easily using future
pub trait Handler<T> {
    /// error when processing
    type Error;

    /// future when building handler
    type Future: Future<Output = Result<(), Self::Error>>;

    fn call(&self, msg: T) -> Self::Future;
}

/// This is a trait that can make into `Handler`
pub trait IntoHandler<H, T>
where
    H: Handler<T>,
{
    fn into_handler(self) -> H;
}

impl<H, T> IntoHandler<H, T> for H
where
    H: Handler<T>,
{
    /// `Handler` can be turn into `Handler` itself
    fn into_handler(self) -> H {
        self
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use futures::future::{ok, Ready};

    use super::*;

    struct Check(String);

    impl<S: Display> Handler<S> for Check {
        type Error = ();
        type Future = Ready<Result<(), ()>>;

        fn call(&self, msg: S) -> Self::Future {
            assert_eq!(msg.to_string(), self.0);
            ok(())
        }
    }

    #[tokio::test]
    async fn handler_test() -> Result<(), ()> {
        let handler = Check("hello".to_string());
        handler.call("hello").await?;
        Ok(())
    }
}
