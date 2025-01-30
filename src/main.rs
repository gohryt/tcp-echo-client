use argh::FromArgs;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(FromArgs, Debug)]
/// tcp-echo-client
struct Args {
    /// address
    #[argh(option, short = 'a', default = "\"127.0.0.1:8080\".to_string()")]
    address: String,
}

#[tokio::main]
async fn main() {
    let Args { address } = argh::from_env();

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let mut stream = TcpStream::connect(&address).await.unwrap();
    stream.set_nodelay(true).unwrap();

    println!("Connected to {}", address);
    println!("Type messages and press Enter to send. Type 'quit' to exit.");

    loop {
        let input = match stdin.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) | Err(_) => break,
        };

        if input.trim().eq_ignore_ascii_case("quit") {
            break;
        }

        let message = format!("{}\n", input);
        if let Err(e) = stream.write_all(message.as_bytes()).await {
            eprintln!("Failed to write to server: {}", e);
            break;
        }
        if let Err(e) = stream.flush().await {
            eprintln!("Failed to flush write buffer: {}", e);
            break;
        }

        let mut buf = vec![0; 1024];

        match stream.read(&mut buf).await {
            Ok(n) if n == 0 => {
                eprintln!("Server closed connection.");
                break;
            }
            Ok(n) => {
                let response = String::from_utf8_lossy(&buf[..n]);
                println!("{}", response.trim());
            }
            Err(e) => {
                eprintln!("Failed to read from server: {}", e);
                break;
            }
        }
    }
}
