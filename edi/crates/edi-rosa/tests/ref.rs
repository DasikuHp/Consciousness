// El port en Rust debe dar exactamente las mismas predicciones que rosa() de RWKV-8.
#[test]
fn igual_que_rwkv8() {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!("ref.json")).unwrap();
    for c in &cases {
        let x: Vec<u32> = c["x"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect();
        let y: Vec<i64> = c["y"].as_array().unwrap().iter().map(|v| v.as_i64().unwrap()).collect();
        let mut r = edi_rosa::Rosa::new(0);
        let got: Vec<i64> = x.iter().map(|&t| r.push(t).map(|p| p.token as i64).unwrap_or(-1)).collect();
        assert_eq!(got, y);
        let r2 = edi_rosa::Rosa::from_hist(x.clone(), 0);
        assert_eq!(r2.states(), r.states());
    }
}
