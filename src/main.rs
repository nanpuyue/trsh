use clap::{App, Arg};

mod client;
mod error;
mod server;
mod term;
mod tls;
mod util;

pub use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("rshell")
        .version("0.1.0")
        .author("南浦月 <nanpuyue@gmail.com>")
        .about("A Reverse Shell Tool with TLS")
        .arg(
            Arg::new("listen")
                .short('l')
                .value_name("IP:PORT")
                .about("Listen address")
                .takes_value(true)
                .requires_all(&["cert", "key"])
                .required_unless_present("server"),
        )
        .arg(
            Arg::new("cert")
                .short('c')
                .value_name("FILE")
                .about("Certificate chain file")
                .takes_value(true),
        )
        .arg(
            Arg::new("key")
                .short('k')
                .value_name("FILE")
                .about("Private key file")
                .takes_value(true),
        )
        .arg(
            Arg::new("server")
                .short('s')
                .value_name("HOST:PORT")
                .about("Server address to connect")
                .takes_value(true)
                .required_unless_present("listen")
                .conflicts_with("listen"),
        )
        .arg(
            Arg::new("domain")
                .short('d')
                .value_name("DOMAIN")
                .about("Server name to verify (optional)")
                .takes_value(true)
                .conflicts_with("listen"),
        )
        .arg(
            Arg::new("not_verify")
                .short('n')
                .about("Do not verify the tls cert")
                .takes_value(false)
                .conflicts_with("listen"),
        )
        .get_matches();

    if let Some(listen) = matches.value_of("listen") {
        let cert = matches.value_of("cert").unwrap();
        let key = matches.value_of("key").unwrap();
        server::server(listen, cert, key).await
    } else if let Some(server) = matches.value_of("server") {
        let sni = if let Some(domain) = matches.value_of("domain") {
            domain
        } else {
            server.split(':').next().unwrap_or_default()
        };
        let verify = matches.index_of("not_verify").is_none();
        client::client(server, sni, verify).await
    } else {
        Ok(())
    }
}
