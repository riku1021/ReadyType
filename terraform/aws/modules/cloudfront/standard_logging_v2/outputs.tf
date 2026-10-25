output "bucket_name" {
  description = "S3 bucket receiving CloudFront standard logging v2 access logs."
  value       = aws_s3_bucket.cloudfront_logs.bucket
}
