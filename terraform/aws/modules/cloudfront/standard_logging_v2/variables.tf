variable "bucket_name" {
  description = "Account-regional S3 bucket name for CloudFront access logs."
  type        = string
}

variable "resource_name" {
  description = "Stable name prefix for CloudFront log delivery resources."
  type        = string
}

variable "account_id" {
  description = "AWS account ID used to scope log delivery and lifecycle rules."
  type        = string
}

variable "distribution_arn" {
  description = "CloudFront distribution ARN whose standard access logs are delivered."
  type        = string
}
