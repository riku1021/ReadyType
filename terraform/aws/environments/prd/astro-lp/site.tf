module "static_site" {
  source = "../../../modules/cloudfront/static"

  bucket_name   = local.site_bucket_name
  resource_name = local.resource_name
}
