resource "aws_s3_bucket" "videos" {
  bucket = "minitube-videos-1"
  acl    = "public-read"
}

resource "aws_s3_bucket" "thumbnails" {
  bucket = "minitube-thumbnails-1"
  acl    = "public-read"
}

resource "aws_s3_bucket" "previews" {
  bucket = "minitube-previews-1"
  acl    = "public-read"
}
