use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {

  // 1. Listen to 127.0.0.1 port 8888
  let listener = TcpListener::bind("127.0.0.1:8888").unwrap(); 
  println!("listening started, ready to accept");

  // 2. For each connected stream, match Err if error happens, otherwise
  //    call handle_connection function
  for raw_stream in listener.incoming() {
    match raw_stream {
      // 2.1 Call panic and print error log if error occurs
      Err(error) => panic!("Error happened while accepting client: {}", error),
      // 2.2 Otherwise print welcome message and call handler
      Ok(mut stream) => {
        stream.write(b"Welcome to rust server!\r\n").unwrap();
        handle_connection(stream);
      }
    }
  }
}

// 3. Connection handler
fn handle_connection(mut stream: TcpStream) {
  // 3.1 Define a buffer to hold client msg
  let mut buffer = [0; 1024];

  // 3.2 Read from client
  stream.read(&mut buffer).unwrap();

  // 3.3 Print message sent from client side
  println!("Client message: {}", String::from_utf8_lossy(&buffer[..]));
}