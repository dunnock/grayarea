// reusable handler of result which should never be given for select!
#[macro_export]
macro_rules! should_not_complete {
    ( $text:expr, $res:expr ) => {
        match $res {
            Ok(_) => {
                info!("All the {} completed", $text);
                Err(anyhow!("All the {} exit", $text))
            }
            Err(err) => {
                error!("{} failure: {}", $text, err);
                Err(anyhow::Error::from(err))
            }
        }
    };
}
