# Updating Services After Removing Anime Code

To ensure the systemd services use the updated version without anime support, follow these steps:

## 1. Rebuild the Project

```bash
# Build the release version
make build

# Or if you want debug builds
DEBUG=1 make build
```

## 2. Stop the Running Services

```bash
# Stop the system service (asusd)
sudo systemctl stop asusd

# Stop the user service (asusd-user)
systemctl --user stop asusd-user
```

## 3. Install the Updated Binaries

```bash
# Install binaries and service files
sudo make install

# Or if you want to specify a custom DESTDIR
sudo make install DESTDIR=/custom/path
```

This will:
- Install the updated `asusctl`, `asusd`, and `asusd-user` binaries to `/usr/bin/`
- Install/update the systemd service files to `/usr/lib/systemd/system/` and `/usr/lib/systemd/user/`

## 4. Reload systemd Configuration

```bash
# Reload systemd to pick up any service file changes
sudo systemctl daemon-reload
systemctl --user daemon-reload
```

## 5. Start the Services

```bash
# Start the system service
sudo systemctl start asusd

# Start the user service
systemctl --user start asusd-user
```

## 6. Verify Services are Running

```bash
# Check system service status
sudo systemctl status asusd

# Check user service status
systemctl --user status asusd-user

# Check if services are enabled (auto-start on boot)
systemctl is-enabled asusd
systemctl --user is-enabled asusd-user
```

## 7. Enable Services (if not already enabled)

If you want the services to start automatically on boot:

```bash
# Enable system service
sudo systemctl enable asusd

# Enable user service
systemctl --user enable asusd-user
```

## Quick One-Liner (if already installed)

If the services are already installed and you just want to rebuild and restart:

```bash
make build && sudo make install && sudo systemctl daemon-reload && sudo systemctl restart asusd && systemctl --user daemon-reload && systemctl --user restart asusd-user
```

## Troubleshooting

If services fail to start, check the logs:

```bash
# Check system service logs
sudo journalctl -u asusd -b

# Check user service logs
journalctl --user -u asusd-user -b

# Follow logs in real-time
sudo journalctl -u asusd -f
journalctl --user -u asusd-user -f
```

## Verify Anime Removal

You can verify that anime support has been removed:

```bash
# Check that anime command is no longer available
asusctl anime --help
# Should show: "Error: unknown command 'anime'"

# Check supported interfaces (should not include xyz.ljones.Anime)
asusctl --show-supported
```
