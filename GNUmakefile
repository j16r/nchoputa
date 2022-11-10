target ?= debug

wasm = viewer/target/wasm32-unknown-unknown/$(target)/viewer.wasm
binary = target/$(target)/nchoputa

bindgen = wasm-bindgen
cargo_opt =
ifeq ($(target),debug)
	bindgen += --keep-debug --debug --no-demangle
else
	cargo_opt += --release
endif

.PHONY: all
all: static/viewer_bg.wasm static/viewer.js $(binary)

.PHONY: install
install: /usr/local/cargo/bin/nchoputa static/viewer_bg.wasm static/viewer.js

static/viewer_bg.wasm.gz: static/viewer_bg.wasm
	gzip -9 < $< > $@

/usr/local/cargo/bin/nchoputa: $(binary)
	cp $< $@ 

$(binary): Cargo.* src/*.rs
	cargo build $(cargo_opt)

$(wasm): viewer/Cargo.* viewer/src/*.rs shared/src/*.rs
	cd $(<D) && cargo build $(cargo_opt)

static/viewer_bg.wasm static/viewer.js: $(wasm)
	RUST_LOG=warn $(bindgen) --out-dir $(@D) --target web --no-typescript --out-name viewer $<

.PHONY: clean
clean:
	rm -f $(wasm) $(binary) static/viewer.js static/viewer_bg.wasm
