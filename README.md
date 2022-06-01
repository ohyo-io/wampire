<div align="center">

[![](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/wampire.svg)](#top)

# Asynchronous Web Application Messaging Protcol (v2)

[![API Docs][docrs-badge]][docrs-url]
[![Crates.io][crates-badge]][crates-url]
[![Code coverage][codecov-badge]][codecov-url]
[![Tests][tests-badge]][tests-url]
[![MPL-2.0 licensed][license-badge]][license-url]
[![Gitter chat][gitter-badge]][gitter-url]
[![loc][loc-badge]][loc-url]
</div>

[docrs-badge]: https://img.shields.io/docsrs/wampire?style=flat-square
[docrs-url]: https://docs.rs/wampire/
[crates-badge]: https://img.shields.io/crates/v/wampire.svg?style=flat-square
[crates-url]: https://crates.io/crates/wampire
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square
[license-url]: https://github.com/ohyo-io/wampire/blob/master/LICENSE
[gitter-badge]: https://img.shields.io/gitter/room/angular_rust/community.svg?style=flat-square
[gitter-url]: https://gitter.im/angular_rust/community
[tests-badge]: https://img.shields.io/github/workflow/status/ohyo-io/wampire/Tests?label=tests&logo=github&style=flat-square
[tests-url]: https://github.com/ohyo-io/wampire/actions/workflows/tests.yml
[codecov-badge]: https://img.shields.io/codecov/c/github/ohyo-io/wampire?logo=codecov&style=flat-square&token=L7KV27OLY0
[codecov-url]: https://codecov.io/gh/ohyo-io/wampire
[loc-badge]: https://img.shields.io/tokei/lines/github/ohyo-io/wampire?style=flat-square
[loc-url]: https://github.com/ohyo-io/wampire

**Wampire** is a asynchronous [Web Application Messaging Protcol v2](http://wamp-proto.org/) router library, client library, and a router service, that implements most of the features defined in the advanced profile. The wampire project is written in [Rust](https://www.rust-lang.org/) and designed for highly concurrent asynchronous I/O. The wampire router provides extended functionality.  The router and client interaction with other WAMP implementations. 
Project initially forked from [wamp-rs v0.1.0](https://github.com/dyule/wamp-rs).

## WAMP use-cases

- Check the [examples/webrtc-simple](examples/webrtc-simple) folder 
for example using wampire as signaling server for WebRTC connection. 
- [Free Your Code - Backends in the Browser](https://crossbario.com/blog/Free-Your-Code-Backends-in-the-Browser/)
- [Security in the IoT](https://crossbario.com/static/presentations/iot-security/index.html)
- [Scaling microservices with Crossbar.io](https://crossbario.com/static/presentations/microservices/index.html)

## WAMP comparison[](#wamp-compared "Permalink to this headline")

Alright. So how does WAMP stack up versus other technologies?

Do we really need another wheel? Yes. Please read below to find out why we think so.

Below you’ll find a table comparing WAMP to other technologies according to **six criteria**:

1. **PubSub** Does it support Publish & Subscribe out of the box?
  
2. **RPC** Does it support Remote Procedure Calls out of the box?
  
3. **Routed RPC** Does it support [routed](https://wamp-proto.org/why/#unified_routing) (not only point-to-point) Remote Procedure Calls?
  
4. **Web native** Does it run *natively* on the Web (without tunneling or bridging)?
  
5. **Cross Language** Does it work from different programming languages and run-times?
  
6. **Open Standard** Is there an open, official specification implemented by different vendors?
  

See also: [Web Technologies for the Internet of Things](http://iotiran.com/media/k2/attachments/web-technologies.pdf) - A master thesis which contains a comparison of WAMP, MQTT, CoAP, REST, SOAP, STOMP and MBWS for IoT applications.

| --- | --- | --- | --- | --- | --- | --- |
| Technology | PubSub | RPC | Routed RPC | Web native | Cross Language | Open Standard |
| WAMP                          | ✔    | ✔   | ✔   | ✔   | ✔   | ✔   |
| [AJAX](#ajax)                 | **-** | ✔   | **-** | ✔   | ✔   | **-** |
| [AMQP](#amqp)                 | ✔    | (✔) | **-** | **-** | ✔   | ✔   |
| [Apache Thrift](#thrift)      | **-** | ✔   | **-** | **-** | ✔   | **-** |
| [Capn’n’Proto](#capnnproto)   | **-** | ✔   | **-** | **-** | ✔   | **-** |
| [Comet](#comet)               | **-** | **-** | **-** | ✔   | ✔   | **-** |
| [OMG DDS](#omg-dds)           | ✔    | **-** | **-** | **-** | ✔   | ✔   |
| [D-Bus](#d-bus)               | ✔    | ✔   | ✔   | **-** | ✔   | ✔   |
| [CORBA](#corba)               | ✔    | ✔   | **-** | **-** | ✔   | ✔   |
| [DCOM](#dcom)                 | ✔    | ✔   | **-** | **-** | ✔   | **-** |
| [Java JMS](#jms)              | ✔    | **-** | **-** | **-** | **-** | ✔   |
| [Java RMI](#java-rmi)         | **-** | ✔   | **-** | **-** | **-** | ✔   |
| [JSON-RPC](#json-rpc)         | **-** | ✔   | **-** | ✔   | ✔   | ✔   |
| [MQTT](#mqtt)                 | ✔    | **-** | **-** | **-** | ✔   | ✔   |
| [OPC-UA](#opc-ua)             | (✔)  | ✔ |     | **-** | (✔) | ✔   | ✔   |
| [REST](#rest)                 | **-** | ✔   | **-** | ✔   | ✔   | **-** |
| [SOAP](#soap)                 | **-** | ✔   | **-** | ✔   | ✔   | ✔   |
| [socket.io](#socketio)        | ✔    | **-** | **-** | ✔   | **-** | **-** |
| [SockJS](#sockjs)             | **-** | **-** | **-** | ✔   | ✔   | **-** |
| [STOMP](#stomp)               | ✔    | **-** | **-** | ✔   | ✔   | ✔   |
| [XML-RPC](#xml-rpc)           | **-** | ✔   | **-** | ✔   | ✔   | ✔   |
| [XMPP](#xmpp)                 | ✔    | **-** | **-** | ✔   | ✔   | ✔   |
| [ZMQ](#zmq)                   | ✔    | **-** | **-** | **-** | ✔   | **-** |

## Implementations

There a lot of implementations for different languages. [Read more..](https://wamp-proto.org/implementations.html)

## Frequently Asked Questions
[Read more..](https://wamp-proto.org/faq.html)

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
wampire = "0.2"
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

See [FEATURES](./FEATURES.md) for details.

## Extended Functionality

Wampire provides [extended functionality](https://github.com/ohyo-io/wampire/wiki/Extended-Functionality) 
around subscriber black/white listing and in the information available via the session meta API.  
This enhances the ability of clients to make desisions about message recipients.

## Supporting Wampire

Wampire is an MIT-licensed open source project. It's an independent project with its ongoing development made possible 
entirely thanks to the support by these awesome [backers](./BACKERS.md). If 
you'd like to join them, please consider:

[![Become a patron](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/patreon.png)](https://www.patreon.com/dudochkin)
[![ko-fi](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/kofi2.png)](https://ko-fi.com/Y8Y3E0YQ)

## License

This work is licensed under the MIT license. See [LICENSE](./LICENSE) for details.
