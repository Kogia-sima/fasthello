BINS = ./turbo-c ./lightning-cpp
BENCHES = \
  target/release/glacial target/release/slow \
  target/release/fast target/release/speedy \
  target/release/turbo ./turbo-c \
  target/release/lightning ./lightning-cpp \
  target/release/ludicrous target/release/handy \
  target/release/serious

CFLAGS = -Wall -O3

all: $(BINS)
	RUSTFLAGS= cargo build --release

./turbo-c: turbo.c
	$(CC) $(CFLAGS) -o turbo-c turbo.c

./lightning-cpp: lightning.cpp
	$(CXX) $(CFLAGS) -o lightning-cpp lightning.cpp

bench: $(BENCHES)
	hyperfine --warmup 2 $(BENCHES) --export-markdown BENCH.md

check: $(BENCHES)
	@for B in $(BENCHES); do echo -n "$$B "; $$B | md5sum ; done

clean:
	-rm -f $(BINS)
	-cargo clean
