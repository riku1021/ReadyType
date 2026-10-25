output "site_bucket_name" {
  description = "S3 bucket receiving the Astro build."
  value       = module.static_site.bucket_name
}

output "cloudfront_logs_bucket_name" {
  description = "S3 bucket receiving CloudFront standard logging v2 access logs."
  value       = module.cloudfront_access_logs.bucket_name
}

output "cloudfront_distribution_id" {
  description = "CloudFront distribution ID for operational checks."
  value       = module.static_site.distribution_id
}

output "site_url" {
  description = "Initial public URL using the CloudFront default domain."
  value       = "https://${module.static_site.distribution_domain_name}"
}

output "github_deploy_role_arn" {
  description = "Role ARN to configure as the GitHub repository variable AWS_DEPLOY_ROLE_ARN."
  value       = var.enable_github_actions_oidc ? aws_iam_role.github_deploy[0].arn : null
}

output "github_oidc_provider_arn" {
  description = "IAM OIDC provider ARN used by GitHub Actions."
  value       = var.enable_github_actions_oidc ? aws_iam_openid_connect_provider.github[0].arn : null
}
