variable "bucket_name" {
  description = "Account-regional S3 bucket name used as the private site origin."
  type        = string
}

variable "resource_name" {
  description = "Stable name prefix for CloudFront resources."
  type        = string
}
