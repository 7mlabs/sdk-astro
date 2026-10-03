fn main() {
    let action = std::env::args().nth(1).unwrap_or_default();
    if action == "--compress-json" || action == "--expand-context-json" {
        use std::io::Read;
        let mut input = String::new();
        std::io::stdin().take((astro_core::MAX_CONTEXT_BYTES + 1) as u64)
            .read_to_string(&mut input).expect("Read UTF-8 JSON from stdin");
        let output = if action == "--compress-json" { astro_core::compress_json(&input) }
            else { astro_core::expand_context_json(&input) };
        println!("{output}");
        return;
    }
    let input = std::env::args().nth(1).unwrap_or_else(||
        r#"{"operation":"natal","utc":{"year":2000,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":10.8231,"longitude":106.6297}}"#.into());
    println!("{}", astro_core::calculate_json(&input));
}
