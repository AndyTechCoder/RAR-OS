#![deny(unsafe_code)]

#[path = "lib.rs"]
mod rarbuild;

use std::env;
use std::process::ExitCode;

use rarbuild::{Route, classify_route, execute_host_command, refusal_outcome};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match classify_route(&arguments) {
        Route::RefuseExecution(reason) => {
            let outcome = refusal_outcome(reason);
            print!("{}", outcome.output);
            ExitCode::from(outcome.exit_code as u8)
        }
        Route::Invalid(reason) => {
            eprintln!("rarbuild-invalid-v1\nreason={reason}");
            ExitCode::from(64)
        }
        Route::Host(command) => {
            let root = match env::current_dir() {
                Ok(root) => root,
                Err(error) => {
                    eprintln!("rarbuild-root-error: {error}");
                    return ExitCode::from(2);
                }
            };
            match execute_host_command(&root, command) {
                Ok(outcome) => {
                    print!("{}", outcome.output);
                    ExitCode::from(outcome.exit_code as u8)
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(2)
                }
            }
        }
    }
}
