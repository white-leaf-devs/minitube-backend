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

resource "aws_lambda_function" "endpoint" {
  package_type  = "Image"
  image_uri     = "768088100333.dkr.ecr.us-east-1.amazonaws.com/minitube-endpoint:0.2.0"
  function_name = "EndpointLambda"
  role          = aws_iam_role.lambda.arn
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

  source_arn = "${aws_api_gateway_rest_api.endpoint.execution_arn}/*/*"
}

output "endpoint_url" {
  value = aws_api_gateway_deployment.endpoint.invoke_url
}
