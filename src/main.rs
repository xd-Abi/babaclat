use chrono::Utc;
use colored::*;
use crossterm::{cursor, execute, terminal};
use inquire::Text;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    error::Error,
    fs,
    hash::{Hash, Hasher},
    io::{stdout, Write},
    net::IpAddr,
    str,
    time::Duration,
};
use tokio::{io, io::AsyncBufReadExt, select};

use libp2p::{
    futures::StreamExt,
    gossipsub, identify,
    identity::Keypair,
    mdns,
    multiaddr::{Multiaddr, Protocol},
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder,
};

#[derive(NetworkBehaviour)]
struct ChatBehaviour {
    identify: identify::Behaviour,
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatMessage {
    content: String,
    timestamp: String,
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

    println!(
        "{}",
        "Connecting you to your local internet chats..."
            .white()
            .bold()
    );

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
        let mut file =
            fs::File::create(&identity_path).expect("Could not create identity key file!");
        file.write_all(&encoded_key)
            .expect("Could not write identity key file!");
    }

    let peer_id = PeerId::from(keypair.public());
    let mut nicknames: HashMap<PeerId, String> = HashMap::new();
    nicknames.insert(peer_id, nickname.clone());

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
        gossipsub_config,
    )
    .expect("Could not create gossipsub behaviour!");

    let identify = identify::Behaviour::new(
        identify::Config::new("/ipfs/0.1.0".into(), keypair.public())
            .with_agent_version(nickname.clone()),
    );

    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), keypair.public().to_peer_id())
        .expect("Could not create mdns!");

    let behaviour = ChatBehaviour {
        gossipsub,
        identify,
        mdns,
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

    let address_tcp = Multiaddr::from(listen_address).with(Protocol::Tcp(0));

    swarm
        .listen_on(address_tcp.clone())
        .expect("Failed to listen on tcp address.");
    swarm
        .listen_on(address_quic.clone())
        .expect("Failed to listen on quic address.");

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                if !line.trim().is_empty() {
                    let message_json = ChatMessage {
                        content: line.clone(),
                        timestamp: Utc::now().to_rfc3339(),
                    };

                    let message = serde_json::to_string(&message_json).unwrap();

                    execute!(
                        stdout(),
                        cursor::MoveUp(1),
                        terminal::Clear(terminal::ClearType::CurrentLine)
                    ).unwrap();

                    let formatted_sender = format!("<{} (You)>", nickname).bright_cyan().bold();
                    let formatted_message = line.white().bold();
                    println!("{}: {}", formatted_sender, formatted_message);

                    if let Err(_e) = swarm
                        .behaviour_mut().gossipsub
                        .publish(chat_topic.clone(), message.as_bytes()) {

                        execute!(
                            stdout(),
                            cursor::MoveUp(1),
                            terminal::Clear(terminal::ClearType::CurrentLine)
                        ).unwrap();

                        let formatted_error = format!("<{} (You)> {} - Could not send message", nickname, line).red().bold();
                        println!("{}", formatted_error);
                    }
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(ChatBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. })) => {
                    nicknames.insert(peer_id, info.agent_version.clone());
                    let message = format!("{} joined the chat", info.agent_version).yellow().bold();
                    println!("{}", message);
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    if let Some(error) = cause {
                       let error_string = format!("{:?}", error);
                        if error_string.contains("code: 10054") {
                            if let Some(person) = nicknames.get(&peer_id) {
                               let message = format!("{} left the chat", person).yellow().bold();
                               println!("{}", message);
                            }
                        }
                    }
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source,
                    message_id: _,
                    message,
                })) => {
                    if let Ok(message_str) = str::from_utf8(&message.data) {
                        if let Ok(parsed_message) = serde_json::from_str::<ChatMessage>(message_str) {
                            let sender = nicknames.get(&propagation_source).unwrap();
                            let formatted_sender = format!("<{}>", sender).bright_green().bold();
                            let formatted_message = parsed_message.content.white().bold();

                            println!("{} {}", formatted_sender, formatted_message);
                        }
                    }
                },
                _ => {}
            }
        }
    }
}
