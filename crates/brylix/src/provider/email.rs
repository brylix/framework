//! Email provider for sending emails via SMTP.
//!
//! This module provides an `EmailProvider` trait and an SMTP implementation
//! for sending emails with support for HTML content and file attachments.
//!
//! # Feature
//!
//! This module requires the `email` feature to be enabled.
//!
//! # Usage
//!
//! ```rust,ignore
//! use brylix::provider::email::{EmailAttachment, EmailMessage, EmailProvider, SmtpProvider};
//!
//! // Create provider from environment variables
//! let provider = SmtpProvider::try_from_env();
//!
//! // Build an email message with attachments
//! let message = EmailMessage::new(
//!     "recipient@example.com",
//!     "Monthly Report",
//!     "<h1>Report</h1><p>Please find the attached files.</p>",
//! )
//! .with_reply_to("support@example.com")
//! .with_attachment(EmailAttachment::pdf("report.pdf", pdf_bytes))
//! .with_attachment(EmailAttachment::csv("data.csv", csv_bytes));
//!
//! // Send the email
//! provider.send(message).await?;
//! ```
//!
//! # Attachments
//!
//! The module provides convenience methods for common file types:
//!
//! - [`EmailAttachment::pdf`] - PDF documents
//! - [`EmailAttachment::csv`] - CSV files
//! - [`EmailAttachment::xlsx`] - Excel spreadsheets
//! - [`EmailAttachment::json`] - JSON files
//! - [`EmailAttachment::png`] / [`EmailAttachment::jpeg`] - Images
//! - [`EmailAttachment::zip`] - ZIP archives
//! - [`EmailAttachment::text`] - Plain text files
//! - [`EmailAttachment::new`] - Custom MIME types
//!
//! # Environment Variables
//!
//! The SMTP provider reads the following environment variables:
//!
//! | Variable | Required | Default | Description |
//! |----------|----------|---------|-------------|
//! | `SMTP_HOST` | Yes | - | SMTP server hostname |
//! | `SMTP_PORT` | No | 465 | SMTP server port |
//! | `SMTP_USER` | Yes | - | SMTP username |
//! | `SMTP_PASSWORD` | Yes | - | SMTP password |
//! | `SMTP_FROM_NAME` | No | "Brylix" | Sender display name |
//! | `SMTP_FROM_EMAIL` | No | `SMTP_USER` | Sender email address |

use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::errors::{DomainError, DomainResult};

/// Email attachment structure.
///
/// Represents a file attachment to be included in an email.
///
/// # Example
///
/// ```rust
/// use brylix::provider::email::EmailAttachment;
///
/// // Create attachment from bytes
/// let attachment = EmailAttachment::new(
///     "report.pdf",
///     "application/pdf",
///     pdf_bytes,
/// );
///
/// // Or use convenience methods
/// let pdf = EmailAttachment::pdf("report.pdf", pdf_bytes);
/// let csv = EmailAttachment::csv("data.csv", csv_content.into_bytes());
/// ```
#[derive(Debug, Clone)]
pub struct EmailAttachment {
    /// Filename for the attachment
    pub filename: String,
    /// MIME content type (e.g., "application/pdf", "text/csv")
    pub content_type: String,
    /// Raw bytes of the attachment
    pub data: Vec<u8>,
}

impl EmailAttachment {
    /// Create a new email attachment.
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to display for the attachment
    /// * `content_type` - MIME type (e.g., "application/pdf")
    /// * `data` - Raw bytes of the file
    pub fn new(
        filename: impl Into<String>,
        content_type: impl Into<String>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            filename: filename.into(),
            content_type: content_type.into(),
            data,
        }
    }

    /// Create a PDF attachment.
    pub fn pdf(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(filename, "application/pdf", data)
    }

    /// Create a CSV attachment.
    pub fn csv(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(filename, "text/csv", data)
    }

    /// Create a plain text attachment.
    pub fn text(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(filename, "text/plain", data)
    }

    /// Create a JSON attachment.
    pub fn json(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(filename, "application/json", data)
    }

    /// Create an Excel (.xlsx) attachment.
    pub fn xlsx(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(
            filename,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            data,
        )
    }

    /// Create a PNG image attachment.
    pub fn png(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(filename, "image/png", data)
    }

    /// Create a JPEG image attachment.
    pub fn jpeg(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(filename, "image/jpeg", data)
    }

    /// Create a ZIP archive attachment.
    pub fn zip(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(filename, "application/zip", data)
    }
}

/// Email message structure.
///
/// Use the builder-style methods to construct an email message.
///
/// # Example
///
/// ```rust
/// use brylix::provider::email::{EmailMessage, EmailAttachment};
///
/// let message = EmailMessage::new(
///     "user@example.com",
///     "Subject Line",
///     "<h1>HTML Body</h1>",
/// )
/// .with_reply_to("reply@example.com")
/// .with_attachment(EmailAttachment::pdf("report.pdf", pdf_bytes));
/// ```
#[derive(Debug, Clone)]
pub struct EmailMessage {
    /// Recipient email address
    pub to: String,
    /// Email subject line
    pub subject: String,
    /// HTML body content
    pub html_body: String,
    /// Optional reply-to address
    pub reply_to: Option<String>,
    /// File attachments
    pub attachments: Vec<EmailAttachment>,
}

impl EmailMessage {
    /// Create a new email message.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient email address
    /// * `subject` - Email subject line
    /// * `html_body` - HTML content for the email body
    pub fn new(
        to: impl Into<String>,
        subject: impl Into<String>,
        html_body: impl Into<String>,
    ) -> Self {
        Self {
            to: to.into(),
            subject: subject.into(),
            html_body: html_body.into(),
            reply_to: None,
            attachments: Vec::new(),
        }
    }

    /// Set the reply-to address.
    ///
    /// # Arguments
    ///
    /// * `reply_to` - The reply-to email address
    #[must_use]
    pub fn with_reply_to(mut self, reply_to: impl Into<String>) -> Self {
        self.reply_to = Some(reply_to.into());
        self
    }

    /// Add an attachment to the email.
    ///
    /// # Arguments
    ///
    /// * `attachment` - The attachment to add
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use brylix::provider::email::{EmailMessage, EmailAttachment};
    ///
    /// let message = EmailMessage::new("user@example.com", "Report", "<p>See attached.</p>")
    ///     .with_attachment(EmailAttachment::pdf("report.pdf", pdf_bytes))
    ///     .with_attachment(EmailAttachment::csv("data.csv", csv_bytes));
    /// ```
    #[must_use]
    pub fn with_attachment(mut self, attachment: EmailAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Add multiple attachments to the email.
    ///
    /// # Arguments
    ///
    /// * `attachments` - Iterator of attachments to add
    #[must_use]
    pub fn with_attachments(mut self, attachments: impl IntoIterator<Item = EmailAttachment>) -> Self {
        self.attachments.extend(attachments);
        self
    }

    /// Check if the email has any attachments.
    pub fn has_attachments(&self) -> bool {
        !self.attachments.is_empty()
    }
}

/// Email provider trait for sending emails.
///
/// Implement this trait to create custom email providers (e.g., AWS SES, SendGrid).
#[async_trait]
pub trait EmailProvider: Send + Sync {
    /// Send an email message.
    ///
    /// # Arguments
    ///
    /// * `message` - The email message to send
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `DomainError` on failure
    async fn send(&self, message: EmailMessage) -> DomainResult<()>;

    /// Check if the provider is configured and ready to use.
    fn is_configured(&self) -> bool {
        true
    }
}

/// SMTP email provider configuration.
#[derive(Debug, Clone)]
struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    from_name: String,
    from_email: String,
}

impl SmtpConfig {
    fn from_env() -> DomainResult<Self> {
        let username = std::env::var("SMTP_USER")
            .map_err(|_| DomainError::Internal("SMTP_USER not set".to_string()))?;

        Ok(Self {
            host: std::env::var("SMTP_HOST")
                .map_err(|_| DomainError::Internal("SMTP_HOST not set".to_string()))?,
            port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "465".to_string())
                .parse()
                .map_err(|_| DomainError::Internal("Invalid SMTP_PORT".to_string()))?,
            username: username.clone(),
            password: std::env::var("SMTP_PASSWORD")
                .map_err(|_| DomainError::Internal("SMTP_PASSWORD not set".to_string()))?,
            from_name: std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Brylix".to_string()),
            from_email: std::env::var("SMTP_FROM_EMAIL").unwrap_or(username),
        })
    }
}

/// SMTP email provider implementation.
///
/// Sends emails using the SMTP protocol with TLS encryption.
///
/// # Example
///
/// ```rust,ignore
/// use brylix::provider::email::{EmailMessage, EmailProvider, SmtpProvider};
///
/// let provider = SmtpProvider::try_from_env();
///
/// if provider.is_configured() {
///     let message = EmailMessage::new(
///         "user@example.com",
///         "Hello",
///         "<p>Email body</p>",
///     );
///     provider.send(message).await?;
/// }
/// ```
pub struct SmtpProvider {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    config: SmtpConfig,
    configured: bool,
}

impl SmtpProvider {
    /// Create a new SMTP provider with the given configuration.
    fn new(config: SmtpConfig) -> DomainResult<Self> {
        let creds = Credentials::new(config.username.clone(), config.password.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
            .map_err(|e| DomainError::ExternalService(format!("SMTP relay error: {}", e)))?
            .port(config.port)
            .credentials(creds)
            .build();

        Ok(Self {
            mailer,
            config,
            configured: true,
        })
    }

    /// Create a new SMTP provider from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are not set.
    fn from_env() -> DomainResult<Self> {
        let config = SmtpConfig::from_env()?;
        Self::new(config)
    }

    /// Try to create an SMTP provider from environment variables.
    ///
    /// Returns an unconfigured provider if environment variables are not set,
    /// allowing graceful degradation. The unconfigured provider will log a
    /// warning and skip sending emails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let provider = SmtpProvider::try_from_env();
    /// // Will work even if SMTP is not configured
    /// provider.send(message).await?; // Logs warning if not configured
    /// ```
    #[must_use]
    pub fn try_from_env() -> Self {
        match Self::from_env() {
            Ok(provider) => provider,
            Err(e) => {
                tracing::warn!("SMTP provider not configured: {}", e);
                Self::unconfigured()
            }
        }
    }

    /// Create an unconfigured provider that skips sending emails.
    fn unconfigured() -> Self {
        Self {
            mailer: AsyncSmtpTransport::<Tokio1Executor>::unencrypted_localhost(),
            config: SmtpConfig {
                host: String::new(),
                port: 0,
                username: String::new(),
                password: String::new(),
                from_name: String::new(),
                from_email: String::new(),
            },
            configured: false,
        }
    }
}

#[async_trait]
impl EmailProvider for SmtpProvider {
    async fn send(&self, msg: EmailMessage) -> DomainResult<()> {
        if !self.configured {
            tracing::warn!("SMTP provider not configured, skipping email send");
            return Ok(());
        }

        let from_address = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let mut builder = Message::builder()
            .from(
                from_address
                    .parse()
                    .map_err(|e| DomainError::ExternalService(format!("Invalid from address: {}", e)))?,
            )
            .to(msg
                .to
                .parse()
                .map_err(|e| DomainError::ExternalService(format!("Invalid to address: {}", e)))?)
            .subject(&msg.subject);

        if let Some(reply_to) = &msg.reply_to {
            builder = builder.reply_to(
                reply_to
                    .parse()
                    .map_err(|e| DomainError::ExternalService(format!("Invalid reply-to: {}", e)))?,
            );
        }

        // Build the email body - use multipart if there are attachments
        let email = if msg.attachments.is_empty() {
            // Simple HTML-only email
            builder
                .header(ContentType::TEXT_HTML)
                .body(msg.html_body)
                .map_err(|e| DomainError::ExternalService(format!("Failed to build email: {}", e)))?
        } else {
            // Multipart email with attachments
            let html_part = SinglePart::builder()
                .header(ContentType::TEXT_HTML)
                .body(msg.html_body);

            let mut multipart = MultiPart::mixed().singlepart(html_part);

            for attachment in msg.attachments {
                let content_type: ContentType = attachment
                    .content_type
                    .parse()
                    .unwrap_or_else(|_| "application/octet-stream".parse().expect("valid fallback content type"));

                let attachment_part = Attachment::new(attachment.filename)
                    .body(attachment.data, content_type);

                multipart = multipart.singlepart(attachment_part);
            }

            builder
                .multipart(multipart)
                .map_err(|e| DomainError::ExternalService(format!("Failed to build email: {}", e)))?
        };

        self.mailer
            .send(email)
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to send email: {}", e)))?;

        tracing::info!("Email sent successfully to {}", msg.to);
        Ok(())
    }

    fn is_configured(&self) -> bool {
        self.configured
    }
}

/// A no-op email provider for testing or when email is not needed.
///
/// This provider logs messages instead of sending them.
pub struct NoOpEmailProvider;

#[async_trait]
impl EmailProvider for NoOpEmailProvider {
    async fn send(&self, message: EmailMessage) -> DomainResult<()> {
        tracing::debug!(
            "NoOpEmailProvider: Would send email to {} with subject: {} ({} attachments)",
            message.to,
            message.subject,
            message.attachments.len()
        );
        Ok(())
    }

    fn is_configured(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_message_new() {
        let msg = EmailMessage::new("test@example.com", "Subject", "<p>Body</p>");
        assert_eq!(msg.to, "test@example.com");
        assert_eq!(msg.subject, "Subject");
        assert_eq!(msg.html_body, "<p>Body</p>");
        assert!(msg.reply_to.is_none());
        assert!(msg.attachments.is_empty());
    }

    #[test]
    fn test_email_message_with_reply_to() {
        let msg = EmailMessage::new("test@example.com", "Subject", "<p>Body</p>")
            .with_reply_to("reply@example.com");
        assert_eq!(msg.reply_to, Some("reply@example.com".to_string()));
    }

    #[test]
    fn test_email_attachment_new() {
        let data = b"test content".to_vec();
        let attachment = EmailAttachment::new("test.txt", "text/plain", data.clone());
        assert_eq!(attachment.filename, "test.txt");
        assert_eq!(attachment.content_type, "text/plain");
        assert_eq!(attachment.data, data);
    }

    #[test]
    fn test_email_attachment_convenience_methods() {
        let data = vec![1, 2, 3, 4];

        let pdf = EmailAttachment::pdf("doc.pdf", data.clone());
        assert_eq!(pdf.content_type, "application/pdf");

        let csv = EmailAttachment::csv("data.csv", data.clone());
        assert_eq!(csv.content_type, "text/csv");

        let json = EmailAttachment::json("data.json", data.clone());
        assert_eq!(json.content_type, "application/json");

        let png = EmailAttachment::png("image.png", data.clone());
        assert_eq!(png.content_type, "image/png");

        let xlsx = EmailAttachment::xlsx("sheet.xlsx", data.clone());
        assert!(xlsx.content_type.contains("spreadsheetml"));
    }

    #[test]
    fn test_email_message_with_attachment() {
        let attachment = EmailAttachment::pdf("report.pdf", vec![1, 2, 3]);
        let msg = EmailMessage::new("test@example.com", "Subject", "<p>Body</p>")
            .with_attachment(attachment);

        assert!(msg.has_attachments());
        assert_eq!(msg.attachments.len(), 1);
        assert_eq!(msg.attachments[0].filename, "report.pdf");
    }

    #[test]
    fn test_email_message_with_multiple_attachments() {
        let msg = EmailMessage::new("test@example.com", "Subject", "<p>Body</p>")
            .with_attachment(EmailAttachment::pdf("doc.pdf", vec![1]))
            .with_attachment(EmailAttachment::csv("data.csv", vec![2]));

        assert_eq!(msg.attachments.len(), 2);
    }

    #[test]
    fn test_email_message_with_attachments_batch() {
        let attachments = vec![
            EmailAttachment::pdf("a.pdf", vec![1]),
            EmailAttachment::csv("b.csv", vec![2]),
            EmailAttachment::json("c.json", vec![3]),
        ];

        let msg = EmailMessage::new("test@example.com", "Subject", "<p>Body</p>")
            .with_attachments(attachments);

        assert_eq!(msg.attachments.len(), 3);
    }

    #[tokio::test]
    async fn test_noop_provider() {
        let provider = NoOpEmailProvider;
        let msg = EmailMessage::new("test@example.com", "Test", "<p>Test</p>");
        let result = provider.send(msg).await;
        assert!(result.is_ok());
        assert!(!provider.is_configured());
    }

    #[tokio::test]
    async fn test_noop_provider_with_attachments() {
        let provider = NoOpEmailProvider;
        let msg = EmailMessage::new("test@example.com", "Test", "<p>Test</p>")
            .with_attachment(EmailAttachment::pdf("test.pdf", vec![1, 2, 3]));
        let result = provider.send(msg).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_smtp_provider_unconfigured() {
        // Clear any existing env vars
        std::env::remove_var("SMTP_HOST");
        std::env::remove_var("SMTP_USER");
        std::env::remove_var("SMTP_PASSWORD");

        let provider = SmtpProvider::try_from_env();
        assert!(!provider.is_configured());
    }
}
