opencv_layer := "https://srv-store5.gofile.io/download/4C7cB0/opencv-layer.zip"

apply:
	@cd terraform; terraform apply

destroy:
	@cd terraform; terraform destroy

docs: api/openapi.yml
	@redoc-cli bundle -o docs/index.html api/openapi.yml

clean-buckets:
	@aws s3 rm s3://minitube.videos --recursive
	@aws s3 rm s3://minitube.previews --recursive 
	@aws s3 rm s3://minitube.thumbnails --recursive

clean-dynamodb:
	@aws dynamodb scan --table-name "Labels" --attributes-to-get "Label" \
		--query "Items[].Label.S" --output text | tr "\t" "\n" |\
		xargs -t -I keyval aws dynamodb delete-item --table-name "Labels" \
		--key "{ \"Label\": { \"S\": \"keyval\" } }"
		