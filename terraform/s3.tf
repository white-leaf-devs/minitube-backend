resource "aws_s3_bucket" "videos" {
  bucket = "minitube.videos"
  acl    = "private"
}

resource "aws_s3_bucket" "thumbnails" {
  bucket = "minitube.thumbnails"
  acl    = "private"
}

resource "aws_s3_bucket" "previews" {
  bucket = "minitube.previews"
  acl    = "private"
}

resource "aws_s3_bucket" "code" {
  bucket = "minitube.code"
  acl    = "private"
}

resource "aws_s3_bucket_object" "opencv_layer" {
  bucket = aws_s3_bucket.code.id
  key    = "opencv-layer.zip"
  source = "../layers/opencv-layer.zip"
  etag   = filemd5("../layers/opencv-layer.zip")
}

resource "aws_s3_bucket_object" "generate_preview" {
  bucket = aws_s3_bucket.code.id
  key    = "generate-preview.zip"
  source = "../generate-preview.zip"
  etag   = filemd5("../generate-preview.zip")
}

resource "aws_s3_bucket_object" "generate_thumbnails" {
  bucket = aws_s3_bucket.code.id
  key    = "generate-thumbnails.zip"
  source = "../generate-thumbnails.zip"
  etag   = filemd5("../generate-thumbnails.zip")
}
