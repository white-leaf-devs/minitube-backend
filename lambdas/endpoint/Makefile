name 	 := "minitube-endpoint"
version  := $(shell cat Cargo.toml | grep -E "^version = .*$$" | cut -d= -f2 | sed 's/[" ]//g')
registry := "768088100333.dkr.ecr.us-east-1.amazonaws.com"


build:
	@docker build -t $(name):$(version) .

tag: build
	@docker tag $(name):$(version) $(registry)/$(name):$(version)

push: tag
	@docker push $(registry)/$(name):$(version)

all: build tag push
