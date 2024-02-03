use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        //can we assume that stream in this case is a single request?
        let stream = stream.unwrap();

        //then we execute that reuest on new thread? if a thread is available?

        //execute function so it takes the closure it’s given and gives it to an idle thread in the pool to run.
        //handleconnection is the function to execute when the thread is created

        //we want to create the threads and have them wait for code that we’ll send later.
        pool.execute(|| handle_connection(stream));
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

    // let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
    //     ("HTTP/1.1 200 OK", "hello.html")
    // } else {
    //     ("HTTP/1.1 404 NOT FOUND", "404.html")
    // };

    /*
        We need to explicitly match on a slice of request_line to pattern match against the string literal values;
        match doesn’t do automatic referencing and dereferencing like the equality method does.
    */
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let len = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {len}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
