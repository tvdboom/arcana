KTX_VERSION := "4.3.2"

run:
    cargo run

build:
    cargo build

build-release:
    cargo build --release

[windows]
install-ktx:
    powershell -NoProfile -NonInteractive -Command '$ErrorActionPreference = "Stop"; $ZipPath = "KTX-Software-{{KTX_VERSION}}-Windows-x64.zip"; $ExtractPath = "C:\KTX-Software"; try { Invoke-WebRequest -Uri "https://github.com/KhronosGroup/KTX-Software/releases/download/v{{KTX_VERSION}}/KTX-Software-{{KTX_VERSION}}-Windows-x64.zip" -OutFile $ZipPath -ErrorAction Stop; Expand-Archive -Path $ZipPath -DestinationPath $ExtractPath -Force; Remove-Item $ZipPath; Write-Host "KTX-Software installed to $ExtractPath"; Write-Host "KTX_BIN_PATH=$ExtractPath/bin" | Out-File -FilePath $env:GITHUB_ENV -Encoding utf8 -Append -ErrorAction SilentlyContinue; } catch { Write-Error "Failed to install KTX-Software: $_"; exit 1; }'

[macos]
install-ktx:
    brew install ktx

[linux]
install-ktx:
    set -e; \
    KTX_VERSION="{{KTX_VERSION}}"; \
    DEB_URL="https://github.com/KhronosGroup/KTX-Software/releases/download/v${KTX_VERSION}/KTX-Software-${KTX_VERSION}-Linux-x86_64.deb"; \
    DEB_FILE="/tmp/ktx.deb"; \
    echo "Downloading KTX-Software from $DEB_URL..."; \
    curl -fsSL "$DEB_URL" -o "$DEB_FILE" || { echo "Failed to download KTX-Software"; exit 1; }; \
    echo "Installing KTX-Software..."; \
    sudo dpkg -i "$DEB_FILE" || { echo "Failed to install KTX-Software"; exit 1; }; \
    rm "$DEB_FILE"; \
    echo "KTX-Software installed successfully"; \
    toktx --version || { echo "toktx not found in PATH"; exit 1; }


install-wasm-prereqs:
    cargo install -f wasm-bindgen-cli --version 0.2.108
    cargo install wasm-server-runner

install-wasm: install-wasm-prereqs
    rustup target install wasm32-unknown-unknown

run-wasm: install-wasm
    cargo run --release --target wasm32-unknown-unknown

watch-wasm:
    cargo watch -cx "run --release --target wasm32-unknown-unknown"

build-wasm: install-wasm
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen --out-dir ./docs/ --target web ./target/wasm32-unknown-unknown/release/arcana.wasm
