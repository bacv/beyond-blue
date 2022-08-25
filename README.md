# Beyond Blue
An experimental multiplayer game written in rust that uses libp2p for peer to peer game data syncing.
---
This is very early WIP

https://user-images.githubusercontent.com/2364987/186654055-bf59af95-3382-4257-9d35-00fac2f3c51b.mp4

## Design goals
* *Simple*: Developer should not worry about the internal peer or relay implementation details.
* *Decentralized*: The server should not hold any game state.

## Features
* Minimal http api for peers to request information about the relay and other peers. This helps to simplify the user experience and doesn't involve complex looking mutliaddresses or private key setup.
* Game state is synced via gossipsub with other peers.
* Developer can define custom messages, the only requirement is that the custom type implements `Serialize` and `Deserialize` traits.
* Msgpack is used to encode data passed through gossipsub.

## How to use
The library has two main components - relay and peer. Relay is a server that coordinates the direct connections between peers and helps with peer discovery. Peer is a code that runs on a seperate thread inside the project and manages the network events when communicating with relay and other peers. The data between peer and the rest of the porject is passed via tokio channels.

### Peer setup
First we need to define the message type that will be understood between peers.

```rust
use serde::{Deserialize, Serialize};
use peer::NetworkEvent;

#[derive(Serialize, Deserialize, Clone)]
pub enum GameMessage {
    Move(f32, f32),
    Jump,
    Fire,
}

pub type GameEvent = NetworkEvent<GameMessage>;
```

Second, peer event loop needs to be spawned.

```rust
// Our code needs to pass `GameMessage` type.
let (local_in, local_out) = mpsc::channel(32);

// The message from `remote_out` channel will be `GameEvent` type.
let (remote_in, remote_out) = mpsc::channel(32);

// Address of the relay http api
let relay_address = url::Url::from("http://remote.example.com:8080");

tokio::spawn(async move {
    let res = peer::Swarm::new_with_default_transport(id.get_key())
	.await?
	.spawn::<GameMessage>(relay_address, remote_in, local_out)
	.await;

    log::info!("Game swarm result: {:?}", res);
});
```

Use channels to communicate with peers

```rust
local_in.try_send(GameMessage::Move(10., 10.))

match remote_out.try_recv() {
	Ok(NetworkEvent::NewConnection(peer_id)) => log::info!("New conn: {}", peer_id),
	Ok(NetworkEvent::Event(peer_id, GameMessage::Move(x, y))) => log::info!("peer {} moved to x:{} y:{}", peer_id, x, y),
	_ => {},
}
```

### Relay setup
To run relay you'll need to provide two parameters to the relay binary: 
```sh
$ ./bb-relay
error: The following required arguments were not provided:
    --secret-key-seed <SECRET_KEY_SEED>
    --swarm-port <SWARM_PORT>
    --http-port <HTTP_PORT>

USAGE:
    bb-relay [OPTIONS] --secret-key-seed <SECRET_KEY_SEED> --swarm-port <SWARM_PORT> --http-port <HTTP_PORT>

For more information try --help

$ ./bb-relay --secret-key-seed 0 --swarm-port 8042 --http-port 8080
```
## TODOs
* Use streaming protocol to pass realtime data about gamestate.
* Leave gossipsub only for nonrealtime data.
* Handle reconnects.
* Extract p2p related code to a new repository and publish a crate for that.
* Use Kademlia algorithm to connect peers into a mesh with fewer p2p connections.
