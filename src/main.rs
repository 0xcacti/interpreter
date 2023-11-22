/// bible-rs is a command line tool for getting a random verse from the Bible.
#[derive(Debug, Parser)]
#[command(name="bible-rs", version=crate_version!(), about="daily bread", long_about = "Get a random verse from the Bible.", arg_required_else_help(true))]
struct BibleParser {
    /// The subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
    /// The version of the Bible to use
    #[arg(short, long, required = false, global = true)]
    bible_version: Option<String>,
    /// The API key to use
    #[arg(short, long, required = false, global = true)]
    api_key: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {}

fn main() {
    println!("hello world");
}
