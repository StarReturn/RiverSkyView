use regex::Regex;
use std::sync::OnceLock;

/// 敏感信息脱敏：用于日志输出前过滤密码、令牌、密钥等
pub struct Redaction;

static SECRET_PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();

impl Redaction {
    fn patterns() -> &'static Vec<(Regex, &'static str)> {
        SECRET_PATTERNS.get_or_init(|| {
            vec![
                // 密码键值对
                (Regex::new(r"(?i)(password|passwd|pwd)\s*[:=]\s*\S+").unwrap(), "$1=***REDACTED***"),
                // 令牌键值对
                (Regex::new(r"(?i)(token|api_key|apikey|secret|access_key|secret_key)\s*[:=]\s*\S+").unwrap(), "$1=***REDACTED***"),
                // Bearer 令牌
                (Regex::new(r"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*").unwrap(), "bearer ***REDACTED***"),
                // 私钥标记
                (Regex::new(r"-----BEGIN [A-Z ]+PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+PRIVATE KEY-----").unwrap(), "***PRIVATE KEY REDACTED***"),
                // 连接字符串中的密码
                (Regex::new(r"(?i)(mongodb|postgres|mysql|redis|amqp)://[^:]+:[^@]+@").unwrap(), "$1://***:***@"),
                // AWS 密钥
                (Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(), "***AWS_KEY_REDACTED***"),
                // 测试秘密标记
                (Regex::new(r"PM_TEST_SECRET_[A-Za-z0-9_]+").unwrap(), "***TEST_SECRET_REDACTED***"),
            ]
        })
    }

    /// 脱敏文本中的敏感信息
    pub fn redact(text: &str) -> String {
        let mut result = text.to_string();
        for (regex, replacement) in Self::patterns() {
            result = regex.replace_all(&result, *replacement).to_string();
        }
        result
    }

    /// 检查文本是否包含疑似敏感信息
    pub fn contains_sensitive(text: &str) -> bool {
        Self::patterns()
            .iter()
            .any(|(regex, _)| regex.is_match(text))
    }

    /// 安全地记录日志（先脱敏）
    pub fn log_info(msg: &str) {
        tracing::info!("{}", Self::redact(msg));
    }

    pub fn log_warn(msg: &str) {
        tracing::warn!("{}", Self::redact(msg));
    }

    pub fn log_error(msg: &str) {
        tracing::error!("{}", Self::redact(msg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_password() {
        let input = "password=MySecret123";
        let redacted = Redaction::redact(input);
        assert!(!redacted.contains("MySecret123"));
        assert!(redacted.contains("REDACTED"));
    }

    #[test]
    fn test_redact_token() {
        let input = "api_key=sk-1234567890abcdef";
        let redacted = Redaction::redact(input);
        assert!(!redacted.contains("sk-1234567890abcdef"));
    }

    #[test]
    fn test_redact_bearer() {
        let input = "Authorization: Bearer abc123def456";
        let redacted = Redaction::redact(input);
        assert!(!redacted.contains("abc123def456"));
    }

    #[test]
    fn test_redact_connection_string() {
        let input = "mongodb://user:pass@host:27017/db";
        let redacted = Redaction::redact(input);
        assert!(!redacted.contains("pass"));
    }

    #[test]
    fn test_redact_test_secret() {
        let input = "PM_TEST_SECRET_7f31a9_DO_NOT_USE";
        let redacted = Redaction::redact(input);
        assert!(!redacted.contains("PM_TEST_SECRET_7f31a9"));
        assert!(redacted.contains("REDACTED"));
    }

    #[test]
    fn test_contains_sensitive() {
        assert!(Redaction::contains_sensitive("password=admin"));
        assert!(!Redaction::contains_sensitive("hello world"));
    }
}
