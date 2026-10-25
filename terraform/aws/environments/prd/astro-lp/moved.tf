// Preserve the deployed resources while moving their definitions into reusable modules.
moved {
  from = aws_s3_bucket.site
  to   = module.static_site.aws_s3_bucket.site
}

moved {
  from = aws_s3_bucket_versioning.site
  to   = module.static_site.aws_s3_bucket_versioning.site
}

moved {
  from = aws_s3_bucket_server_side_encryption_configuration.site
  to   = module.static_site.aws_s3_bucket_server_side_encryption_configuration.site
}

moved {
  from = aws_s3_bucket_abac.site
  to   = module.static_site.aws_s3_bucket_abac.site
}

moved {
  from = aws_s3_bucket_policy.site
  to   = module.static_site.aws_s3_bucket_policy.site
}

moved {
  from = aws_cloudfront_origin_access_control.site
  to   = module.static_site.aws_cloudfront_origin_access_control.site
}

moved {
  from = aws_cloudfront_function.astro_routes
  to   = module.static_site.aws_cloudfront_function.astro_routes
}

moved {
  from = aws_cloudfront_distribution.site
  to   = module.static_site.aws_cloudfront_distribution.site
}

moved {
  from = aws_cloudwatch_log_delivery_source.cloudfront_access
  to   = module.cloudfront_access_logs.aws_cloudwatch_log_delivery_source.cloudfront_access
}

moved {
  from = aws_cloudwatch_log_delivery_destination.cloudfront_access
  to   = module.cloudfront_access_logs.aws_cloudwatch_log_delivery_destination.cloudfront_access
}

moved {
  from = aws_cloudwatch_log_delivery.cloudfront_access
  to   = module.cloudfront_access_logs.aws_cloudwatch_log_delivery.cloudfront_access
}

moved {
  from = aws_s3_bucket.cloudfront_logs
  to   = module.cloudfront_access_logs.aws_s3_bucket.cloudfront_logs
}

moved {
  from = aws_s3_bucket_versioning.cloudfront_logs
  to   = module.cloudfront_access_logs.aws_s3_bucket_versioning.cloudfront_logs
}

moved {
  from = aws_s3_bucket_server_side_encryption_configuration.cloudfront_logs
  to   = module.cloudfront_access_logs.aws_s3_bucket_server_side_encryption_configuration.cloudfront_logs
}

moved {
  from = aws_s3_bucket_abac.cloudfront_logs
  to   = module.cloudfront_access_logs.aws_s3_bucket_abac.cloudfront_logs
}

moved {
  from = aws_s3_bucket_lifecycle_configuration.cloudfront_logs
  to   = module.cloudfront_access_logs.aws_s3_bucket_lifecycle_configuration.cloudfront_logs
}

moved {
  from = aws_s3_bucket_policy.cloudfront_logs
  to   = module.cloudfront_access_logs.aws_s3_bucket_policy.cloudfront_logs
}
