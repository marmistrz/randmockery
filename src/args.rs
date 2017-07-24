use clap::{App, Arg};

const APP: &'static str = env!("CARGO_PKG_NAME");

pub fn get_parser<'a, 'b>() -> App<'a, 'b> {
    App::new(APP)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Proof-of-concept that we can mock randomness")
        .arg(
            Arg::with_name("library")
                .short("l")
                .required(false)
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
        )
        .arg(Arg::with_name("command").multiple(true).required(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_command() {
        let matches =
            get_parser().get_matches_from(&["./randmockery", "grep", "bash", "/etc/passwd"]);

        assert_eq!(
            matches.values_of("command").unwrap().collect::<Vec<_>>(),
            ["grep", "bash", "/etc/passwd"]
        );
    }

    #[test]
    fn test_mutliple_libs() {
        let matches =
            get_parser().get_matches_from(&["./randmockery", "-l", "xd", "-l", "xxd", "echo"]);

        assert_eq!(
            matches.values_of("command").unwrap().collect::<Vec<_>>(),
            ["echo"]
        );
        assert_eq!(
            matches.values_of("library").unwrap().collect::<Vec<_>>(),
            ["xd", "xxd"]
        );
    }
}
