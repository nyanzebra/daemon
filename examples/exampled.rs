use std::{fs::read_to_string, thread::sleep, time::Duration};

fn main() -> std::io::Result<()> {
    let content = read_to_string("/etc/exampled/config.toml")?;
    println!("read configuration file and got {content}");
    sleep(Duration::from_secs(30));
    Ok(())
}
