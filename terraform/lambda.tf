resource "aws_lambda_function" "generate_preview" {
  package_type  = "Image"
  image_uri     = "768088100333.dkr.ecr.us-east-1.amazonaws.com/generate-preview:0.1.0"
  function_name = "GeneratePreviewLambda"
  role          = aws_iam_role.lambda.arn
  timeout       = 120
}

resource "aws_lambda_function" "generate_thumbnail" {
  package_type  = "Image"
  image_uri     = "768088100333.dkr.ecr.us-east-1.amazonaws.com/generate-thumbnail:0.1.0"
  function_name = "GenerateThumbnailLambda"
  role          = aws_iam_role.lambda.arn
  timeout       = 120
}

resource "aws_lambda_function" "label_thumbnail" {
  package_type  = "Image"
  image_uri     = "768088100333.dkr.ecr.us-east-1.amazonaws.com/label-thumbnail:0.2.4"
  function_name = "LabelThumbnailLambda"
  role          = aws_iam_role.lambda.arn
  timeout       = 120
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
