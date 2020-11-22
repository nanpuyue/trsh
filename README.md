# A Reverse Shell Tool with TLS

## Usage

```
$ rshell -h
rshell 0.1.1
南浦月 <nanpuyue@gmail.com>
A Reverse Shell Tool with TLS

USAGE:
    rshell [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -r               Readonly mode (client)
    -n               Do not verify the server certificate (client)
    -V, --version    Prints version information

OPTIONS:
    -c <FILE>             Certificate chain file (server, required)
    -d <DOMAIN>           Server name to verify (client)
    -k <FILE>             Private key file (server, required)
    -l <IP:PORT>          Listen address (server, required)
    -s <HOST:PORT>        Server address to connect (client, required)
```

### Server

```shell script
$ rshell -l 0.0.0.0:2022 -c rshell.crt -k rshell.key
Server fingerprint: KjyG4ONKfTUjjsAzgEFcPpwCCaLeVtHgNqEAfWo9Oj8=
Waiting for client to connect...
```

### Client

```shell script
$ rshell -r -n -s server.host:2022
Server fingerprint: KjyG4ONKfTUjjsAzgEFcPpwCCaLeVtHgNqEAfWo9Oj8=
Do you want continue? [y/N]
y
You can use "Ctrl + C" to disconnect at any time.

```

Or you can use a certificate trusted by the system without `-n`.

## Tips

### Generate a self-signed certificate

```shell script
openssl genrsa -out rshell.key 2048
openssl req -new -x509 -days 365 -key rshell.key -out rshell.crt -subj "/CN=rshell"
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/nanpuyue/rshell/blob/master/LICENSE
