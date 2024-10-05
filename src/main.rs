use std::fs;
use std::io::Write;
use colored::*;
use inquire::Text;
use libp2p::identity::Keypair;

fn main() {
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
}

