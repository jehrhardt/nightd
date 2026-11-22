//! The CLI to control nightd.

fn main() {
    println!("{}", status());
}

/// Reports the current status of the CLI.
fn status() -> String {
    format!(
        "{} {}: ready to control nightd",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::status;

    #[test]
    fn status_reports_the_cli() {
        let status = status();

        assert!(status.contains(env!("CARGO_PKG_NAME")));
        assert!(status.contains(env!("CARGO_PKG_VERSION")));
        assert!(status.contains("ready to control nightd"));
    }
}
