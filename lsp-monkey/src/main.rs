use anyhow::{Context, Result};
use jsonrpc_core::{IoHandler, Value};
use std::io::{BufRead, BufReader, Read, Write};

fn main() -> Result<()> {
    let mut io = IoHandler::new();
    io.add_sync_method("say_hello", |_params| Ok(Value::String("hello".to_owned())));

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();

    loop {
        let mut content_length: Option<usize> = None;
        loop {
            buffer.clear();
            let bytes_read = reader
                .read_line(&mut buffer)
                .context("Failed to read line")?;

            if bytes_read == 0 {
                return Ok(());
            }

            let length_prefix = "Content-Length: ";
            let line = buffer.trim_end();
            if line.is_empty() {
                break;
            }

            if line.starts_with("Content-Length: ") {
                let content_length_str = buffer[length_prefix.len()..].trim_end();
                match content_length_str.parse::<usize>() {
                    Ok(len) => {
                        content_length = Some(len);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse content length: {}", e);
                    }
                }
            }
            // if line.starts_with("Content-Type: ") {
            //     let content_mime_type = buffer[length_prefix.len()..].trim_end();
            // }
        }

        if let Some(length) = content_length {
            let mut content = vec![0; length];
            reader
                .read_exact(&mut content)
                .context("Failed to read request content")?;
            let request_str =
                String::from_utf8(content).context("Failed to parse request to utf8 string")?;
            eprintln!("Request: {}", request_str);

            let response = io.handle_request_sync(&request_str);

            if let Some(response_str) = response {
                let response_bytes = response_str.as_bytes();
                write!(stdout, "Content-Length: {}\r\n\r\n", response_bytes.len())
                    .context("Failed to write response length to stdout")?;
                stdout
                    .write_all(response_bytes)
                    .context("Failed to write response to stdout")?;
                stdout.flush().context("Failed to flush stdout")?;
                eprintln!("\nSent response");
            }
        } else {
            eprintln!("No Content-Length header found, skipping")
        }
    }
}
