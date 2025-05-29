pub mod app_logger {

    use crate::cli_arguments::cli_args::CliArgsTrait;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Root};
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::Config;

    pub fn configure_logger(cli_args: &impl CliArgsTrait) {
        // https://medium.com/nerd-for-tech/logging-in-rust-e529c241f92e
        // https://tms-dev-blog.com/log-to-a-file-in-rust-with-log4rs/
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{h({d(%Y-%m-%d %H:%M:%S)(local)} - {l}: {m}{n})}",
            )))
            .build();
        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(
                Root::builder().appender("stdout").build(
                    cli_args
                        .get_verbose()
                        .log_level()
                        .expect("Verbosity should be convertible to LevelFilter")
                        .to_level_filter(),
                ),
            )
            .unwrap();
        let _handle = log4rs::init_config(config).unwrap();
    }
}
