all: docker

docker: output Dockerfile
	docker build --rm -t cattha/catt-rs .

output:
	script/builder.sh

clean:
	rm -rf output target

.PHONY: fs docker all output clean
