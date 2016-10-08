CARGO_CMD="cargo build --release --verbose"

DOCKER_CMD="docker run -ti --rm -v $(PWD):/source cattha/rust:nightly make docker_fs"

all: cargo

docker: docker_fs Dockerfile
	docker build --rm -t cattha/catt-rs .

cargo:
ifndef BUILD_DOCKER
	eval $(CARGO_CMD)
else
	eval $(DOCKER_CMD)
endif

docker_fs: cargo
ifndef BUILD_DOCKER
	mkdir -p docker_fs/bin docker_fs/lib64
	BINS=$$(find target/release -maxdepth 1 -type f -executable); \
	for bin in $$BINS; do \
		cp $$bin docker_fs/bin/; \
		deps=$$(ldd $$bin | \
        sed -e '/=>/s/.*=> //' -e 's/\t*//' -e 's/ \(.*\)//'); \
		for dep in $$deps; do \
			if [ -e "$$dep" ]; then \
				dep_name=$$(basename "$$dep"); \
				if [ ! -e "docker_fs/lib64/$$dep_name" ]; then \
					cp -L "$$dep" "docker_fs/lib64/$$dep_name"; \
					chmod 755 "docker_fs/lib64/$$dep_name"; \
				fi; \
			fi; \
		done; \
	done
endif

clean:
	rm -rf docker_fs target

.PHONY: fs docker all docker_fs clean cargo
