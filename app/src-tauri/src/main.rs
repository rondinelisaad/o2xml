#[cfg(target_os = "macos")]
const SAFE_RENDERING_MARKER: &str = "--o2xml-safe-rendering";

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
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
    #[cfg(target_os = "macos")]
    enable_safe_webkit_rendering();

    o2xml_desktop_lib::run();
}

#[cfg(all(test, target_os = "macos"))]
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
