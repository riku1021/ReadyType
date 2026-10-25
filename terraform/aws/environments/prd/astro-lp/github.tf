resource "aws_iam_openid_connect_provider" "github" {
  count = var.enable_github_actions_oidc ? 1 : 0

  url            = "https://token.actions.githubusercontent.com"
  client_id_list = ["sts.amazonaws.com"]
}

data "aws_iam_policy_document" "github_deploy_assume_role" {
  count = var.enable_github_actions_oidc ? 1 : 0

  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRoleWithWebIdentity"]

    principals {
      type        = "Federated"
      identifiers = [aws_iam_openid_connect_provider.github[0].arn]
    }

    condition {
      test     = "StringEquals"
      variable = "token.actions.githubusercontent.com:aud"
      values   = ["sts.amazonaws.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "token.actions.githubusercontent.com:sub"
      values   = ["repo:${var.github_repository}:ref:refs/heads/main"]
    }
  }
}

resource "aws_iam_role" "github_deploy" {
  count = var.enable_github_actions_oidc ? 1 : 0

  name               = "${local.resource_name}-github-deploy"
  assume_role_policy = data.aws_iam_policy_document.github_deploy_assume_role[0].json
}

data "aws_iam_policy_document" "github_deploy" {
  statement {
    sid       = "ListSiteBucket"
    effect    = "Allow"
    actions   = ["s3:GetBucketLocation", "s3:ListBucket"]
    resources = [module.static_site.bucket_arn]
  }

  statement {
    sid    = "UpdateSiteObjects"
    effect = "Allow"
    actions = [
      "s3:AbortMultipartUpload",
      "s3:DeleteObject",
      "s3:ListMultipartUploadParts",
      "s3:PutObject",
    ]
    resources = ["${module.static_site.bucket_arn}/*"]
  }
}

resource "aws_iam_role_policy" "github_deploy" {
  count = var.enable_github_actions_oidc ? 1 : 0

  name   = "${local.resource_name}-github-deploy"
  role   = aws_iam_role.github_deploy[0].id
  policy = data.aws_iam_policy_document.github_deploy.json
}
