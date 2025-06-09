TARGET_DIR = ./target/wasm32-unknown-unknown/release
RAW_WASM = $(TARGET_DIR)/art_of_constellation.wasm
OPTIMIZED_WASM = ./art_of_constellation_opt.wasm
DEBUG = 0
ifeq ($(DEBUG), 1)
	DEBUG_FLAGS = -g
else
	DEBUG_FLAGS =
endif

.PHONY: opt

opt: $(OPTIMIZED_WASM)

$(OPTIMIZED_WASM): $(RAW_WASM)
	wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code $(RAW_WASM) -o $(OPTIMIZED_WASM)
	wasm-opt --strip-debug --strip-producers --ignore-implicit-traps --zero-filled-memory --traps-never-happen --flatten -Oz $(DEBUG_FLAGS) $(OPTIMIZED_WASM) -o $(OPTIMIZED_WASM)
