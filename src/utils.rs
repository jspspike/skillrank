use std::fmt::Write;

use cfg_if::cfg_if;
use tinytemplate::error::Result;

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn format_float(val: &serde_json::Value, output: &mut String) -> Result<()> {
    if let serde_json::Value::Number(num) = val {
        if let Some(num) = num.as_f64() {
            write!(output, "{:.2}", num)?;
            return Ok(());
        }
    }

    tinytemplate::format(val, output)?;
    Ok(())
}
