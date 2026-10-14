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
    pub examples: String,
    pub references: String,
    pub description: String,
    pub source: String,
    pub target: String,
    pub request: String,
    pub response: String,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            endpoint_prefix: "Эндпоинт:".into(),
            path_prefix: "Path:".into(),
            logic: "Логика работы".into(),
            components: "Компоненты".into(),
            mappings: "Маппинги".into(),
            examples: "Примеры".into(),
            references: "Ссылки".into(),
            description: "Описание".into(),
            source: "Source".into(),
            target: "Target".into(),
            request: "Запрос".into(),
            response: "Ответ".into(),
        }
    }
}

impl Labels {
    pub fn sections(&self) -> [&str; 5] {
        self.named_sections().map(|(_, label)| label)
    }
    pub fn validation_errors(&self) -> Vec<String> {
        let section_labels = self.named_sections();
        let other_labels = [
            ("endpoint_prefix", self.endpoint_prefix.as_str()),
            ("path_prefix", self.path_prefix.as_str()),
            ("description", self.description.as_str()),
            ("source", self.source.as_str()),
            ("target", self.target.as_str()),
            ("request", self.request.as_str()),
            ("response", self.response.as_str()),
        ];
        let mut errors: Vec<String> = section_labels
            .iter()
            .chain(other_labels.iter())
            .filter_map(|(key, value)| shape_error(key, value))
            .collect();
        for (index, (key, value)) in section_labels.iter().enumerate() {
            let duplicates = section_labels
                .iter()
                .skip(index + 1)
                .filter(|(_, other)| other == value)
                .map(|(other, _)| format!("labels.{key} and labels.{other} must differ"));
            errors.extend(duplicates);
        }
        if self.source == self.target {
            errors.push("labels.source and labels.target must differ".into());
        }
        if self.request == self.response {
            errors.push("labels.request and labels.response must differ".into());
        }
        errors
    }
    const fn named_sections(&self) -> [(&'static str, &str); 5] {
        [
            ("logic", self.logic.as_str()),
            ("components", self.components.as_str()),
            ("mappings", self.mappings.as_str()),
            ("examples", self.examples.as_str()),
            ("references", self.references.as_str()),
        ]
    }
}

fn shape_error(key: &str, value: &str) -> Option<String> {
    let reason = if value.is_empty() {
        "must not be empty"
    } else if value.trim() != value {
        "must not have surrounding whitespace"
    } else if value.contains(['\n', '\r']) {
        "must not span multiple lines"
    } else {
        return None;
    };
    Some(format!("labels.{key} {reason}"))
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DocumentConfig {
    pub require_endpoint: bool,
    pub require_logic: bool,
    pub require_components: bool,
    pub require_mappings: bool,
    pub require_examples: bool,
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
            require_examples: false,
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
    pub toc_allowed: bool,
}

impl Default for ReferenceConfig {
    fn default() -> Self {
        Self {
            require_reference_style: true,
            definitions_at_end: true,
            validate_local_paths: true,
            validate_anchors: true,
            toc_allowed: false,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiagnosticConfig {
    pub warnings_as_errors: bool,
}
