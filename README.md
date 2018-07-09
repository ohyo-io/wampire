# wampire
Rust implementation of a WAMP [Web Application Messaging Protcol](http://wamp-proto.org/). client and router

<p align="center">
    <img src="https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/logo_wampire.png" alt="Wampire logo" width="256" />
</p>


At present the entire Basic Profile is supported, as well as pattern based subscriptions and registrations from the Advanced Profile.

There is currently no support for secure connections.

To include in your project, place the following in your `Cargo.toml`

```toml
[dependencies]
wampire = "0.1"
```

wampire uses [serde-rs](https://github.com/serde-rs/serde), which requires Rust 1.15 or greater.

Initial forked from https://github.com/dyule/wamp-rs

## router
To start router in development mode use
```bash
RUST_LOG=info cargo run wampire
```

### Nginx configuration
To pass WebSocket connection to router add it to Nginx config.
PS. can be used with SSL too.
```
location /ws/ {
    proxy_pass http://127.0.0.1:8090;
    
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection $connection_upgrade;
    proxy_read_timeout 1800s;

    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header Host $http_host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
}
```
### Systemd
Build router:
1. Clone repo using `git clone https://github.com/ohyo-io/wampire.git`
2. `cd wampire && cargo build`
3. Copy `wampire` from `target` folder to `/usr/local/bin`
4. Copy `wampire.service` from `dist` to `/usr/lib/systemd/system` or `/lib/systemd/system` (depend on your system).

To start a service:
``` bash
systemctl start wampire
```
To enable as system service:
``` bash
systemctl enable wampire
```

## examples
Please see the [examples](examples) directory.
For instructions on how to check the examples

```bash
RUST_LOG=info cargo run --example api_user
```
```bash
RUST_LOG=info cargo run --example endpoint
```
```bash
RUST_LOG=info cargo run --example pubsubclient
```