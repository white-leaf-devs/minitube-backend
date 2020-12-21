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
