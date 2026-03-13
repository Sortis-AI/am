use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Text,
}

impl Format {
    pub fn from_str_opt(s: Option<&str>) -> Self {
        match s {
            Some("text") => Format::Text,
            _ => Format::Json,
        }
    }
}

pub fn print_json<T: Serialize>(value: &T) -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(value)?;
    println!("{json}");
    Ok(())
}

pub fn print_json_pretty<T: Serialize>(value: &T) -> Result<(), serde_json::Error> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{json}");
    Ok(())
}

pub fn eprint_json<T: Serialize>(value: &T) -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(value)?;
    eprintln!("{json}");
    Ok(())
}

pub fn eprint_json_pretty<T: Serialize>(value: &T) -> Result<(), serde_json::Error> {
    let json = serde_json::to_string_pretty(value)?;
    eprintln!("{json}");
    Ok(())
}
