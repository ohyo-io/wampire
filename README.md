# Wampire

[![Build Status](https://travis-ci.org/ohyo-io/wampire.svg)](https://travis-ci.org/ohyo-io/wampire)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Chat](https://img.shields.io/badge/chat-on%20discord-7289da.svg)](https://discord.gg/Y2k3GAW)

**Wampire** is a [Web Application Messaging Protcol v2](http://wamp-proto.org/) router library, client library, and a router service, 
that implements most of the features defined in the advanced profile. The wampire project is written 
in [Rust](https://www.rust-lang.org/) and designed for highly concurrent asynchronous I/O. The wampire router 
provides extended functionality.  The router and client interaction with other WAMP implementations. 
Project initially forked from [wamp-rs v0.1.0](https://github.com/dyule/wamp-rs).

<p align="center">
    <img src="https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/wampire_webrtc.png" alt="Wampire logo" width="405" />
</p>

Check the [examples/webrtc-simple](examples/webrtc-simple) folder 
for nodejs based example using wampire as signaling server for WebRTC connection. 

## Supporting Wampire

Wampire is an MIT-licensed open source project. It's an independent project with its ongoing development made possible 
entirely thanks to the support by these awesome [backers](./BACKERS.md). If 
you'd like to join them, please consider:

[![Become a patron](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/patreon.png)](https://www.patreon.com/dudochkin)
[![ko-fi](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/kofi2.png)](https://ko-fi.com/Y8Y3E0YQ)
## Full Documentation

See the [**Wampire Project Wiki**](https://github.com/ohyo-io/wampire/wiki) for full documentation, examples, and operational details.

At present the entire Basic Profile is supported, as well as pattern based subscriptions and registrations from the Advanced Profile.

You may be looking for:

- [API documentation](https://docs.rs/wampire/)
- [Release notes](https://github.com/ohyo-io/wampire/releases)

There is currently no support for secure connections.

To include in your project, place the following in your `Cargo.toml`

```toml
[dependencies]
wampire = "0.1"
```
Wampire uses [serde-rs](https://github.com/serde-rs/serde), which requires Rust 1.15 or greater.

## Router
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
## Examples
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

## Advanced Profile Feature Support

### RPC Features

| Feature | Supported |
| ------- | --------- |
| progressive_call_results | Yes |
| progressive_calls | No |
| call_timeout | Yes |
| call_canceling | Yes |
| caller_identification | Yes |
| call_trustlevels | No |
| registration_meta_api | Yes
| pattern_based_registration | Yes |
| shared_registration | Yes |
| sharded_registration | No |
| registration_revocation | No |
| procedure_reflection | No |

### PubSub Features

| Feature | Supported |
| ------- | --------- |
| subscriber_blackwhite_listing | Yes |
| publisher_exclusion | Yes |
| publisher_identification | Yes |
| publication_trustlevels | No|
| subscription_meta_api | Yes |
| pattern_based_subscription | Yes |
| sharded_subscription | No |
| event_history | No |
| topic_reflection | No |
| testament_meta_api | Yes |

### Other Advanced Features

| Feature | Supported |
| ------- | --------- |
| challenge-response authentication | Yes |
| cookie authentication | Yes |
| ticket authentication | Yes |
| rawsocket transport | Yes |
| batched WS transport | No |
| longpoll transport | No |
| session meta api | Yes |
| TLS for websockets | Yes |
| TLS for rawsockets | Yes |
| websocket compression | Yes |

## Extended Functionality

Wampire provides [extended functionality](https://github.com/ohyo-io/wampire/wiki/Extended-Functionality) 
around subscriber black/white listing and in the information available via the session meta API.  
This enhances the ability of clients to make desisions about message recipients.

## Legal

### License

This work is licensed under the MIT license. See [LICENSE](./LICENSE) for details.
