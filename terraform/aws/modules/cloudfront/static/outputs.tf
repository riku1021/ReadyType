output "bucket_name" {
  description = "S3 bucket receiving the static site build."
  value       = aws_s3_bucket.site.bucket
}

output "bucket_arn" {
  description = "ARN of the private S3 site bucket."
  value       = aws_s3_bucket.site.arn
}

output "distribution_id" {
  description = "CloudFront distribution ID."
  value       = aws_cloudfront_distribution.site.id
}

output "distribution_arn" {
  description = "CloudFront distribution ARN."
  value       = aws_cloudfront_distribution.site.arn
}

output "distribution_domain_name" {
  description = "CloudFront distribution domain name."
  value       = aws_cloudfront_distribution.site.domain_name
}
