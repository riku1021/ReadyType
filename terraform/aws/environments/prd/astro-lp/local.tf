data "aws_caller_identity" "current" {}

locals {
  resource_name           = "${substr(var.site_bucket_name, 0, 40)}-${substr(md5(var.site_bucket_name), 0, 8)}"
  account_regional_suffix = "${data.aws_caller_identity.current.account_id}-${var.aws_region}-an"
  site_bucket_name        = "${var.site_bucket_name}-${local.account_regional_suffix}"
  logs_bucket_name        = "${var.site_bucket_name}-logs-${local.account_regional_suffix}"
}
