use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    spawn,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (mut socket, _addr) = listener.accept().await.unwrap();
        spawn(async move {
            // Just to note, I tried this, but with_capacity actually just
            // allocs memory, it's an empty vec (len = 0), and I guess
            // socket.read(buf) doesn't like that?
            // After consulting the doc for async_read_ext:

            // /// ... If `n` is `0`, then it can indicate one of two
            // /// scenarios:
            // ///
            // /// 1. This reader has reached its "end of file" and will likely no longer
            // ///    be able to produce bytes. Note that this does not mean that the
            // ///    reader will *always* no longer be able to produce bytes.
            // /// 2. The buffer specified was 0 bytes in length.

            // So that explains why this doesn't work
            // let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);

            // better to read into more like 4kb (4*1024) buffer but using 10
            // bytes here to see how read will loop 10 bytes at a time until no
            // bytes left
            // I like this answer for why 4kb is good
            // https://stackoverflow.com/a/6578442/9360856
            let mut buf = vec![0; 10];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("CONN CLOSED");
                        break;
                    }
                    Ok(n) => {
                        // write the data back to the socket
                        // cool to note, e.g. with 10 byte read buffer, if you
                        // send more than 10 bytes, you'll see this read 10
                        // bytes at a time until no data left, then just waits.
                        // Probably something with epoll telling process when
                        // more data is ready)
                        println!("READ {} BYTES", n);
                        if socket.write(&buf[..n]).await.is_err() {
                            println!("FAILED WRITING TO SOCKET, CLOSING");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("FAIL: {}", e);
                    }
                }
            }
        });
    }

    // loop {
    //     let (mut socket, _addr) = listener.accept().await.unwrap();
    //     spawn(async move {
    //         // // 4kb read buffer
    //         // let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);

    //         // let (reader, mut writer) = socket.split();
    //         // let mut line = String::new();
    //         // let mut buf_reader = BufReader::new(reader);

    //         let (mut reader, mut writer) = socket.split();
    //         // let bytes_read = buf_reader.read_line(&mut line).await.unwrap();
    //         // if bytes_read == 0 {
    //         //     println!("CONNECTION CLOSED");
    //         //     break;
    //         // }
    //         // if line.starts_with("exit") {
    //         //     println!("CONNECTION WANTS TO EXIT");
    //         //     break;
    //         // } else {
    //         //     // let _result = writer.write_all(line.as_bytes()).await;
    //         //     let _result = io::copy(&mut buf_reader, &mut writer).await;
    //         //     line.clear();
    //         // }
    //         let _result = io::copy(&mut reader, &mut writer).await;
    //     });
    // }
}
