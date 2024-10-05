use std::{fs, io};
use std::io::{BufRead, Write};
use std::{
    collections::hash_map::DefaultHasher, error::Error, hash::{Hash, Hasher}, net::IpAddr, path::Path, time::Duration
};

use colored::*;
use inquire::Text;
use libp2p::identity::Keypair;
use libp2p::{
    gossipsub, identify, identity, mdns, noise, swarm::{NetworkBehaviour, SwarmEvent}, tcp, yamux,  multiaddr::{Multiaddr, Protocol}, PeerId, Swarm, SwarmBuilder
};
use libp2p::futures::StreamExt;
use tokio::select;

#[derive(NetworkBehaviour)]
struct ChatBehaviour {
    identify: identify::Behaviour,
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let banner = r#"

███╗      ███╗    ██████╗  █████╗ ██████╗  █████╗  ██████╗██╗      █████╗ ████████╗
██╔╝▄ ██╗▄╚██║    ██╔══██╗██╔══██╗██╔══██╗██╔══██╗██╔════╝██║     ██╔══██╗╚══██╔══╝
██║  ████╗ ██║    ██████╔╝███████║██████╔╝███████║██║     ██║     ███████║   ██║
██║ ▀╚██╔▀ ██║    ██╔══██╗██╔══██║██╔══██╗██╔══██║██║     ██║     ██╔══██║   ██║
███╗  ╚═╝ ███║    ██████╔╝██║  ██║██████╔╝██║  ██║╚██████╗███████╗██║  ██║   ██║
╚══╝      ╚══╝    ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚══════╝╚═╝  ╚═╝   ╚═╝

                  A P2P chat application build in rust
"#;

    println!("{}", banner.blue().bold());

    let nickname = Text::new(&"Enter a nickname:".white().bold().to_string())
        .with_placeholder("Falcon")
        .with_default("Falcon")
        .prompt()
        .unwrap();

    println!("{}", "Connecting you to your local internet chats...".white().bold());

    let app_dir = dirs::data_local_dir()
        .expect("Could not retrieve app data directory!")
        .join("Babaclat");

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("Could not create app data directory!");
    }

    let identity_path = app_dir.join(".identity");
    let mut keypair = Keypair::generate_ed25519();
    if identity_path.exists() {
        println!("{}", "Loading your identity key...".white().bold());
        let bytes = fs::read(&identity_path).expect("Could not read identity key!");
        keypair = Keypair::from_protobuf_encoding(&bytes).unwrap();
    } else {
        println!("{}", "Generating new identity key...".green().bold());
        let encoded_key = keypair.to_protobuf_encoding().unwrap();
        let mut file = fs::File::create(&identity_path).expect("Could not create identity key file!");
        file.write_all(&encoded_key).expect("Could not write identity key file!");
    }

    let peer_id = PeerId::from(keypair.public());
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .message_id_fn(message_id_fn)
        .build()
        .expect("Could not create gossipsub config!");

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config
    ).expect("Could not create gossipsub behaviour!");

    let identify = identify::Behaviour::new(
        identify::Config::new(
            "/ipfs/0.1.0".into(),
            keypair.public()
        )
    );

    let mdns = mdns::tokio::Behaviour::new(
        mdns::Config::default(),
        keypair.public().to_peer_id()
    ).expect("Could not create mdns!");

    let behaviour = ChatBehaviour {
        gossipsub,
        identify,
        mdns
    };

    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let chat_topic = gossipsub::IdentTopic::new("babaclat-chat");
    swarm.behaviour_mut().gossipsub.subscribe(&chat_topic)?;

    let listen_address: IpAddr = "0.0.0.0".parse().expect("Invalid IP address!");
    let address_quic = Multiaddr::from(listen_address)
        .with(Protocol::Udp(0))
        .with(Protocol::QuicV1);

    let address_tcp = Multiaddr::from(listen_address)
        .with(Protocol::Tcp(0));

    swarm.listen_on(address_tcp.clone()).expect("Failed to listen on tcp address.");
    swarm.listen_on(address_quic.clone()).expect("Failed to listen on quic address.");

    // let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                },
                // Prints peer id identify info is being sent to.
                SwarmEvent::Behaviour(ChatBehaviourEvent::Identify(identify::Event::Sent { peer_id, .. })) => {
                    println!("Sent identify info to {peer_id:?}")
                }
                // Prints out the info received via the identify event
                SwarmEvent::Behaviour(ChatBehaviourEvent::Identify(identify::Event::Received { info, .. })) => {
                    println!("Received {info:?}")
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                _ => {}
            }
        }
    }
}

