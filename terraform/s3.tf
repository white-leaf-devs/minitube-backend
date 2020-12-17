resource "aws_s3_bucket" "videos" {
  bucket = "minitube.videos"
  acl    = "public-read"
}

resource "aws_s3_bucket" "thumbnails" {
  bucket = "minitube.thumbnails"
  acl    = "public-read"
}

resource "aws_s3_bucket" "previews" {
  bucket = "minitube.previews"
  acl    = "public-read"
}

resource "aws_s3_bucket" "code" {
  bucket = "minitube.code"
  acl    = "private"
}
