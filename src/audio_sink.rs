pub struct NullSinkGuard {
    child_process: std::process::Child,
}

impl NullSinkGuard {
    pub fn new() -> Option<Self> {
        let child = std::process::Command::new("pw-loopback")
            .args([
                "-m", "2",
                "--capture-props", "node.name=netshare_sink node.description=NetShare media.class=Audio/Sink",
                "--playback-props", "node.name=netshare node.description=NetShare media.class=Audio/Source",
            ])
            .spawn()
            .ok()?;

        Some(NullSinkGuard { child_process: child })
    }
}

impl Drop for NullSinkGuard {
    fn drop(&mut self) {
        let _ = self.child_process.kill();
    }
}