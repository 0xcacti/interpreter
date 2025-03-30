use jsonrpc_core::{IoHandler, Value};
use std::io::{BufRead, BufReader, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the JSON-RPC handler
    let mut io = IoHandler::new();

    io.add_sync_method("say_hello", |_params| Ok(Value::String("hello".to_owned())));

    // Main message loop using stdio
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();

    loop {
        // Read the Content-Length header
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer)?;

        if bytes_read == 0 {
            break; // EOF reached
        }

        // Debug print to see what we're receiving
        eprintln!("Read header: {:?}", buffer);

        // Parse the Content-Length header
        let length_prefix = "Content-Length: ";
        if !buffer.starts_with(length_prefix) {
            eprintln!("Invalid header format, skipping");
            continue;
        }

        // Carefully trim and parse, handling potential CRLF issues
        let content_length_str = buffer[length_prefix.len()..].trim_end();
        eprintln!("Length string: {:?}", content_length_str);

        let content_length: usize = match content_length_str.parse() {
            Ok(len) => len,
            Err(e) => {
                eprintln!("Failed to parse content length: {}", e);
                continue;
            }
        };

        // Skip the empty line after headers
        buffer.clear();
        reader.read_line(&mut buffer)?;

        // Read the JSON content
        let mut content = vec![0; content_length];
        reader.read_exact(&mut content)?;

        // Parse and handle the JSON-RPC request
        let request_str = String::from_utf8(content)?;
        eprintln!("Request: {}", request_str);

        let response = io.handle_request_sync(&request_str);

        if let Some(response_str) = response {
            // Write the response with proper headers
            let response_bytes = response_str.as_bytes();
            write!(stdout, "Content-Length: {}\r\n\r\n", response_bytes.len())?;
            stdout.write_all(response_bytes)?;
            stdout.flush()?;

            eprintln!("Sent response");
        }
    }

    Ok(())
}
