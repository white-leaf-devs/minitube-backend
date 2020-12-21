opencv_layer := "https://srv-store5.gofile.io/download/4C7cB0/opencv-layer.zip"

apply:
	@cd terraform; terraform apply

destroy:
	@cd terraform; terraform destroy

docs: api/openapi.yml
	@redoc-cli bundle -o docs/index.html api/openapi.yml
