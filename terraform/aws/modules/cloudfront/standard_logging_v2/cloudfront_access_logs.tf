resource "aws_cloudwatch_log_delivery_source" "cloudfront_access" {
  region       = "us-east-1"
  name         = "${var.resource_name}-access-logs"
  log_type     = "ACCESS_LOGS"
  resource_arn = var.distribution_arn
}

resource "aws_cloudwatch_log_delivery_destination" "cloudfront_access" {
  region        = "us-east-1"
  name          = "${var.resource_name}-s3-access-logs"
  output_format = "json"

  delivery_destination_configuration {
    destination_resource_arn = aws_s3_bucket.cloudfront_logs.arn
  }
}

resource "aws_cloudwatch_log_delivery" "cloudfront_access" {
  region                   = "us-east-1"
  delivery_source_name     = aws_cloudwatch_log_delivery_source.cloudfront_access.name
  delivery_destination_arn = aws_cloudwatch_log_delivery_destination.cloudfront_access.arn
  record_fields = [
    "date",
    "time",
    "x-edge-location",
    "sc-bytes",
    "c-ip",
    "cs-method",
    "cs(Host)",
    "cs-uri-stem",
    "sc-status",
    "cs(User-Agent)",
    "x-edge-result-type",
    "time-taken",
    "cs-protocol",
    "ssl-protocol",
    "cs-protocol-version",
  ]

  depends_on = [aws_s3_bucket_policy.cloudfront_logs]
}
