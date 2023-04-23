use std::collections::HashMap;

const R: &[u8] = &[36, 250, 199, 34, 9, 236, 102, 39];
const L: &[u8] = &[51, 148, 160, 224, 43, 59, 156, 105];

pub fn calculate_sig(method: &str, path: &str, params: &HashMap<&str, &str>) -> String {
    let mut param_list = Vec::new();
    for (k, v) in params.iter() {
        if k != &"sig" {
            param_list.push(format!("{v}={k}"));
        }
    }
    param_list.sort_unstable();
    let param_str = param_list.join(";");
    let base_string = format!("{}&{}&{};", method.to_uppercase(), path, param_str);
    let data: Vec<u8> = [R, base_string.as_bytes(), L].concat();
    format!("{:x}", md5::compute(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculate_sig() {
        let sig = calculate_sig("post", "/v1/game/farm/stall/query", &HashMap::from([]));
        dbg!(sig);
    }
}
