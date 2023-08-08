use std::collections::HashMap;
use std::net::SocketAddr;
use crossbeam_channel::{unbounded, Sender};
use hyper::service::{service_fn, make_service_fn};
use hyper::{Request, Response, Server, Body};
use tokio::sync::oneshot::Receiver;

pub const TOKEN_URL: &str = "127.0.0.1:3000";

async fn handle_request(req: Request<Body>, tx: Sender<String>) -> Result<Response<Body>, hyper::Error> {
    if req.uri().path() == "/login" {
        let token = req.uri().query().unwrap_or("").to_owned();
        println!("Received token");

        match tx.send(token) {
            Ok(_) => {},
            Err(e) => {
                // println!("{:?}",e.0);
                println!("An Internal Server error occured!");
            },
        }
        
    }

    Ok(Response::new(Body::default()))
}

#[tokio::main]
pub async fn server(tx: Sender<String>, shutdown_signal: Receiver<()>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // This address is localhost
    let addr: SocketAddr = ([127, 0, 0, 1], 4352).into();

    let tx2 = tx.clone();
    let make_svc = make_service_fn(move |_conn| {   
        println!("Request made");
        
        let tx3 = tx2.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_request(req, tx3.clone())
            }))

        }

    });
    let server = Server::bind(&addr).serve(make_svc);

    println!("Graceful shutdown ready");
    let graceful = server
    .with_graceful_shutdown(async {
        shutdown_signal.await.ok();
    });

    // Await the `server` receiving the signal...
    if let Err(e) = graceful.await {
        println!("server error: {}", e);
    }

    println!("Internal HTTP server ending");

    Ok(())
}
