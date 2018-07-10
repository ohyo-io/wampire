const fastify = require("fastify")();
const path = require("path");

fastify.register(require("fastify-static"), {
  root: path.join(__dirname, "static")
});

var autobahn = require("autobahn");

const PORT = 9008;
const IP_ADDRESS = '127.0.0.1';

var wsList = [];

var wamp = new autobahn.Connection({
  url: `ws://${IP_ADDRESS}:8090/ws/`,
  realm: "wampire_realm"
});

wamp.onopen = function(session) {
  console.log("ON OPEN");

  session.subscribe("room", args => {
    console.log("Event:", args[0]);
  });

  session.publish("room", ["Hello, world!"]);

  // 3) register a procedure for remoting
  function add2(args) {
    console.log("ADD INVOCATION");
    return args[0] + args[1];
  }
  session.register("com.myapp.add2", add2);

  // 4) call a remote procedure
  // session.call("com.myapp.add2", [2, 3]).then(function(res) {
  //   console.log("Result:", res);
  // });
};

wamp.open();

const jsonrpc_schema = {
  body: {
    type: "object",
    properties: {
      jsonrpc: { type: "string" },
      method: { type: "string" },
      params: { type: "object" },
      id: { type: "number" }
    }
  }
};

const opt_schema = {
  body: {
    type: "string"
  }
};

fastify.addHook("onRequest", (req, res, next) => {
  console.log(req.method, req.url, req.headers);
  next();
});

fastify.addHook("preHandler", (req, res, next) => {
  next();
});

fastify.addHook("onSend", (req, res, payload, next) => {
  next();
});

fastify.addHook("onResponse", (res, next) => {
  next();
});

fastify.options("/api", { jsonrpc_schema }, function(request, reply) {
  reply.header("Access-Control-Allow-Origin", "*");
  reply.header("Access-Control-Allow-Methods", "OPTIONS, GET");
  reply.header(
    "Access-Control-Allow-Headers",
    "Origin, X-Requested-With, Content-Type, Accept"
  );
  reply.send("");
});

fastify.post("/message/:room/:client_id", { opt_schema }, function(
  request,
  reply
) {
  console.log(request.body);
  reply.send({ result: "SUCCESS" });
});

fastify.post(
  "/leave/:room/:client_id",
  function(request, reply) {
    console.log(request.body);
    reply.send({ result: "SUCCESS" });
  }
);

fastify.post(
  "/join/:room",
  function(request, reply) {

    console.log(request.body);
    reply.header("Access-Control-Allow-Origin", "*");
    reply.header("Access-Control-Allow-Methods", "OPTIONS, GET");
    reply.header(
      "Access-Control-Allow-Headers",
      "Origin, X-Requested-With, Content-Type, Accept"
    );

    reply.send({
      params: {
        is_initiator: "true",
        room_link: "https://localhost/r/21204838",
        version_info:
          '{"gitHash": "20cdd7652d58c9cf47ef92ba0190a5505760dc05", "branch": "master", "time": "Fri Mar 9 17:06:42 2018 +0100"}',
        messages: [],
        error_messages: [],
        client_id: "29315775",
        ice_server_transports: "",
        bypass_join_confirmation: "false",
        wss_url: "wss://localhost/ws/",
        media_constraints: '{"audio": true, "video": true}',
        include_loopback_js: "",
        is_loopback: "false",
        offer_options: "{}",
        pc_constraints: '{"optional": []}',
        pc_config:
          '{"rtcpMuxPolicy": "require", "bundlePolicy": "max-bundle", "iceServers": []}',
        wss_post_url: "https://localhost/post",
        // ice_server_url: "https://networktraversal.googleapis.com/v1alpha/iceconfig?key=AIzaSyAJdh2HkajseEIltlZ3SIXO02Tze9sO3NY",
        warning_messages: [],
        room_id: "21204838",
        include_rtstats_js:
          '<script src="/js/rtstats.js"></script><script src="/pako/pako.min.js"></script>'
      },
      result: "SUCCESS"
    });
  }
);

console.log(`server listening on ${IP_ADDRESS}:${PORT}`);

const start = async () => {
  try {
    await fastify.listen(PORT, IP_ADDRESS);
  } catch (err) {
    fastify.log.error(err);
    process.exit(1);
  }
};

start();
