use snapd::UdsIo;

use std::future::IntoFuture;

use http_body_util::{BodyExt, Empty};
use hyper::{
    body::{Body, Bytes},
    client::conn::http1 as conn,
    Request, Uri,
};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let uri = Uri::builder()
        .scheme("http")
        .authority("localhost")
        .path_and_query("/v2/aliases")
        .build()
        .unwrap();

    println!("A");
    // Note to self this is an io::Error;
    let stream = UdsIo::from(UnixStream::connect("/run/snapd.socket").await.unwrap());
    println!("B");

    let (mut sender, connection) = conn::handshake(stream).await.unwrap();
    println!("C");

    let handle = tokio::task::spawn(async move {
        if let Err(err) = connection.await {
            panic!("{}", err)
        }
    });
    println!("D");

    let req = Request::builder()
        .uri(uri)
        .header(hyper::header::HOST, "localhost")
        .body(Empty::<Bytes>::new())
        .unwrap();

    let response = sender.send_request(req).await.unwrap();
    println!("E");

    let body = response.collect().await.unwrap().aggregate();
    println!("F");

    drop(sender);
    handle.await.unwrap();

    Ok(())
}
