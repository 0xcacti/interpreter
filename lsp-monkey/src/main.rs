use jsonrpc_core::{IoHandler, Value};
use std::io::{BufRead, BufReader, Read, Write};

fn main() {
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
            let bytes_read = reader.read_line(&mut buffer).unwrap();

            if bytes_read == 0 {
                return; // TODO what to do here
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
            match reader.read_exact(&mut content) {
                Ok(_) => {
                    let request_str = String::from_utf8(content).unwrap();
                    eprintln!("Request: {}", request_str);

                    let response = io.handle_request_sync(&request_str);

                    if let Some(response_str) = response {
                        let response_bytes = response_str.as_bytes();
                        write!(stdout, "Content-Length: {}\r\n\r\n", response_bytes.len()).unwrap();
                        stdout.write_all(response_bytes).unwrap();
                        stdout.flush().unwrap();
                        eprintln!("\nSent response");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read request content: {}", e);
                    return;
                }
            }
        } else {
            eprintln!("No Content-Length header found, skipping")
        }
    }
}
