#[cfg(test)]
mod layer_test {
    use std::fmt::Display;
    use std::marker::PhantomData;

    use futures::future::{ok, LocalBoxFuture, Ready};
    use num_traits::PrimInt;

    use cubby_connect_server_core::handler::Handler;
    use cubby_connect_server_core::layer::Layer;
    use cubby_connect_server_macro::apply;

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
    async fn handler_macro_test() -> Result<(), ()> {
        let handler = apply!(PlusOneFactory, PlusOneFactory, PlusOneFactory to Check::new("6"));
        handler.call(3).await?;
        Ok(())
    }
}
