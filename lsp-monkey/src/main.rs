use jsonrpc_core::IoHandler;
use jsonrpc_core::Value;
use jsonrpc_stdio_server::ServerBuilder;

#[tokio::main]
async fn main() {
    let mut io = IoHandler::new();
    io.add_sync_method("say_hello", |_| Ok(Value::String("hello".to_string())));
    let server = ServerBuilder::new(io).build();
    server.await;
}
