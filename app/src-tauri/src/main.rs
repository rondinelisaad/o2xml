#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

#[cfg(all(target_os = "macos", not(debug_assertions)))]
const SAFE_RENDERING_MARKER: &str = "--o2xml-safe-rendering";

#[cfg(all(target_os = "macos", not(debug_assertions)))]
const WEBKIT_SAFE_RENDERING_ARGUMENTS: [&str; 8] = [
    "-WebKitDebugUseGPUProcessForDOMRenderingEnabled",
    "NO",
    "-WebKitDebugUseGPUProcessForCanvasRenderingEnabled",
    "NO",
    "-WebKitDebugUseGPUProcessForMediaEnabled",
    "NO",
    "-WebKitDebugUseGPUProcessForWebGLEnabled",
    "NO",
];

#[cfg(all(target_os = "macos", not(debug_assertions)))]
fn enable_safe_webkit_rendering() {
    use std::os::unix::process::CommandExt;

    let arguments = std::env::args_os().collect::<Vec<_>>();
    if arguments
        .iter()
        .any(|argument| argument == SAFE_RENDERING_MARKER)
    {
        return;
    }

    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => {
            eprintln!("O2XML could not enable safe WebKit rendering: {error}");
            return;
        }
    };

    let error = std::process::Command::new(executable)
        .args(arguments.iter().skip(1))
        .arg(SAFE_RENDERING_MARKER)
        .args(WEBKIT_SAFE_RENDERING_ARGUMENTS)
        .exec();
    eprintln!("O2XML could not restart with safe WebKit rendering: {error}");
}

fn main() {
    #[cfg(all(target_os = "macos", not(debug_assertions)))]
    enable_safe_webkit_rendering();

    o2xml_desktop_lib::run();
}

#[cfg(all(test, target_os = "macos", not(debug_assertions)))]
mod tests {
    use super::*;

    #[test]
    fn safe_rendering_disables_gpu_backed_webkit_features() {
        assert_eq!(WEBKIT_SAFE_RENDERING_ARGUMENTS.len(), 8);
        for feature in ["DOMRendering", "CanvasRendering", "Media", "WebGL"] {
            assert!(WEBKIT_SAFE_RENDERING_ARGUMENTS
                .iter()
                .any(|argument| argument.contains(feature)));
        }
        assert_eq!(
            WEBKIT_SAFE_RENDERING_ARGUMENTS
                .iter()
                .filter(|argument| **argument == "NO")
                .count(),
            4
        );
    }
}
