//! build.rs が生成する埋め込みランタイム定義

include!(concat!(env!("OUT_DIR"), "/embedded_runtime.rs"));
