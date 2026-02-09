# wap

Watch and push - auto-commits and pushes changes to a git repo.

## Install as systemd service

```bash
cargo build --release
cp wap.service.template ~/.config/systemd/user/wap.service
# Edit wap.service: replace {WAP_BINARY_PATH} and {REPO_PATH}
systemctl --user daemon-reload
systemctl --user enable --now wap
```

## Install as launchd service (macOS)

```bash
cargo build --release
cp com.wap.plist ~/Library/LaunchAgents/
# Edit plist: replace {WAP_BINARY_PATH} and {REPO_PATH}
launchctl load ~/Library/LaunchAgents/com.wap.plist
```
