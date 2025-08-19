use std::fmt;
use wayland_client::protocol::wl_output;

#[derive(Debug, Clone)]
pub struct OutputInfo {
    pub id: u32,
    pub name: String,
    pub description: String,
}

impl fmt::Display for OutputInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<ID: #{} (Name: <{}> Manufacturer: <{}>)>",
            self.id, self.name, self.description
        )
    }
}

#[derive(Debug, Clone)]
pub struct PendingOutputInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub proxy: wl_output::WlOutput,
}

impl fmt::Display for PendingOutputInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<Pending Output (Name: {:?}, Description: {:?})>",
            self.name, self.description
        )
    }
}

pub fn clean_description(description: &str, name: &str) -> String {
    let mut cleaned_description = description.to_string();
    let suffix_to_remove = format!(" ({name})");
    if let Some(stripped) = cleaned_description.strip_suffix(&suffix_to_remove) {
        cleaned_description = stripped.to_string();
    }
    cleaned_description
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_description_with_suffix() {
        let name = "HDMI-1";
        let description = "Dell Monitor (HDMI-1)";
        let cleaned = clean_description(description, name);
        assert_eq!(cleaned, "Dell Monitor");
    }

    #[test]
    fn test_clean_description_without_suffix() {
        let name = "HDMI-1";
        let description = "Dell Monitor";
        let cleaned = clean_description(description, name);
        assert_eq!(cleaned, "Dell Monitor");
    }

    #[test]
    fn test_output_info_display() {
        let output = OutputInfo {
            id: 42,
            name: "HDMI-1".to_string(),
            description: "Dell Monitor".to_string(),
        };
        let display_string = format!("{}", output);
        assert_eq!(
            display_string,
            "<ID: #42 (Name: <HDMI-1> Manufacturer: <Dell Monitor>)>"
        );
    }
}
