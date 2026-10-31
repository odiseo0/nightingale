use serde_json::Value;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};

pub(crate) struct TestApi {
    pub url: String,
    handle: thread::JoinHandle<Vec<String>>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl TestApi {
    pub fn new(count: usize, mut reply: impl FnMut(&str) -> (u16, Value) + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let signal = stop.clone();
        let handle = thread::spawn(move || {
            let mut requests = Vec::new();

            for _ in 0..count {
                let deadline = Instant::now() + Duration::from_secs(5);
                let mut stream = loop {
                    if signal.load(std::sync::atomic::Ordering::Relaxed) {
                        return requests;
                    }
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && Instant::now() < deadline =>
                        {
                            thread::sleep(Duration::from_millis(5))
                        }
                        Err(_) => panic!("falta una petición al servidor de prueba"),
                    }
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(2)))
                    .unwrap();

                let mut bytes = Vec::new();

                loop {
                    let mut buffer = [0; 4096];
                    let length = stream.read(&mut buffer).unwrap();
                    assert!(length > 0 && bytes.len() + length < 1024 * 1024);
                    bytes.extend_from_slice(&buffer[..length]);

                    if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                        let header = String::from_utf8_lossy(&bytes[..end]).to_ascii_lowercase();
                        let body_length = header
                            .lines()
                            .find_map(|line| {
                                line.strip_prefix("content-length:")
                                    .and_then(|value| value.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);

                        if bytes.len() >= end + 4 + body_length {
                            break;
                        }
                    }
                }

                let request = String::from_utf8(bytes).unwrap();
                let (status, body) = reply(&request);

                if status != 0 {
                    let body = body.to_string();
                    let response = format!(
                        "HTTP/1.1 {status} Test\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    );
                    stream.write_all(response.as_bytes()).unwrap();
                }
                requests.push(request);
            }

            requests
        });

        Self { url, handle, stop }
    }

    pub fn dynamic(reply: impl FnMut(&str) -> (u16, Value) + Send + 'static) -> Self {
        Self::new(usize::MAX, reply)
    }

    pub fn finish(self) -> Vec<String> {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        self.handle.join().unwrap()
    }
}
