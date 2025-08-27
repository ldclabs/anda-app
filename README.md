# Anda AI Agents Client Application

Anda AI is your symbiotic partner, a client-side agent that learns from you. It acts on your behalf across the network of agents and ultimately becomes your digital embodiment in the agentic web.

## Installation

Download the latest installers from GitHub Releases (choose the file that matches your OS and CPU architecture, file names may vary by version): https://github.com/ldclabs/anda-app/releases/latest

Tip: Check your CPU architecture
- macOS/Linux:
```bash
uname -m
```
- Common values: arm64 or aarch64 (Apple Silicon/ARM), x86_64 or amd64 (Intel/AMD 64-bit)

### macOS
Choose one of the following based on your Mac:
- Apple Silicon (M1/M2/M3):
  - Installer: Anda.AI_VERSION_aarch64.dmg
  - Alternative (unpacked app): Anda.AI_aarch64.app.tar.gz
- Intel:
  - Installer: Anda.AI_VERSION_x64.dmg
  - Alternative (unpacked app): Anda.AI_x64.app.tar.gz

Install via DMG:
1) Open the .dmg file
2) Drag “Anda AI” into the Applications folder
3) Launch from Applications (on first launch, you may need to right-click “Open” to bypass Gatekeeper)

Install via .app.tar.gz:
1) Extract the archive
```bash
tar -xzf Anda.AI_aarch64.app.tar.gz
```
2) Move “Anda AI.app” into /Applications
```bash
mv "Anda AI.app" /Applications
```

### Windows
Choose the installer that matches your system:
- 64-bit (most PCs): Anda.AI_VERSION_x64-setup.exe
- ARM64 devices (e.g., Surface with ARM): Anda.AI_VERSION_arm64-setup.exe
- 32-bit (rare/legacy): Anda.AI_VERSION_x86-setup.exe

Install:
1) Double-click the .exe installer
2) Follow the setup wizard
3) Launch “Anda AI” from Start Menu
Note: If SmartScreen warns, click “More info” → “Run anyway”.

### Linux
Pick one of the formats supported by your distribution and architecture.

Debian/Ubuntu (.deb):
- amd64 (x86_64): Anda.AI_VERSION_amd64.deb
- arm64 (aarch64): Anda.AI_VERSION_arm64.deb

Install (with dependencies resolved automatically):
```bash
sudo apt install ./Anda.AI_VERSION_amd64.deb
```

RPM-based distros (.rpm):
- aarch64: Anda.AI-VERSION.aarch64.rpm
- x86_64: Anda.AI-VERSION.x86_64.rpm

Install (rpm):
```bash
sudo rpm -i Anda.AI-VERSION.x86_64.rpm
```
Or with dnf (Fedora):
```bash
sudo dnf install ./Anda.AI-VERSION.x86_64.rpm
```

AppImage (portable; no root required):
- amd64: Anda.AI_VERSION_amd64.AppImage
- aarch64: Anda.AI_VERSION_aarch64.AppImage

Make executable:
```bash
chmod +x Anda.AI_VERSION_amd64.AppImage
```
Run:
```bash
./Anda.AI_VERSION_amd64.AppImage
```

Notes:
- Replace file names with the matching architecture (e.g., use aarch64 variants on ARM devices).
- On first run, your desktop environment may ask to trust the AppImage.

## Development

```bash
pnpm start
```

## 📝 License

Copyright © 2025 [LDC Labs](https://github.com/ldclabs).

`ldclabs/anda-app` is licensed under the MIT License. See [LICENSE](./LICENSE) for the full license text.
