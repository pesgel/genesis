pub mod bundle;
mod compile;

#[derive(Debug, Clone, Default)]
pub enum LoaderTypeEnum {
    #[default]
    FS,
    DB,
}
