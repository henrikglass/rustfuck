
.PHONY: clean

all:
	#rustc -C opt-level=0 -g src/rustfuck.rs -o rustfuck
	rustc -C opt-level=2 src/rustfuck.rs -o rustfuck

clean:
	rm -rf rustfuck
