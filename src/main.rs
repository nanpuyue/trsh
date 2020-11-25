use clap::{App, Arg};

mod client;
mod error;
mod server;
mod term;
mod tls;
mod util;

use error::Result;
use term::{restore_termios, set_exit_handler};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let matches = App::new("trsh")
        .version("0.1.1")
        .author("南浦月 <nanpuyue@gmail.com>")
        .about("A TLS encrypted Reverse Shell")
        .arg(
            Arg::new("listen")
                .short('l')
                .value_name("IP:PORT")
                .about("Listen address (server, required)")
                .takes_value(true)
                .requires_all(&["cert", "key"])
                .required_unless_present("server"),
        )
        .arg(
            Arg::new("cert")
                .short('c')
                .value_name("FILE")
                .about("Certificate chain file (server, required)")
                .takes_value(true),
        )
        .arg(
            Arg::new("key")
                .short('k')
                .value_name("FILE")
                .about("Private key file (server, required)")
                .takes_value(true),
        )
        .arg(
            Arg::new("server")
                .short('s')
                .value_name("HOST:PORT")
                .about("Server address to connect (client, required)")
                .takes_value(true)
                .required_unless_present("listen")
                .conflicts_with("listen"),
        )
        .arg(
            Arg::new("domain")
                .short('d')
                .value_name("DOMAIN")
                .about("Server name to verify (client)")
                .takes_value(true)
                .conflicts_with("listen"),
        )
        .arg(
            Arg::new("verify")
                .short('n')
                .about("Do not verify the server certificate (client)")
                .takes_value(false)
                .conflicts_with("listen"),
        )
        .arg(
            Arg::new("readonly")
                .short('r')
                .about("Readonly mode (client)")
                .takes_value(false)
                .conflicts_with("listen"),
        )
        .get_matches();

    let _ = set_exit_handler();

    let ret = if let Some(listen) = matches.value_of("listen") {
        let cert = matches.value_of("cert").unwrap();
        let key = matches.value_of("key").unwrap();
        server::server(listen, cert, key).await
    } else if let Some(server) = matches.value_of("server") {
        let sni = if let Some(domain) = matches.value_of("domain") {
            domain
        } else {
            server.split(':').next().unwrap_or_default()
        };
        let verify = matches.index_of("verify").is_none();
        let readonly = matches.index_of("readonly").is_some();
        client::client(server, sni, verify, readonly).await
    } else {
        Ok(())
    };

    let _ = restore_termios();

    ret.map_err(|e| e.to_string().into())
}
