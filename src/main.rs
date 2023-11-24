use clap::arg;
use clap::crate_version;
use clap::Parser;
use monkey::monkey::repl;

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
    /// Execution mode (vm or raw)
    #[arg(short = 'm', long = "mode", required = false, global = true)]
    mode: Option<String>,

    /// Enter interactive mode after executing 'script'
    #[arg(short = 'i', long = "interactive", required = false, global = true)]
    script: Option<String>,
}

// desired behavior, if it has no path => repl, if it has a path, read and run
// if it has exec mode, use that exec mode

fn main() {
    let args = MonkeyCmd::parse();

    match args.path {
        Some(path) => {}
        None => {}
    }

    // match args.mode {
    //     Some(mode) => {}
    //     None => {
    //         // launch repl
    //     }
    // }

    // repl mode
    match args.script {
        Some(path) => match repl(Some(path)) {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
        None => match repl(None) {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
    }
}
