use std::process::Command;

fn escape_single_quotes(input: &str) -> String {
    input.replace('\'', "''")
}

pub fn speak(text: &str) {
    println!("Nova: {}", text);

    if std::env::var("NOVA_DISABLE_TTS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "Add-Type -AssemblyName System.Speech; \
             $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
             $s.Rate = 0; $s.Speak('{}')",
            escape_single_quotes(text)
        );
        let _ = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(script)
            .status();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("say").arg(text).status();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("espeak").arg(text).status();
    }
}
