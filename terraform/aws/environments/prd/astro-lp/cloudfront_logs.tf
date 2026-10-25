module "cloudfront_access_logs" {
  source = "../../../modules/cloudfront/standard_logging_v2"

  bucket_name      = local.logs_bucket_name
  resource_name    = local.resource_name
  account_id       = data.aws_caller_identity.current.account_id
  distribution_arn = module.static_site.distribution_arn
}
