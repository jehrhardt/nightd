//! The control plane of the nightd agent platform.

fn main() {
    println!("{}", status());
}

/// Reports the current status of the control plane.
fn status() -> String {
    format!(
        "{} {}: control plane running",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::status;

    #[test]
    fn status_reports_the_control_plane() {
        let status = status();

        assert!(status.contains(env!("CARGO_PKG_NAME")));
        assert!(status.contains(env!("CARGO_PKG_VERSION")));
        assert!(status.contains("control plane running"));
    }
}
