use std::io;

/// Asks for confirmation
pub fn really(msg: &str) -> bool {
    println!("{} [y/N]", msg);
    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() {
        return false;
    }
    ["yes", "y", "j", "ja", "oui", "si", "da"].contains(&answer.trim())
}
