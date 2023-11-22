/// monkey is the binary for executing the monkey programming language
#[derive(Debug, Parser)]
#[command(name="monkey", version=crate_version!(), about="monkey language", long_about = "Run monkey code", arg_required_else_help(true))]
struct MonkeyCmd {
    /// Exec mode
    #[arg(short, long, required = false, global = true)]
    exec_mode: Option<String>,
}

fn main() {
    println!("hello world");
    let args = MonkeyCmd::parse();
}
