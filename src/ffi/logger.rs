use env_logger::Builder;
use ffi_support::FfiStr;
use log::{LevelFilter, Metadata, Record};
use std::{
    env,
    ffi::CString,
    io::Write,
    ptr,
    os::raw::{c_char, c_void}
};

pub type LogContext = *const c_void;

pub type EnabledCB = extern "C" fn(context: *const c_void, level: u32, target: FfiStr<'_>) -> bool;

pub type LogCB = extern "C" fn(context: LogContext,
                               level: u32,
                               target: FfiStr<'_>,
                               message: FfiStr<'_>,
                               module_path: FfiStr<'_>,
                               file: FfiStr<'_>,
                               line: u32);

pub type FlushCB = extern "C" fn(context: *const c_void);

pub struct AriesCredXFrameworkLogger {
    context: LogContext,
    enabled: Option<EnabledCB>,
    log: LogCB,
    flush: Option<FlushCB>,
}

impl AriesCredXFrameworkLogger {
    fn new(
        context: LogContext,
        enabled: Option<EnabledCB>,
        log: LogCB,
        flush: Option<FlushCB>,
    ) -> Self {
        Self {
            context,
            enabled,
            log,
            flush,
        }
    }
}

impl log::Log for AriesCredXFrameworkLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if let Some(enabled_cb) = self.enabled {
            let level = metadata.level() as u32;
            let target = CString::new(metadata.target()).unwrap();

            enabled_cb(self.context, level, target.as_ptr())
        } else {
            true
        }
    }

    fn log(&self, record: &Record) {
        let log_cb = self.log;

        let level = record.level() as u32;
        let target = CString::new(record.target()).unwrap();
        let message = CString::new(record.args().to_string()).unwrap();

        let module_path = record.module_path().map(|a| CString::new(a).unwrap());
        let file = record.file().map(|a| CString::new(a).unwrap());
        let line = record.line().unwrap_or(0);

        log_cb(
            self.context,
            level,
            target.as_ptr(),
            message.as_ptr(),
            module_path
                .as_ref()
                .map(|p| p.as_ptr())
                .unwrap_or(ptr::null()),
            file.as_ref().map(|p| p.as_ptr()).unwrap_or(ptr::null()),
            line,
        )
    }

    fn flush(&self) {
        if let Some(flush) = self.flush {
            flush(self.context)
        }
    }
}

unsafe impl Sync for AriesCredXFrameworkLogger {}

unsafe impl Send for AriesCredXFrameworkLogger {}

impl AriesCredXFrameworkLogger {
    pub fn init(
        context: LogContext,
        enabled: Option<EnabledCB>,
        log: LogCB,
        flush: Option<FlushCB>,
    ) -> Result<(), String> {
        let logger = AriesCredXFrameworkLogger::new(context, enabled, log, flush);

        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(LevelFilter::Trace);

        Ok(())
    }
}

pub struct AriesCredXFrameworkDefaultLogger;

impl AriesCredXFrameworkDefaultLogger {
    pub fn init(pattern: Option<String>) -> Result<(), String> {
        let pattern = pattern.or_else(|| env::var("RUST_LOG").ok());

        Builder::new()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{:>5}|{:<30}|{:>35}:{:<4}| {}",
                    record.level(),
                    record.target(),
                    record.file().get_or_insert(""),
                    record.line().get_or_insert(0),
                    record.args()
                )
            })
            .filter(None, LevelFilter::Off)
            .parse_filters(pattern.as_ref().map(String::as_str).unwrap_or(""))
            .try_init()?;

        Ok(())
    }
}