use ::monkey::monkey::ExecMode;
use ::monkey::utils;
use clap::arg;
use clap::crate_version;
use clap::Parser;
use monkey::monkey;

/// monkey is the binary for executing the monkey programming language
#[derive(Debug, Parser)]
#[command(name="monkey", version=crate_version!(), about="monkey language", long_about = "Run monkey code")]
struct MonkeyCmd {
    /// Path
    #[arg(required = false, global = true)]
    path: Option<String>,
    /// Execution mode (vm or direct)
    #[arg(
        short = 'm',
        long = "mode",
        default_value = "vm",
        required = false,
        global = true
    )]
    mode: Option<ExecMode>,

    /// Enter interactive mode after executing 'script'
    #[arg(short = 'i', long = "interactive", required = false, global = true)]
    script: Option<String>,
}

fn main() {
    let args = MonkeyCmd::parse();

    match args.path {
        Some(path) => match utils::load_monkey(path) {
            Ok(contents) => match monkey::interpret_chunk(args.mode.unwrap(), contents, None, None)
            {
                Ok(_) => return,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        None => {}
    };

    // repl mode
    match args.script {
        Some(path) => match monkey::repl(Some(path), args.mode.unwrap()) {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
        None => match monkey::repl(None, args.mode.unwrap()) {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
    }
}
