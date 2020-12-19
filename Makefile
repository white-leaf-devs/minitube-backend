opencv_layer := "https://srv-store5.gofile.io/download/4C7cB0/opencv-layer.zip"

setup:
	@./scripts/download.sh $(opencv_layer) layers/opencv-layer.zip
	@./scripts/zip.sh lambdas/generate-preview generate-preview.zip
	@./scripts/zip.sh lambdas/generate-thumbnails generate-thumbnails.zip

clean:
	@rm -rf *.zip

apply: | setup
	@cd terraform; terraform apply

destroy:
	@cd terraform; terraform destroy

docs:
	@redoc-cli bundle -o docs/index.html api/openapi.yml