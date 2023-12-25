use std::io::{Error, ErrorKind};

pub fn make_error_message_after_command_call(command_name: &str, err: Error) -> String {
    match err.kind() {
        ErrorKind::NotFound => format!("{} binary not found", command_name),
        _ => format!("Could not start {}", command_name),
    }
}

pub fn trim_string(s: &mut String) -> &mut String {
    s.chars()
        .position(|c| c != ' ' && c != '\n' && c != '\t')
        .map(|index| s.drain(0..index));

    if let Some(index) = s
        .chars()
        .rev()
        .position(|c| c != ' ' && c != '\n' && c != '\t')
    {
        s.truncate(s.len() - index)
    }

    s
}
