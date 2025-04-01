use anyhow::{Context, Result};
use lsp_monkey::server::LspServer;

fn main() -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut server = LspServer::new(stdin.lock(), stdout.lock());
    server.run().context("LSP server failed to run")
}
