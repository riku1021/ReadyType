resource "aws_s3_bucket" "cloudfront_logs" {
  bucket           = var.bucket_name
  bucket_namespace = "account-regional"
}

resource "aws_s3_bucket_public_access_block" "cloudfront_logs" {
  bucket                  = aws_s3_bucket.cloudfront_logs.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_versioning" "cloudfront_logs" {
  bucket = aws_s3_bucket.cloudfront_logs.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "cloudfront_logs" {
  bucket = aws_s3_bucket.cloudfront_logs.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }

    blocked_encryption_types = ["SSE-C"]
  }
}

resource "aws_s3_bucket_abac" "cloudfront_logs" {
  bucket = aws_s3_bucket.cloudfront_logs.id

  abac_status {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "cloudfront_logs" {
  bucket = aws_s3_bucket.cloudfront_logs.id

  rule {
    id     = "expire-cloudfront-access-logs"
    status = "Enabled"

    filter {
      prefix = "AWSLogs/${var.account_id}/CloudFront/"
    }

    expiration {
      days = 90
    }

    noncurrent_version_expiration {
      noncurrent_days = 1
    }

    abort_incomplete_multipart_upload {
      days_after_initiation = 1
    }
  }

  rule {
    id     = "remove-expired-delete-markers"
    status = "Enabled"

    filter {
      prefix = "AWSLogs/${var.account_id}/CloudFront/"
    }

    expiration {
      expired_object_delete_marker = true
    }
  }
}
