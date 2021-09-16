target ?= debug

wasm = target/wasm32-unknown-unknown/$(target)/viewer.wasm
binary = target/$(target)/nchoputa

.PHONY: all
all: static/viewer.wasm

$(wasm): viewer/Cargo.* viewer/src/**.rs
	cd "$(<D)" && cargo build

$(binary): Cargo.* src/**.rs $(wasm)
	cargo build

static/viewer.wasm: $(binary)
	cp $< $@

.PHONY: clean
clean:
	rm -f $(binary)
	rm -f $(wasm)
