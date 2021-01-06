use std::fs::File;
use std::io::Read;
use std::pin::Pin;

use openssl::base64::encode_block;
use openssl::hash::MessageDigest;
use openssl::ssl::{Ssl, SslAcceptor, SslConnector, SslFiletype, SslMethod, SslVerifyMode};
use openssl::x509::X509;
use tokio::net::TcpStream;
use tokio_openssl::SslStream;

use crate::error::Result;

pub fn cert_digest(file: &str) -> Result<String> {
    let mut file = File::open(file)?;
    let pem = &mut String::new();
    file.read_to_string(pem)?;
    let cert = X509::from_pem(pem.as_bytes())?;
    let digest = cert.digest(MessageDigest::sha256())?;
    Ok(encode_block(digest.as_ref()))
}

pub fn peer_digest(tls: &SslStream<TcpStream>) -> Result<String> {
    let digest = tls
        .ssl()
        .peer_certificate()
        .unwrap()
        .digest(MessageDigest::sha256())?;
    Ok(encode_block(digest.as_ref()))
}

pub async fn tls_accept(stream: TcpStream, cert: &str, key: &str) -> Result<SslStream<TcpStream>> {
    let mut acceptor_builder = SslAcceptor::mozilla_modern(SslMethod::tls_server())?;
    acceptor_builder.set_certificate_chain_file(cert)?;
    acceptor_builder.set_private_key_file(key, SslFiletype::PEM)?;

    let acceptor = acceptor_builder.build();
    let mut ssl_stream = SslStream::new(Ssl::new(acceptor.context())?, stream)?;

    Pin::new(&mut ssl_stream).accept().await?;
    Ok(ssl_stream)
}

pub async fn tls_connect(
    stream: TcpStream,
    sni: &str,
    verify: bool,
) -> Result<SslStream<TcpStream>> {
    let mut connector_builder = SslConnector::builder(SslMethod::tls_client())?;
    if !verify {
        connector_builder.set_verify(SslVerifyMode::NONE);
    }

    let connector = connector_builder.build();
    let mut ssl = Ssl::new(connector.context())?;
    ssl.set_hostname(sni)?;
    let mut ssl_stream = SslStream::new(ssl, stream)?;

    Pin::new(&mut ssl_stream).connect().await?;
    Ok(ssl_stream)
}
