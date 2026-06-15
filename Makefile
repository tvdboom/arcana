KTX_VERSION ?= 4.3.2

run:
	cargo run

build:
	cargo build

build-release:
	cargo build --release

# Install KTX-Software (provides toktx) – required when process-assets feature is enabled.
ifeq ($(OS),Windows_NT)
install-ktx:
	powershell -Command " \
		Invoke-WebRequest -Uri 'https://github.com/KhronosGroup/KTX-Software/releases/download/v$(KTX_VERSION)/KTX-Software-$(KTX_VERSION)-Windows-x64.exe' -OutFile ktx-installer.exe; \
		Start-Process -FilePath ktx-installer.exe -ArgumentList '/S' -Wait; \
		Remove-Item ktx-installer.exe"
else
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
install-ktx:
	brew install ktx
else
install-ktx:
	curl -fsSL "https://github.com/KhronosGroup/KTX-Software/releases/download/v$(KTX_VERSION)/KTX-Software-$(KTX_VERSION)-Linux-x86_64.deb" -o /tmp/ktx.deb
	sudo dpkg -i /tmp/ktx.deb
	rm /tmp/ktx.deb
endif
endif

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
