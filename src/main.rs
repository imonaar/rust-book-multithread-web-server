use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection Established");
        handle_connection(stream)
    }
}

fn handle_connection(mut stream: TcpStream) {
    /*
       read the stream into a buffer to reduce syscalls
       BufReader adds buffering by managing calls to the std::io::Read

       The Read trait allows for reading bytes from a source
       Readers are defined by one required method, read().
       Each call to read() will attempt to pull bytes from this source into a provided buffer.

       *read() may involve a system call, and therefore, using something that implements BufRead, such as BufReader, will be more efficient
    */

    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // let http_request: Vec<_> = buf_reader
    //     .lines()
    //     .map(|result| result.unwrap())
    //     .take_while(|line| !line.is_empty())
    //     .collect();

    // if request_line == "GET / HTTP/1.1" {
    //     let status_line = "HTTP/1.1 200 OK";
    //     let contents = fs::read_to_string("hello.html").unwrap();
    //     let len = contents.len();
    //     let response = format!("{status_line}\r\nContents_Length: {len}\r\n\r\n{contents}");
    //     stream.write_all(response.as_bytes()).unwrap();
    // } else {
    //     let status_line = "HTTP/1.1 404 NOT FOUND";
    //     let contents = fs::read_to_string("404.html").unwrap();
    //     let len = contents.len();

    //     let response = format!("{status_line}\r\nContent-Length: {len}\r\n\r\n{contents}");

    //     stream.write_all(response.as_bytes()).unwrap();
    // }

    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let len = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {len}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();

}