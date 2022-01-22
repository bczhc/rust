use http::Request;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper_tls::HttpsConnector;

#[tokio::main]
async fn main() {
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let response = client
        .get("https://www.google.com".parse().unwrap())
        .await
        .unwrap();
    println!("{:?}", response);
}
