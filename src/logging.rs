//! 日志系统配置模块
//!
//! 本模块提供统一的日志系统配置，支持多种输出格式和日志级别。

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 日志级别配置
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    /// 追踪级别（最详细）
    Trace,
    /// 调试级别
    Debug,
    /// 信息级别
    Info,
    /// 警告级别
    Warn,
    /// 错误级别
    Error,
}

impl LogLevel {
    /// 转换为tracing的过滤器字符串
    fn as_filter_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

/// 日志输出格式
#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    /// 普通文本格式（人类可读）
    Pretty,
    /// 紧凑格式
    Compact,
    /// JSON格式（机器可读，生产环境推荐）
    Json,
}

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// 日志级别
    pub level: LogLevel,
    /// 输出格式
    pub format: LogFormat,
    /// 是否显示代码位置（文件名和行号）
    pub show_location: bool,
    /// 是否显示线程ID
    pub show_thread_id: bool,
    /// 是否显示时间戳
    pub show_timestamp: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Pretty,
            show_location: true,
            show_thread_id: false,
            show_timestamp: true,
        }
    }
}

/// 初始化日志系统
///
/// # 示例
///
/// ```no_run
/// use bitcoin_simulation::logging::{init_logging, LogConfig, LogLevel, LogFormat};
///
/// // 使用默认配置
/// init_logging(&LogConfig::default());
///
/// // 自定义配置
/// let config = LogConfig {
///     level: LogLevel::Debug,
///     format: LogFormat::Pretty,
///     show_location: true,
///     show_thread_id: false,
///     show_timestamp: true,
/// };
/// init_logging(&config);
/// ```
pub fn init_logging(config: &LogConfig) {
    // 从环境变量读取日志级别，如果没有设置则使用配置的级别
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "bitcoin_simulation={},tower_http=info,axum=info",
            config.level.as_filter_str()
        ))
    });

    match config.format {
        LogFormat::Pretty => {
            // 美化格式 - 适合开发环境
            let fmt_layer = fmt::layer()
                .pretty()
                .with_thread_ids(config.show_thread_id)
                .with_file(config.show_location)
                .with_line_number(config.show_location)
                .with_target(true);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Compact => {
            // 紧凑格式 - 适合日志文件
            let fmt_layer = fmt::layer()
                .compact()
                .with_thread_ids(config.show_thread_id)
                .with_file(config.show_location)
                .with_line_number(config.show_location)
                .with_target(true);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Json => {
            // JSON格式 - 适合生产环境和日志分析
            let fmt_layer = fmt::layer()
                .json()
                .with_thread_ids(config.show_thread_id)
                .with_file(config.show_location)
                .with_line_number(config.show_location)
                .with_target(true)
                .with_current_span(true);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .init();
        }
    }
}

/// 快速初始化 - 使用默认配置
///
/// 等同于 `init_logging(&LogConfig::default())`
pub fn init_default_logging() {
    init_logging(&LogConfig::default());
}

/// 开发环境日志配置
pub fn dev_log_config() -> LogConfig {
    LogConfig {
        level: LogLevel::Debug,
        format: LogFormat::Pretty,
        show_location: true,
        show_thread_id: false,
        show_timestamp: true,
    }
}

/// 生产环境日志配置
pub fn prod_log_config() -> LogConfig {
    LogConfig {
        level: LogLevel::Info,
        format: LogFormat::Json,
        show_location: false,
        show_thread_id: true,
        show_timestamp: true,
    }
}

/// 测试环境日志配置
pub fn test_log_config() -> LogConfig {
    LogConfig {
        level: LogLevel::Trace,
        format: LogFormat::Compact,
        show_location: true,
        show_thread_id: false,
        show_timestamp: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{debug, error, info, trace, warn};

    #[test]
    fn test_log_level_filter() {
        assert_eq!(LogLevel::Trace.as_filter_str(), "trace");
        assert_eq!(LogLevel::Debug.as_filter_str(), "debug");
        assert_eq!(LogLevel::Info.as_filter_str(), "info");
        assert_eq!(LogLevel::Warn.as_filter_str(), "warn");
        assert_eq!(LogLevel::Error.as_filter_str(), "error");
    }

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert!(matches!(config.level, LogLevel::Info));
        assert!(matches!(config.format, LogFormat::Pretty));
        assert!(config.show_location);
        assert!(!config.show_thread_id);
        assert!(config.show_timestamp);
    }

    #[test]
    fn test_env_configs() {
        let dev = dev_log_config();
        assert!(matches!(dev.level, LogLevel::Debug));
        assert!(matches!(dev.format, LogFormat::Pretty));

        let prod = prod_log_config();
        assert!(matches!(prod.level, LogLevel::Info));
        assert!(matches!(prod.format, LogFormat::Json));

        let test = test_log_config();
        assert!(matches!(test.level, LogLevel::Trace));
        assert!(matches!(test.format, LogFormat::Compact));
    }

    #[test]
    fn test_logging_macros() {
        // 注意：这个测试只验证宏是否可用，不验证输出
        // 实际的日志输出需要初始化后才能看到
        trace!("This is a trace message");
        debug!("This is a debug message");
        info!("This is an info message");
        warn!("This is a warn message");
        error!("This is an error message");

        // 结构化日志
        info!(user = "alice", amount = 1000, "User performed transaction");
    }
}
