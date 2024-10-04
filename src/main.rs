use colored::*;
use inquire::Text;

fn main() {
    let banner = r#"

███╗███╗    ██████╗  █████╗ ██████╗  █████╗  ██████╗██╗      █████╗ ████████╗
██╔╝╚██║    ██╔══██╗██╔══██╗██╔══██╗██╔══██╗██╔════╝██║     ██╔══██╗╚══██╔══╝
██║  ██║    ██████╔╝███████║██████╔╝███████║██║     ██║     ███████║   ██║
██║  ██║    ██╔══██╗██╔══██║██╔══██╗██╔══██║██║     ██║     ██╔══██║   ██║
███╗███║    ██████╔╝██║  ██║██████╔╝██║  ██║╚██████╗███████╗██║  ██║   ██║
╚══╝╚══╝    ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚══════╝╚═╝  ╚═╝   ╚═╝

                  A P2P chat application build in rust
"#;

    println!("{}", banner.blue().bold());

    let nickname = Text::new(&"Enter a nickname:".white().bold().to_string())
        .with_placeholder("Falcon")
        .prompt();

    match nickname {
        Ok(name) => println!("Nickname: {}", name),
        Err(_) => println!("An error occurred while reading input."),
    }
}
