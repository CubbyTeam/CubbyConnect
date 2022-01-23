#[cfg(test)]
mod fn_layer_test {
    use num_traits::PrimInt;

    use cubby_connect_server_core::handler::Handler;
    use cubby_connect_server_macro::apply;

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
        let handler = apply!(plus_one to check);
        handler.call(1).await?;
        Ok(())
    }

    #[tokio::test]
    async fn plus_multi_times_test() -> Result<(), ()> {
        make_check!("5");
        let handler = apply!(plus_one, plus_one, plus_one to check);
        handler.call(2).await?;
        Ok(())
    }
}
