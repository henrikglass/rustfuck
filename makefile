.PHONY: clean

all:
	#rustc -C opt-level=0 -g src/rustfucked.rs -o rustfucked
	rustc -C opt-level=2 src/rustfucked.rs -o rustfucked

clean:
	rm -rf rustfucked
