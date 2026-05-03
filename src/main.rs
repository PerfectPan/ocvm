use std::process::ExitCode;

fn main() -> ExitCode {
    match ocvm::cli::run(std::env::args_os()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
