# Deployment

Brylix applications deploy to AWS Lambda for serverless execution.

## Prerequisites

- AWS CLI configured with credentials
- cargo-lambda installed
- IAM permissions for Lambda deployment

## Build for Lambda

### Using CLI

```bash
brylix build
```

### Manual Build

```bash
# ARM64 (Graviton - recommended, cheaper)
cargo lambda build --release --arm64

# x86_64
cargo lambda build --release
```

Output: `target/lambda/{function-name}/bootstrap`

## Deploy

### Using CLI

```bash
brylix deploy
```

### Manual Deploy

```bash
# First deployment (creates function)
cargo lambda deploy my-api

# With specific profile
AWS_PROFILE=production cargo lambda deploy my-api

# With specific region
AWS_REGION=us-east-1 cargo lambda deploy my-api
```

## Lambda Configuration

### Environment Variables

Set via AWS Console or CLI:

```bash
aws lambda update-function-configuration \
  --function-name my-api \
  --environment "Variables={
    DB_HOST=prod-db.example.com,
    DB_PORT=3306,
    DB_USER=app,
    DB_PASSWORD=secret,
    DB_NAME=production,
    JWT_SECRET=your-production-secret,
    RUST_LOG=info
  }"
```

### Memory & Timeout

```bash
aws lambda update-function-configuration \
  --function-name my-api \
  --memory-size 256 \
  --timeout 30
```

Recommended:
- Memory: 256MB - 512MB
- Timeout: 30 seconds

### Architecture

```bash
# ARM64 (Graviton2) - 20% cheaper, better performance
aws lambda update-function-configuration \
  --function-name my-api \
  --architectures arm64
```

## API Gateway

### HTTP API (Recommended)

```bash
# Create HTTP API
aws apigatewayv2 create-api \
  --name my-api \
  --protocol-type HTTP

# Create Lambda integration
aws apigatewayv2 create-integration \
  --api-id {api-id} \
  --integration-type AWS_PROXY \
  --integration-uri arn:aws:lambda:region:account:function:my-api \
  --payload-format-version 2.0

# Create catch-all route
aws apigatewayv2 create-route \
  --api-id {api-id} \
  --route-key '$default' \
  --target integrations/{integration-id}

# Deploy
aws apigatewayv2 create-stage \
  --api-id {api-id} \
  --stage-name prod \
  --auto-deploy
```

### Function URL (Simpler Alternative)

```bash
aws lambda create-function-url-config \
  --function-name my-api \
  --auth-type NONE
```

## Database Access

### VPC Configuration

For RDS/Aurora in VPC:

```bash
aws lambda update-function-configuration \
  --function-name my-api \
  --vpc-config SubnetIds=subnet-xxx,subnet-yyy,SecurityGroupIds=sg-zzz
```

### RDS Proxy (Recommended)

Use RDS Proxy to manage connections:
- Handles connection pooling
- Supports IAM authentication
- Reduces connection overhead

## Secrets Management

### AWS Secrets Manager

```rust
// In your config loading
let secret = aws_sdk_secretsmanager::Client::new(&aws_config)
    .get_secret_value()
    .secret_id("myapp/production/db")
    .send()
    .await?;
```

### Parameter Store

```bash
# Store secret
aws ssm put-parameter \
  --name "/myapp/production/jwt-secret" \
  --value "your-secret" \
  --type SecureString

# Grant Lambda access
aws lambda update-function-configuration \
  --function-name my-api \
  --environment "Variables={JWT_SECRET_PARAM=/myapp/production/jwt-secret}"
```

## CI/CD

### GitHub Actions

```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install cargo-lambda
        run: pip3 install cargo-lambda

      - name: Build
        run: cargo lambda build --release --arm64

      - name: Configure AWS
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1

      - name: Deploy
        run: cargo lambda deploy my-api
```

## Monitoring

### CloudWatch Logs

Logs automatically stream to CloudWatch. View with:

```bash
aws logs tail /aws/lambda/my-api --follow
```

### X-Ray Tracing

Enable active tracing:

```bash
aws lambda update-function-configuration \
  --function-name my-api \
  --tracing-config Mode=Active
```

## Cold Starts

Minimize cold start latency:

1. **Use ARM64** - Faster startup than x86
2. **Optimize dependencies** - Remove unused features
3. **Provisioned Concurrency** - For critical paths

```bash
aws lambda put-provisioned-concurrency-config \
  --function-name my-api \
  --qualifier prod \
  --provisioned-concurrent-executions 5
```

## Best Practices

1. **Use ARM64** - Better price/performance
2. **Keep functions small** - Faster cold starts
3. **Use RDS Proxy** - Connection management
4. **Enable X-Ray** - Debugging and performance
5. **Set reasonable timeouts** - Don't default to max
6. **Use Secrets Manager** - Never hardcode secrets
