resource "aws_dynamodb_table" "labels" {
  name         = "Labels"
  hash_key     = "Label"
  billing_mode = "PAY_PER_REQUEST"

  attribute {
    name = "Label"
    type = "S"
  }
}
