use std::process::Command;

fn cleanup_old_modules() {
    let output = match Command::new("pactl")
        .args(["list", "short", "modules"])
        .output()
    {
        Ok(out) => out,
        Err(_) => return,
    };

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        let mut parts = line.splitn(3, '\t');

        let id = parts.next().unwrap_or("");
        let module = parts.next().unwrap_or("");
        let args = parts.next().unwrap_or("");

        let should_remove =
            (module == "module-null-sink"
                && args.contains("sink_name=netshare_sink"))
                ||
                (module == "module-loopback"
                    && args.contains("sink=netshare_sink"));

        if should_remove {
            let _ = Command::new("pactl")
                .args(["unload-module", id])
                .status();
        }
    }
}

pub struct NullSinkGuard {}

impl NullSinkGuard {
    pub fn new() -> Option<Self> {
        cleanup_old_modules();
        let output = Command::new("pactl")
            .args([
                "load-module", "module-null-sink",
                "sink_name=netshare_sink",
                "sink_properties=device.description=NetShare",
            ])
            .output()
            .ok()?;

        if !output.status.success() { return None; }

        Some(NullSinkGuard {})
    }
}