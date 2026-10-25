# Astro LP の AWS デプロイ

`astro-lp/` は GitHub Actions で検証し、本番デプロイは当面ローカルから手動で実行します。Terraform と共通デプロイスクリプトは、後から GitHub Actions OIDC に切り替えられる構成です。

## Terraform と S3 バケット名

Terraform AWS Provider 6.70.0 は `aws_s3_bucket.bucket_namespace = "account-regional"` をサポートします。この設定では `bucket` にも `<prefix>-<AWS_ACCOUNT_ID>-<REGION>-an` の完全な名前を指定する必要があり、Provider は suffix を自動追加しません。Terraform が AWS account ID と `ap-southeast-2` から決定的に組み立てるため、ランダム suffix は不要で再適用しても名前は変わりません。

Provider の現行仕様: [aws_s3_bucket](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/s3_bucket)

| 用途 | バケット名の prefix | 実際のバケット名 |
| --- | --- | --- |
| Astro の配信元 | `readytype-lp-prod` | `readytype-lp-prod-<AWS_ACCOUNT_ID>-ap-southeast-2-an` |
| CloudFront 標準アクセスログ v2 | `readytype-lp-prod-logs` | `readytype-lp-prod-logs-<AWS_ACCOUNT_ID>-ap-southeast-2-an` |

どちらも `ap-southeast-2` に作成します。両バケットで S3 Block Public Access の4設定を明示的に有効化し、`BucketOwnerEnforced`（ACL 無効）を維持します。CloudFront OAC とログ配送の権限はバケットポリシーに限定します。両バケットは SSE-S3、HTTPS 強制、バージョニング、ABAC を設定します。サイトバケットの非現行バージョンは30日後に削除し、期限切れ delete marker と未完了 multipart upload も整理します。

## CloudFront 標準アクセスログ v2

配信先は `readytype-lp-prod-logs` を prefix とし、account ID・リージョン・`-an` を付けた専用バケットです。Terraform は標準ログ v2 の delivery source / destination / delivery を作成し、S3 Server Access Logging は有効にしません。

CloudFront ログ配送リソースは CloudWatch Logs API の `us-east-1` に作成します。これは CloudFront グローバルサービスのログ配送に必要な設定です。ログバケット自体は選択リージョン `ap-southeast-2` に置きます。

CloudFront 標準ログ v2 の S3 配送は `delivery.logs.amazonaws.com` から `bucket-owner-full-control` ACL を指定して書き込みます。S3 の `BucketOwnerEnforced` は ACL 無効のまま、この ACL と ACL を指定しない PUT のみ受け付けるため互換性があります。Terraform はその ACL、`aws:SourceAccount`、delivery source ARN を条件にした書き込みだけを許可します。

ログのオブジェクトは `AWSLogs/<AWS-account-id>/CloudFront/` 以下に配送します。ライフサイクルで現行ログを90日後に期限切れにし、非現行バージョンと期限切れ delete marker も後続のライフサイクル処理で削除します。S3 のライフサイクル削除は非同期で実行されます。

ログフィールドには日時、リクエストパス、ステータス、転送量、ユーザーエージェント等を含め、Cookie とクエリ文字列は含めません。

仕様: [CloudFront 標準ログ v2](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/standard-logging.html)、[CloudWatch Logs から S3 への配送](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/AWS-logs-infrastructure-V2-S3.html)、[S3 Object Ownership](https://docs.aws.amazon.com/AmazonS3/latest/userguide/about-object-ownership.html)

## GitHub Actions OIDC と AWS 新しいエクスペリエンス

Terraform は `enable_github_actions_oidc` が `false` の間、OIDC プロバイダーと GitHub 用デプロイロールを作りません。これにより現在拒否されている IAM OIDC プロバイダー作成が、S3 / CloudFront 構成全体の適用を妨げません。デフォルトは `false` です。

OIDC 作成の拒否については、IAM シミュレーションで `iam:ListOpenIDConnectProviders` と `iam:CreateOpenIDConnectProvider` が `OrganizationsDecisionDetail.AllowedByOrganizations = false` になりました。AWS Organizations のポリシー本文は現在の権限では取得できず、拒否した SCP の個別 statement までは特定できていません。AWS の新しいエクスペリエンスでは AWS 管理の SCP/RCP が適用され、利用者は直接変更できません。

将来 GitHub Actions デプロイへ切り替える際は、OIDC 拒否を解消したうえで `enable_github_actions_oidc = true` を設定し、Terraform を再適用します。信頼ポリシーは `riku1021/ReadyType` の `main` ブランチと `sts.amazonaws.com` audience に限定され、デプロイ権限は配信バケット内のオブジェクト操作に絞られます。


高度な機能を有効にすると AWS Organization と SCP/RCP を利用者が管理できます。有効化には Paid Plan への変更が必要です。高度な機能の有効化は取り消せず、設定済み Spend Limit は解除され、以後設定できません。これは人が費用・ガバナンス変更を理解したうえで判断する操作です。Agent は有効化しません。

このプロジェクトは直近の確認時点で Free Plan でした。そのため高度な機能を使うには Paid Plan へのアップグレードも必要です。

1. 必要なら [AWS Settings](https://settings.aws.com/) の **Projects → Actions → Explore advanced features** で影響を確認し、AWS の案内に従って有効化します。
2. 有効化後に AWS IAM で GitHub OIDC provider の一覧を取得します。既存なら Terraform state に import し、なければ Terraform で作成します。
3. 同じ拒否が続く場合は、AWS Support に管理 SCP の拒否 statement と対象 API の解除可否を問い合わせます。管理 SCP を迂回したり、長期アクセスキーへ切り替えたりしません。

公式資料: [新しい AWS エクスペリエンスのサポートサービス](https://docs.aws.amazon.com/accounts/latest/reference/supported-services-sign-up-new.html)、[高度な機能の有効化と影響](https://docs.aws.amazon.com/accounts/latest/reference/activate-advanced-features.html)、[組織の管理ポリシー](https://docs.aws.amazon.com/accounts/latest/reference/scps-and-rcps-for-projects.html)

## AWS リソースの作成（手動デプロイ用）

Terraform は環境ルートを `terraform/aws/environments/prd/astro-lp/`、再利用モジュールを `terraform/aws/modules/cloudfront/` に置きます。`static/` はS3＋CloudFront配信、`standard_logging_v2/` はCloudFront標準ログ配送を管理します。GitHub OIDCロールは ReadyType の環境ルートに残します。

Terraform state はローカル保存です。同じディレクトリの `terraform.tfstate` を安全に保管してください。今回の環境ルート移動ではstateファイルも新しいルートへ移動し、既存リソースのアドレスは `moved` ブロックでモジュールへ移行します。チーム共有に移行する場合は、事前に Terraform state backend を設計します。

```bash
mise run astro-lp:infra:plan
# planを確認してから、apply時に再表示されるplanも承認する
mise run astro-lp:infra:apply
```

`terraform.tfvars` がまだない場合は、初回に `terraform/aws/environments/prd/astro-lp/terraform.tfvars.example` をコピーして作成します。デフォルト値は `ap-southeast-2`、サイトバケット `readytype-lp-prod`、GitHub リポジトリ `riku1021/ReadyType` です。`enable_github_actions_oidc` は既定で無効なので、OIDC 拒否を解消する前でも S3 バケット、CloudFront、ログ配送を作成できます。選択リージョンは他の値へ変更しないでください。

`astro-lp:infra:plan` はTerraform初期化・書式確認・validate・planを実行します。`astro-lp:infra:apply` は同じ検証を行ってから現在の構成でplanを再生成し、Terraformの確認プロンプト後に適用します。サイトの通常更新にはインフラapplyは含まれず、次の `astro-lp:deploy` を使います。

## 手動デプロイ

Terraform 適用後、次のコマンドでサイトを検証・ビルドし、S3 に同期します。`readytype` プロファイルの有効な認証情報が必要です。

```bash
AWS_PROFILE=readytype \
AWS_REGION=ap-southeast-2 \
AWS_S3_BUCKET="$(terraform -chdir=terraform/aws/environments/prd/astro-lp output -raw site_bucket_name)" \
./scripts/deploy-astro-lp.sh
```

スクリプトは依存関係をインストールし、lint・typecheck・Astro build の順に実行します。すべて成功してから S3 を同期します。HTML 等には再検証を要求し、`_astro/` のハッシュ付きアセットには1年間の immutable cache-control を設定します。古いハッシュ付きアセットは、デプロイ済みページが参照し続ける場合に備えて同期時に削除しません。HTML はCloudFrontでキャッシュ・圧縮せず、キャッシュする`/_astro/*`アセットは gzip / Brotli 圧縮します。CloudFrontはキャッシュ無効ポリシーと圧縮を併用できないため、この設定にしています。ハッシュ付きファイル名で内容が変わるため、通常デプロイ時の CloudFront invalidation は行いません。AWS CLI が必要です。

## GitHub Actions 自動デプロイへの切り替え

現在 [.github/workflows/astro-lp-ci.yml](../.github/workflows/astro-lp-ci.yml) は `main` への push、pull request、手動実行で検証・ビルドのみを行います。AWS 権限は付与していません。

OIDC の SCP 拒否を解消した後、以下の順に切り替えます。

1. `terraform/aws/environments/prd/astro-lp/terraform.tfvars` に `enable_github_actions_oidc = true` を設定し、Terraform を再適用します。
2. `github_deploy_role_arn` 出力値を GitHub Actions の Repository Variable `AWS_DEPLOY_ROLE_ARN` に登録します。
3. 自動デプロイ用 job に `id-token: write` と `aws-actions/configure-aws-credentials` を設定し、`AWS_REGION` と `AWS_S3_BUCKET` を環境変数として `./scripts/deploy-astro-lp.sh` を実行します。

この手順でも長期アクセスキーは使いません。ローカル手動デプロイと GitHub Actions で同じスクリプトを使います。

## デプロイ後の確認と後片付け

```bash
cd terraform/aws/environments/prd/astro-lp
terraform output -raw site_url
```

CloudFront URL でトップ、プライバシー、利用規約のページを確認し、ログバケットに `AWSLogs/<AWS-account-id>/CloudFront/` が作られることを確認してください。Terraform が作成したリソースの削除が必要になった場合は、先に `terraform plan -destroy` の内容を確認します。
