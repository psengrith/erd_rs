#[derive(Clone, Copy, Default)]
pub enum Vis {
    Public,
    Internal,
    Protected,
    #[default]
    Private,
}

impl From<Visibility> for Vis {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Restricted(res) => {
                if res.path.is_ident("self") {
                    Vis::Private
                } else {
                    Vis::Internal
                }
            }
            Visibility::Inherited => Vis::Private,
            _ => Vis::Public,
        }
    }
}

pub trait ClassDiagramFormater {
    fn format_class(&self, name: String) -> String;

    fn format_field(&self, vis: String, name: String, ty: String) -> String;

    fn format_link(
        &self,
        clazz1: String,
        cardinality1: String,
        clazz2: String,
        cardinality2: String,
        label: String,
    ) -> String;

    fn format_class_end(&self) -> String;

    fn format_fn(
        &self,
        clazz: String,
        vis: String,
        method_name: String,
        inputs: Vec<(String, String)>,
        output: String,
    ) -> String;

    fn format_vis(&self, vis: Vis) -> String;
}

pub mod mmd_formater;
pub use mmd_formater::MMDFormater;
use syn::Visibility;
