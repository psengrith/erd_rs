use super::{ClassDiagramFormater, Vis};

pub struct MMDFormater();

impl ClassDiagramFormater for MMDFormater {
    fn format_class(&self, name: String) -> String {
        format!(" class {name} {{")
    }

    fn format_field(&self, vis: String, name: String, ty: String) -> String {
        format!("  {vis}{ty} {name}")
    }

    fn format_link(
        &self,
        clazz1: String,
        cardinality1: String,
        clazz2: String,
        cardinality2: String,
        label: String,
    ) -> String {
        format!(" {clazz1} \"{cardinality1}\" -- \"{cardinality2}\" {clazz2} : {label}")
    }

    fn format_class_end(&self) -> String {
        " }".to_string()
    }

    fn format_fn(
        &self,
        clazz: String,
        vis: String,
        method_name: String,
        inputs: Vec<(String, String)>,
        output: String,
    ) -> String {
        let inputs = inputs
            .into_iter()
            .map(|i| i.0)
            .collect::<Vec<String>>()
            .join(", ");
        format!(" {clazz}: {vis}{method_name}({inputs}) {output}")
    }

    fn format_vis(&self, vis: super::Vis) -> String {
        {
            match vis {
                Vis::Public => "+",
                Vis::Internal => "~",
                Vis::Protected => "#",
                Vis::Private => "-",
            }
            .to_string()
        }
    }
}
