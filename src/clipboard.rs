use std::{mem::size_of, ptr::copy_nonoverlapping};

use url::Url;
use windows::{
    Win32::{
        Foundation::{HANDLE, HGLOBAL, HWND},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
                OpenClipboard, SetClipboardData,
            },
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
            Ole::CF_UNICODETEXT,
        },
    },
    core::{Error, Result},
};

use crate::config::RewriteTarget;

pub fn rewrite_url(input: &str, target: RewriteTarget) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) {
        return None;
    }

    let mut url = Url::parse(trimmed).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }

    if url.host_str()? != "x.com" {
        return None;
    }

    let segments: Vec<_> = url.path_segments()?.collect();
    if segments.len() < 3
        || segments[0].is_empty()
        || segments[1] != "status"
        || segments[2].is_empty()
    {
        return None;
    }

    url.set_host(Some(target.host())).ok()?;
    Some(url.into())
}

pub fn read_clipboard_text(owner: HWND) -> Result<Option<String>> {
    unsafe {
        OpenClipboard(Some(owner))?;
        let _guard = ClipboardGuard;

        if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_err() {
            return Ok(None);
        }

        let handle = GetClipboardData(CF_UNICODETEXT.0 as u32)?;
        let ptr = GlobalLock(HGLOBAL(handle.0));
        if ptr.is_null() {
            return Err(Error::from_thread());
        }

        let text = read_wide_string(ptr.cast::<u16>());
        let _ = GlobalUnlock(HGLOBAL(handle.0));
        Ok(Some(text))
    }
}

pub fn write_clipboard_text(owner: HWND, text: &str) -> Result<()> {
    unsafe {
        let encoded: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        OpenClipboard(Some(owner))?;
        let _guard = ClipboardGuard;
        EmptyClipboard()?;

        let bytes = encoded.len() * size_of::<u16>();
        let handle = GlobalAlloc(GMEM_MOVEABLE, bytes)?;

        let ptr = GlobalLock(handle);
        if ptr.is_null() {
            return Err(Error::from_thread());
        }

        copy_nonoverlapping(encoded.as_ptr(), ptr.cast::<u16>(), encoded.len());
        let _ = GlobalUnlock(handle);
        SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(handle.0)))?;

        Ok(())
    }
}

struct ClipboardGuard;

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

unsafe fn read_wide_string(mut ptr: *const u16) -> String {
    let start = ptr;
    let mut len = 0;
    while *ptr != 0 {
        len += 1;
        ptr = ptr.add(1);
    }

    let slice = std::slice::from_raw_parts(start, len);
    String::from_utf16_lossy(slice)
}

#[cfg(test)]
mod tests {
    use crate::config::RewriteTarget;

    use super::rewrite_url;

    #[test]
    fn rewrites_to_fx_domain() {
        let output = rewrite_url("https://x.com/example/status/123", RewriteTarget::Fx).unwrap();
        assert_eq!(output, "https://fxtwitter.com/example/status/123");
    }

    #[test]
    fn rewrites_to_vx_domain() {
        let output = rewrite_url("http://x.com/example/status/123", RewriteTarget::Vx).unwrap();
        assert_eq!(output, "http://vxtwitter.com/example/status/123");
    }

    #[test]
    fn preserves_query_and_fragment() {
        let output = rewrite_url(
            "https://x.com/example/status/123?ref=abc#section",
            RewriteTarget::Fx,
        )
        .unwrap();
        assert_eq!(
            output,
            "https://fxtwitter.com/example/status/123?ref=abc#section"
        );
    }

    #[test]
    fn ignores_non_status_urls() {
        assert!(rewrite_url("https://x.com/example", RewriteTarget::Fx).is_none());
    }

    #[test]
    fn ignores_non_urls() {
        assert!(rewrite_url("not a url", RewriteTarget::Fx).is_none());
    }

    #[test]
    fn ignores_text_containing_url() {
        assert!(rewrite_url("see https://x.com/example/status/123", RewriteTarget::Fx).is_none());
    }

    #[test]
    fn ignores_already_rewritten_urls() {
        assert!(
            rewrite_url(
                "https://fxtwitter.com/example/status/123",
                RewriteTarget::Fx
            )
            .is_none()
        );
        assert!(
            rewrite_url(
                "https://vxtwitter.com/example/status/123",
                RewriteTarget::Vx
            )
            .is_none()
        );
    }
}
