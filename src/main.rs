use clap::arg;
use clap::crate_version;
use clap::Parser;
use monkey::repl;

// #[derive(Debug)]
// enum ExecMode {
//     VM,
//     Raw,
// }

/// monkey is the binary for executing the monkey programming language
#[derive(Debug, Parser)]
#[command(name="monkey", version=crate_version!(), about="monkey language", long_about = "Run monkey code")]
struct MonkeyCmd {
    /// Path
    #[arg(required = false, global = true)]
    path: Option<String>,
    /// Exec mode
    #[arg(short, long, required = false, global = true)]
    exec_mode: Option<String>,
}

// desired behavior, if it has no path => repl, if it has a path, read and run
// if it has exec mode, use that exec mode

fn main() {
    let args = MonkeyCmd::parse();

    // match args.exec_mode {
    //     Some(mode) => {}
    //     None => {
    //         // launch repl
    //     }
    // }

    match args.path {
        Some(path) => {
            repl::repl(Some(path));
        }
        None => {
            repl::repl(None);
        }
    }
}
