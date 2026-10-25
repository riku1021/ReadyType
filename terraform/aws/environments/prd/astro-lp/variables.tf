variable "aws_region" {
  description = "Selected AWS Region for regional resources. CloudFront is global."
  type        = string
  default     = "ap-southeast-2"

  validation {
    condition     = var.aws_region == "ap-southeast-2"
    error_message = "This AWS project is restricted to the ap-southeast-2 selected Region."
  }
}

variable "site_bucket_name" {
  description = "Stable account-regional name for the private Astro site bucket."
  type        = string
  default     = "readytype-lp-prod"

  validation {
    condition     = var.site_bucket_name == "readytype-lp-prod"
    error_message = "The production site bucket name is fixed to readytype-lp-prod."
  }
}

variable "github_repository" {
  description = "GitHub repository allowed to deploy, in owner/name form."
  type        = string
  default     = "riku1021/ReadyType"
}

variable "enable_github_actions_oidc" {
  description = "Create the GitHub Actions OIDC provider and least-privilege deployment role."
  type        = bool
  default     = false
}
