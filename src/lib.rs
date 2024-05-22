macro_rules! im_lazy {
    () => {
        || anyhow::anyhow!("file: {}, line: {}", file!(), line!())
    };
}

pub mod primitives;
pub mod rendering_engine;
