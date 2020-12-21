name 	 := "generate-thumbnail"
version  := "0.1.0"
registry := "768088100333.dkr.ecr.us-east-1.amazonaws.com"


build: 
	@docker build -t $(name):$(version) .

tag: build
	@docker tag $(name):$(version) $(registry)/$(name):$(version)

push: tag
	@docker push $(registry)/$(name):$(version)

all: build tag push
