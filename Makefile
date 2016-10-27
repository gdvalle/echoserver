PREFIX?=/usr/local
TARGET?=x86_64-unknown-linux-musl
BIN_PATH=target/$(TARGET)/release/echoserver
RUST_DISTRIBUTION?=nightly

.PHONY: build compress bootstrap install clean

all: build compress

build:
	cargo build --release --target=$(TARGET)

compress:
	strip -s $(BIN_PATH)
	upx --brute $(BIN_PATH)

bootstrap:
	curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$(RUST_DISTRIBUTION)
	$(HOME)/.cargo/bin/rustup target add $(TARGET)

install:
	install -D -m 0755 -t $(DESTDIR)$(PREFIX)/bin $(BIN_PATH)

clean:
	rm -r target
