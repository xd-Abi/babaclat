use std::fs;

use colored::*;
use inquire::Text;

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
    let app_dir = dirs::data_local_dir()
        .expect("Could not retrieve app data directory!")
        .join("Babaclat");

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("Could not create app data directory!");
    }

    let nickname = Text::new(&"Enter a nickname:".white().bold().to_string())
        .with_placeholder("Falcon")
        .prompt()
        .unwrap();

    println!("{}", "Connecting you to your local internet chats...".white().bold());

}

