use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub labels: Labels,
    pub document: DocumentConfig,
    pub references: ReferenceConfig,
    pub diagnostics: DiagnosticConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Labels {
    pub endpoint_prefix: String,
    pub path_prefix: String,
    pub logic: String,
    pub components: String,
    pub mappings: String,
    pub references: String,
    pub description: String,
    pub source: String,
    pub target: String,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            endpoint_prefix: "Эндпоинт:".into(),
            path_prefix: "Path:".into(),
            logic: "Логика работы".into(),
            components: "Компоненты".into(),
            mappings: "Маппинги".into(),
            references: "Ссылки".into(),
            description: "Описание".into(),
            source: "Source".into(),
            target: "Target".into(),
        }
    }
}

impl Labels {
    pub fn sections(&self) -> [&str; 4] {
        [
            &self.logic,
            &self.components,
            &self.mappings,
            &self.references,
        ]
    }
    pub fn is_valid(&self) -> bool {
        let sections = self.sections();
        let labels = [
            self.endpoint_prefix.as_str(),
            self.path_prefix.as_str(),
            self.description.as_str(),
            self.source.as_str(),
            self.target.as_str(),
        ];
        sections.iter().chain(labels.iter()).all(|label| {
            !label.is_empty() && label.trim() == *label && !label.contains(['\n', '\r'])
        }) && sections.iter().enumerate().all(|(index, label)| {
            !sections
                .iter()
                .take(index)
                .any(|previous| previous == label)
        }) && self.source != self.target
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DocumentConfig {
    pub require_endpoint: bool,
    pub require_logic: bool,
    pub require_components: bool,
    pub require_mappings: bool,
    pub enforce_section_order: bool,
    pub allow_unknown_sections: bool,
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self {
            require_endpoint: true,
            require_logic: true,
            require_components: false,
            require_mappings: false,
            enforce_section_order: true,
            allow_unknown_sections: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReferenceConfig {
    pub require_reference_style: bool,
    pub definitions_at_end: bool,
    pub validate_local_paths: bool,
    pub validate_anchors: bool,
}

impl Default for ReferenceConfig {
    fn default() -> Self {
        Self {
            require_reference_style: true,
            definitions_at_end: true,
            validate_local_paths: true,
            validate_anchors: true,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiagnosticConfig {
    pub warnings_as_errors: bool,
}
