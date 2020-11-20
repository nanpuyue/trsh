use std::fs::File;
use std::io::Read;

use openssl::{
    base64,
    hash::MessageDigest,
    ssl::{SslAcceptor, SslConnector, SslFiletype, SslMethod, SslVerifyMode},
    x509::X509,
};
use tokio::net::TcpStream;
use tokio_openssl::{accept, connect, SslStream};

use crate::*;

pub fn cert_digest(file: &str) -> Result<String> {
    let mut file = File::open(file)?;
    let pem = &mut String::new();
    file.read_to_string(pem)?;
    let cert = X509::from_pem(pem.as_bytes())?;
    let digest = cert.digest(MessageDigest::sha256())?;
    Ok(base64::encode_block(digest.as_ref()))
}

pub fn peer_digest(tls: &SslStream<TcpStream>) -> Result<String> {
    let digest = tls
        .ssl()
        .peer_certificate()
        .unwrap()
        .digest(MessageDigest::sha256())?;
    Ok(base64::encode_block(digest.as_ref()))
}

pub async fn tls_accept(stream: TcpStream, cert: &str, key: &str) -> Result<SslStream<TcpStream>> {
    let mut ssl_acceptor_builder = SslAcceptor::mozilla_modern(SslMethod::tls_server())?;
    ssl_acceptor_builder.set_certificate_chain_file(cert)?;
    ssl_acceptor_builder.set_private_key_file(key, SslFiletype::PEM)?;

    let ssl_acceptor = ssl_acceptor_builder.build();
    Ok(accept(&ssl_acceptor, stream).await?)
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
    let config = connector_builder.build().configure()?;
    Ok(connect(config, sni, stream).await?)
}
