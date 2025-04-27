mod error;

use crate::types::{InitializeParams, RequestId};
use error::LspError;
use jsonrpc_core::{IoHandler, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

pub struct LspServer<R, W>
where
    R: Read,
    W: Write,
{
    reader: BufReader<R>,
    writer: BufWriter<W>,
    request_cancellations: HashMap<RequestId, bool>,
    io: IoHandler,
    initialized: bool,
}

impl<R, W> LspServer<R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(reader: R, writer: W) -> Self {
        LspServer {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
            request_cancellations: HashMap::new(),
            io: IoHandler::new(),
            initialized: false,
        }
    }

    pub fn setup_handlers(&mut self) {
        let io = &mut self.io;
        io.add_sync_method("say_hello", |_params| Ok(Value::String("hello".to_owned())));
    }

    pub fn initialize(&mut self, params: InitializeParams) -> Result<(), LspError> {
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), LspError> {
        let mut buffer = String::new();
        loop {
            let mut content_length: Option<usize> = None;
            loop {
                buffer.clear();
                let bytes_read = self.reader.read_line(&mut buffer)?;

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
                if line.starts_with("Content-Type: ")
                    && !line.contains("charset=utf-8")
                    && !line.contains("utf8")
                {
                    let error_response = r#"{"jsonrpc":"2.0","error":{"code":-32701,"message":"Unsupported encoding, only utf-8 is supported"},"id":null}"#;
                    let response_bytes = error_response.as_bytes();
                    write!(
                        self.writer,
                        "Content-Length: {}\r\n\r\n",
                        response_bytes.len()
                    )?;

                    self.writer.write_all(response_bytes)?;
                    self.writer.flush()?;
                    continue;
                }
            }

            if let Some(length) = content_length {
                let mut content = vec![0; length];
                self.reader.read_exact(&mut content)?;
                let request_str = String::from_utf8(content)?;
                eprintln!("Request: {}", request_str);

                let response = self.io.handle_request_sync(&request_str);

                if let Some(response_str) = response {
                    let response_bytes = response_str.as_bytes();
                    write!(
                        self.writer,
                        "Content-Length: {}\r\n\r\n",
                        response_bytes.len()
                    )?;
                    self.writer.write_all(response_bytes)?;
                    self.writer.flush()?;
                    eprintln!("\nSent response");
                }
            } else {
                eprintln!("No Content-Length header found, skipping")
            }
        }
    }
}
