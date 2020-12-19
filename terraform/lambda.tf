resource "aws_iam_role" "lambda" {
  name = "IamForLambda"

  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": "sts:AssumeRole",
      "Principal": {
        "Service": "lambda.amazonaws.com"
      },
      "Effect": "Allow",
      "Sid": ""
    }
  ]
}
EOF
}

resource "aws_iam_role_policy_attachment" "lambda" {
  role       = aws_iam_role.lambda.name
  policy_arn = "arn:aws:iam::aws:policy/AWSLambdaFullAccess"
}

resource "aws_lambda_layer_version" "opencv" {
  layer_name          = "OpenCVLayer"
  s3_bucket           = aws_s3_bucket.code.id
  s3_key              = aws_s3_bucket_object.opencv_layer.id
  compatible_runtimes = ["python3.8"]
}

resource "aws_lambda_function" "generate_preview" {
  package_type  = "Zip"
  function_name = "GeneratePreviewLambda"
  role          = aws_iam_role.lambda.arn
  s3_bucket     = aws_s3_bucket.code.id
  s3_key        = aws_s3_bucket_object.generate_preview.id
  handler       = "main.lambda_handler"
  runtime       = "python3.8"
  timeout       = 180

  layers = [
    aws_lambda_layer_version.opencv.arn,
    "arn:aws:lambda:us-east-1:770693421928:layer:Klayers-python38-Pillow:7"
  ]
}

resource "aws_lambda_function" "generate_thumbnails" {
  package_type  = "Zip"
  function_name = "GenerateThumbnailsLambda"
  role          = aws_iam_role.lambda.arn
  s3_bucket     = aws_s3_bucket.code.id
  s3_key        = aws_s3_bucket_object.generate_thumbnails.id
  handler       = "main.lambda_handler"
  runtime       = "python3.8"
  timeout       = 180

  layers = [
    aws_lambda_layer_version.opencv.arn,
  ]
}

resource "aws_lambda_function" "endpoint" {
  package_type  = "Image"
  image_uri     = "768088100333.dkr.ecr.us-east-1.amazonaws.com/minitube-endpoint:0.3.0"
  function_name = "EndpointLambda"
  role          = aws_iam_role.lambda.arn
}

resource "aws_lambda_function" "label_thumbnail" {
  package_type  = "Image"
  image_uri     = "768088100333.dkr.ecr.us-east-1.amazonaws.com/label-thumbnail:0.1.0"
  function_name = "LabelThumbnailLambda"
  role          = aws_iam_role.lambda.arn
}

resource "aws_s3_bucket_notification" "previews_notification" {
  bucket = aws_s3_bucket.videos.id

  lambda_function {
    lambda_function_arn = aws_lambda_function.generate_preview.arn
    events              = ["s3:ObjectCreated:*"]
  }

  depends_on = [aws_lambda_permission.allow_bucket_videos]
}

resource "aws_s3_bucket_notification" "thumbnails_notification" {
  bucket = aws_s3_bucket.thumbnails.id

  lambda_function {
    lambda_function_arn = aws_lambda_function.label_thumbnail.arn
    events              = ["s3:ObjectCreated:*"]
  }

  depends_on = [aws_lambda_permission.allow_bucket_thumbnails]
}

resource "aws_api_gateway_integration" "endpoint" {
  rest_api_id = aws_api_gateway_rest_api.endpoint.id
  resource_id = aws_api_gateway_method.proxy.resource_id
  http_method = aws_api_gateway_method.proxy.http_method

  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.endpoint.invoke_arn
}

resource "aws_api_gateway_integration" "endpoint_root" {
  rest_api_id = aws_api_gateway_rest_api.endpoint.id
  resource_id = aws_api_gateway_method.proxy_root.resource_id
  http_method = aws_api_gateway_method.proxy_root.http_method

  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.endpoint.invoke_arn
}

resource "aws_api_gateway_deployment" "endpoint" {
  depends_on = [
    aws_api_gateway_integration.endpoint,
    aws_api_gateway_integration.endpoint_root,
  ]

  rest_api_id = aws_api_gateway_rest_api.endpoint.id
  stage_name  = "test"
}

resource "aws_lambda_permission" "apigw" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.endpoint.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.endpoint.execution_arn}/*/*"
}


resource "aws_lambda_permission" "allow_bucket_videos" {
  statement_id  = "AllowExecutionFromS3Bucket"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.generate_preview.function_name
  principal     = "s3.amazonaws.com"
  source_arn    = aws_s3_bucket.videos.arn
}

resource "aws_lambda_permission" "allow_bucket_thumbnails" {
  statement_id  = "AllowExecutionFromS3Bucket"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.label_thumbnail.function_name
  principal     = "s3.amazonaws.com"
  source_arn    = aws_s3_bucket.thumbnails.arn
}

output "endpoint_url" {
  value = aws_api_gateway_deployment.endpoint.invoke_url
}
