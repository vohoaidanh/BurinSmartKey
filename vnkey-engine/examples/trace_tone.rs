use vnkey_engine::engine::Engine;
use vnkey_engine::input::InputMethod;

fn type_word(engine: &mut Engine, keys: &str) -> String {
    engine.reset();
    let mut total_output = Vec::<u8>::new();
    for ch in keys.bytes() {
        let res = engine.process(ch as u32);
        if res.processed {
            for _ in 0..res.backspaces {
                let s = String::from_utf8_lossy(&total_output).to_string();
                let mut chars: Vec<char> = s.chars().collect();
                if !chars.is_empty() { chars.pop(); }
                total_output = chars.iter().collect::<String>().into_bytes();
            }
            total_output.extend_from_slice(&res.output);
        } else {
            total_output.push(ch);
        }
    }
    String::from_utf8_lossy(&total_output).to_string()
}

fn main() {
    let mut e = Engine::new();
    e.set_input_method(InputMethod::Telex);

    // Trace each keystroke for "muoons"
    println!("=== Tracing 'muoons' keystroke by keystroke ===");
    e.reset();
    let mut total_output = Vec::<u8>::new();
    for ch in "muoons".bytes() {
        let res = e.process(ch as u32);
        let at_begin = e.at_word_beginning();
        println!("  key='{}' processed={} backspaces={} output={:?} at_word_beginning={}",
            ch as char, res.processed, res.backspaces,
            String::from_utf8_lossy(&res.output).to_string(),
            at_begin);
        if res.processed {
            for _ in 0..res.backspaces {
                let s = String::from_utf8_lossy(&total_output).to_string();
                let mut chars: Vec<char> = s.chars().collect();
                if !chars.is_empty() { chars.pop(); }
                total_output = chars.iter().collect::<String>().into_bytes();
            }
            total_output.extend_from_slice(&res.output);
        } else {
            total_output.push(ch);
        }
        println!("  accumulated: '{}'", String::from_utf8_lossy(&total_output));
    }

    println!("\n=== Tracing 'muonf' keystroke by keystroke ===");
    e.reset();
    total_output.clear();
    for ch in "muonf".bytes() {
        let res = e.process(ch as u32);
        let at_begin = e.at_word_beginning();
        println!("  key='{}' processed={} backspaces={} output={:?} at_word_beginning={}",
            ch as char, res.processed, res.backspaces,
            String::from_utf8_lossy(&res.output).to_string(),
            at_begin);
        if res.processed {
            for _ in 0..res.backspaces {
                let s = String::from_utf8_lossy(&total_output).to_string();
                let mut chars: Vec<char> = s.chars().collect();
                if !chars.is_empty() { chars.pop(); }
                total_output = chars.iter().collect::<String>().into_bytes();
            }
            total_output.extend_from_slice(&res.output);
        } else {
            total_output.push(ch);
        }
        println!("  accumulated: '{}'", String::from_utf8_lossy(&total_output));
    }

    println!("\n=== Tracing 'fix' keystroke by keystroke ===");
    e.reset();
    total_output.clear();
    for ch in "fix".bytes() {
        let res = e.process(ch as u32);
        let at_begin = e.at_word_beginning();
        println!("  key='{}' processed={} backspaces={} output={:?} at_word_beginning={}",
            ch as char, res.processed, res.backspaces,
            String::from_utf8_lossy(&res.output).to_string(),
            at_begin);
        if res.processed {
            for _ in 0..res.backspaces {
                let s = String::from_utf8_lossy(&total_output).to_string();
                let mut chars: Vec<char> = s.chars().collect();
                if !chars.is_empty() { chars.pop(); }
                total_output = chars.iter().collect::<String>().into_bytes();
            }
            total_output.extend_from_slice(&res.output);
        } else {
            total_output.push(ch);
        }
        println!("  accumulated: '{}'", String::from_utf8_lossy(&total_output));
    }

    // Also trace the summary results
    println!("\n=== Summary ===");
    let cases = vec![
        "muonf", "muoons", "muoonf", "chaof", "toons",
        "banr", "tangf", "toanf", "bans", "muons",
        "fix", "linux", "flex",
    ];
    for input in cases {
        let out = type_word(&mut e, input);
        println!("'{}' -> '{}'", input, out);
    }
}
