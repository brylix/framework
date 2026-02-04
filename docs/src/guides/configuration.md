# Configuration

Brylix uses environment variables for configuration, supporting both `.env` files and system environment variables.

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DB_HOST` | Database hostname | `localhost` |
| `DB_PORT` | Database port | `3306` |
| `DB_USER` | Database username | `root` |
| `DB_PASSWORD` | Database password | `secret` |
| `DB_NAME` | Database name | `myapp` |
| `JWT_SECRET` | Secret for JWT signing | `your-secret-key` |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level | `info` |
| `MULTI_TENANT_MODE` | Enable multi-tenancy | `false` |
| `BRYLIX_DB_PASSWORD` | Tenant DB password | - |
| `REQUIRED_DB_VERSION` | Migration version check | `1` |

## Configuration in Code

### Basic Usage

```rust
use brylix::prelude::*;

let config = BrylixConfig::from_env()?;
```

### Builder Pattern

For programmatic configuration:

```rust
use brylix::config::BrylixConfigBuilder;

let config = BrylixConfigBuilder::new()
    .database_url("mysql://user:pass@localhost/myapp")
    .jwt_secret("my-secret")
    .jwt_expiry_hours(24)
    .multi_tenant(false)
    .build()?;
```

### Custom Configuration

Extend with your own settings:

```rust
use brylix::prelude::*;

pub struct AppConfig {
    pub brylix: BrylixConfig,
    pub api_key: String,
    pub feature_flags: FeatureFlags,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            brylix: BrylixConfig::from_env()?,
            api_key: std::env::var("API_KEY")?,
            feature_flags: FeatureFlags::from_env()?,
        })
    }
}
```

## Database Configuration

### MySQL

```env
DB_HOST=localhost
DB_PORT=3306
DB_USER=root
DB_PASSWORD=password
DB_NAME=myapp
```

### PostgreSQL

```env
DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_PASSWORD=password
DB_NAME=myapp
```

## JWT Configuration

```env
# Secret key for signing tokens (use a strong random value)
JWT_SECRET=your-256-bit-secret-key

# Token expiry in hours (default: 24)
JWT_EXPIRY_HOURS=24
```

## Multi-Tenant Configuration

```env
# Enable multi-tenant mode
MULTI_TENANT_MODE=true

# Password for tenant database connections
BRYLIX_DB_PASSWORD=tenant-db-password

# Required database version for migrations
REQUIRED_DB_VERSION=1
```

## S3 Configuration

For file uploads via presigned URLs (requires `s3` feature):

```env
# Required
S3_BUCKET=my-app-uploads

# Optional
S3_REGION=us-east-1
S3_UPLOAD_EXPIRES_SECS=3600
S3_DOWNLOAD_EXPIRES_SECS=3600
```

AWS credentials are loaded via the standard credential chain:
- Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
- IAM role (recommended for Lambda)
- AWS profile

## Environment-Specific Config

### Development (.env)

```env
RUST_LOG=debug
DB_HOST=localhost
DB_NAME=myapp_dev
```

### Production

Set environment variables in AWS Lambda configuration or use AWS Secrets Manager:

```bash
aws lambda update-function-configuration \
  --function-name my-api \
  --environment "Variables={DB_HOST=prod-db.example.com,JWT_SECRET=...}"
```

## Best Practices

1. **Never commit `.env` files** - Add to `.gitignore`
2. **Use strong JWT secrets** - At least 256 bits of entropy
3. **Rotate secrets regularly** - Especially in production
4. **Use AWS Secrets Manager** - For sensitive production values
5. **Validate at startup** - Fail fast if config is invalid
