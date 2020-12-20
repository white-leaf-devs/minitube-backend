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

resource "aws_lambda_function" "label_thumbnail" {
  package_type  = "Image"
  image_uri     = "768088100333.dkr.ecr.us-east-1.amazonaws.com/label-thumbnail:0.2.4"
  function_name = "LabelThumbnailLambda"
  role          = aws_iam_role.lambda.arn
  timeout       = 180
}

resource "aws_s3_bucket_notification" "videos" {
  bucket = aws_s3_bucket.videos.id

  lambda_function {
    lambda_function_arn = aws_lambda_function.generate_preview.arn
    events              = ["s3:ObjectCreated:*"]
  }

  depends_on = [aws_lambda_permission.allow_bucket_videos]
}

resource "aws_lambda_permission" "allow_bucket_videos" {
  statement_id  = "AllowExecutionFromS3Bucket"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.generate_preview.function_name
  principal     = "s3.amazonaws.com"
  source_arn    = aws_s3_bucket.videos.arn
}
