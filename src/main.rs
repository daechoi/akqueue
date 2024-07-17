use mio::{net::TcpStream, Events, Interest, Poll, Token};
use std::{
    error::Error,
    io::{ErrorKind, Read, Write},
    net::SocketAddr,
    //    net::TcpStream,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut poll = Poll::new()?;
    let n_events: usize = 5;
    let mut streams = vec![];
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    for i in 0..n_events {
        let delay = (n_events - i) * 1000;
        let mut stream = TcpStream::connect(addr).expect("Can't connect to server");
        stream.set_nodelay(true)?;
        println!("Connecting?");
        poll.registry().register(
            &mut stream,
            Token(i),
            Interest::READABLE | Interest::WRITABLE,
        )?;

        let mut events = Events::with_capacity(10);

        let mut processed = false;
        loop {
            poll.poll(&mut events, None)?;
            for event in events.iter() {
                if event.token() == Token(i) && event.is_writable() {
                    println!("Connected to server");
                    let url_path = format!("/{delay}/request-{i}");
                    let request = get_req(&url_path);
                    stream.write_all(&request)?;
                    processed = true;
                }
            }
            if processed {
                break;
            }
        }
        streams.push(stream);
    }

    println!("Sent all events!");
    let mut handled_events: usize = 0;
    while handled_events < n_events {
        let mut events = Events::with_capacity(10);
        println!("Polling... {handled_events}, n_events: {n_events}");
        poll.poll(&mut events, None)?;
        if events.is_empty() {
            println!("TIMEOUT (OR SPURIOUS EVENT NOTIFICATION)");
            continue;
        }

        handled_events += handle_events(&events, &mut streams)?;
    }
    Ok(())
}

fn get_req(path: &str) -> Vec<u8> {
    format!(
        "GET {path} HTTP/1.1\r\n\
        Host: localhost\r\n\
        Connection: close\r\n\r\n"
    )
    .into_bytes()
}

fn handle_events(events: &Events, streams: &mut [TcpStream]) -> Result<usize, Box<dyn Error>> {
    let mut handled_events = 0;

    for event in events {
        let index = event.token();
        let mut data = vec![0u8; 4096];
        let stream = &mut streams[index.0];
        if event.is_readable() {
            match stream.read(&mut data) {
                Ok(0) => {
                    handled_events += 1;
                    return Ok(handled_events);
                }
                Ok(n) => {
                    let txt = String::from_utf8_lossy(&data[..n]);
                    println!("Read {} bytes", n);
                    println!("{txt}\n=======================\n");
                    handled_events += 1;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    println!("Would block");
                    break;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
    Ok(handled_events)
}
