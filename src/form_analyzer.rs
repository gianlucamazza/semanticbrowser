//! Form Analyzer Module
//!
//! Analyzes HTML forms to discover fields, their types, and semantic meaning.
//! Enables intelligent form filling without hardcoded selectors.

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// Type of form field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Tel,
    Number,
    Url,
    Search,
    Date,
    Checkbox,
    Radio,
    Select,
    Textarea,
    File,
    Hidden,
    Unknown,
}

impl FieldType {
    /// Parse from HTML input type attribute
    pub fn from_input_type(type_attr: &str) -> Self {
        match type_attr.to_lowercase().as_str() {
            "text" => Self::Text,
            "email" => Self::Email,
            "password" => Self::Password,
            "tel" | "phone" => Self::Tel,
            "number" => Self::Number,
            "url" => Self::Url,
            "search" => Self::Search,
            "date" | "datetime-local" | "month" | "week" | "time" => Self::Date,
            "checkbox" => Self::Checkbox,
            "radio" => Self::Radio,
            "file" => Self::File,
            "hidden" => Self::Hidden,
            _ => Self::Unknown,
        }
    }
}

/// Description of a form field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDescription {
    /// CSS selector to locate the field
    pub selector: String,
    /// Semantic type of the field
    pub field_type: FieldType,
    /// Label text associated with the field
    pub label: Option<String>,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Name attribute
    pub name: Option<String>,
    /// ID attribute
    pub id: Option<String>,
    /// Whether the field is required
    pub required: bool,
    /// Confidence score (0.0-1.0) for semantic matching
    pub confidence: f32,
    /// Alternative selectors
    pub alternatives: Vec<String>,
}

impl FieldDescription {
    /// Calculate semantic similarity score with a hint
    pub fn similarity_score(&self, hint: &str) -> f32 {
        let hint_lower = hint.to_lowercase();
        let mut score: f32 = 0.0;
        let mut matches: f32 = 0.0;

        // Check ID
        if let Some(ref id) = self.id {
            if id.to_lowercase().contains(&hint_lower) {
                score += 0.4;
                matches += 1.0;
            }
        }

        // Check name
        if let Some(ref name) = self.name {
            if name.to_lowercase().contains(&hint_lower) {
                score += 0.3;
                matches += 1.0;
            }
        }

        // Check label
        if let Some(ref label) = self.label {
            if label.to_lowercase().contains(&hint_lower) {
                score += 0.2;
                matches += 1.0;
            }
        }

        // Check placeholder
        if let Some(ref placeholder) = self.placeholder {
            if placeholder.to_lowercase().contains(&hint_lower) {
                score += 0.1;
                matches += 1.0;
            }
        }

        // Normalize by number of checks
        if matches > 0.0 {
            score
        } else {
            0.0
        }
    }
}

/// Form purpose classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormPurpose {
    Login,
    Registration,
    Search,
    Contact,
    Payment,
    Profile,
    Settings,
    Comment,
    Unknown,
}

/// Complete form description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormDescription {
    /// CSS selector for the form element
    pub selector: String,
    /// Detected purpose of the form
    pub purpose: FormPurpose,
    /// All fields in the form
    pub fields: Vec<FieldDescription>,
    /// Submit button selector
    pub submit_button: Option<String>,
    /// Form action URL
    pub action: Option<String>,
    /// Form method (GET/POST)
    pub method: String,
}

impl FormDescription {
    /// Find field by semantic hint
    pub fn find_field(&self, hint: &str) -> Option<&FieldDescription> {
        self.fields
            .iter()
            .max_by(|a, b| {
                a.similarity_score(hint)
                    .partial_cmp(&b.similarity_score(hint))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .filter(|field| field.similarity_score(hint) > 0.3)
    }

    /// Get all fields matching a type
    pub fn fields_by_type(&self, field_type: FieldType) -> Vec<&FieldDescription> {
        self.fields.iter().filter(|f| f.field_type == field_type).collect()
    }
}

/// Form analyzer
pub struct FormAnalyzer;

impl FormAnalyzer {
    /// Analyze HTML to discover forms
    pub fn analyze_html(html: &str) -> Vec<FormDescription> {
        let document = Html::parse_document(html);
        let form_selector = Selector::parse("form").unwrap();

        document
            .select(&form_selector)
            .enumerate()
            .map(|(idx, form)| Self::analyze_form_element(&document, form, idx))
            .collect()
    }

    /// Analyze a single form element
    fn analyze_form_element(
        document: &Html,
        form: scraper::ElementRef,
        index: usize,
    ) -> FormDescription {
        let selector = format!("form:nth-of-type({})", index + 1);
        let action = form.value().attr("action").map(String::from);
        let method = form.value().attr("method").unwrap_or("get").to_uppercase();

        // Discover fields
        let fields = Self::discover_fields(document, form);

        // Detect purpose
        let purpose = Self::detect_purpose(&fields, &action);

        // Find submit button
        let submit_button = Self::find_submit_button(form);

        FormDescription { selector, purpose, fields, submit_button, action, method }
    }

    /// Discover all form fields
    fn discover_fields(document: &Html, form: scraper::ElementRef) -> Vec<FieldDescription> {
        let mut fields = Vec::new();

        // Input fields
        let input_selector = Selector::parse("input").unwrap();
        for (idx, input) in form.select(&input_selector).enumerate() {
            if let Some(field) = Self::analyze_input(document, input, idx) {
                fields.push(field);
            }
        }

        // Textarea fields
        let textarea_selector = Selector::parse("textarea").unwrap();
        for (idx, textarea) in form.select(&textarea_selector).enumerate() {
            if let Some(field) = Self::analyze_textarea(document, textarea, idx) {
                fields.push(field);
            }
        }

        // Select fields
        let select_selector = Selector::parse("select").unwrap();
        for (idx, select) in form.select(&select_selector).enumerate() {
            if let Some(field) = Self::analyze_select(document, select, idx) {
                fields.push(field);
            }
        }

        fields
    }

    /// Analyze input element
    fn analyze_input(
        document: &Html,
        input: scraper::ElementRef,
        index: usize,
    ) -> Option<FieldDescription> {
        let input_type = input.value().attr("type").unwrap_or("text").to_string();

        // Skip submit/button inputs
        if matches!(input_type.as_str(), "submit" | "button" | "reset" | "image") {
            return None;
        }

        let field_type = FieldType::from_input_type(&input_type);
        let name = input.value().attr("name").map(String::from);
        let id = input.value().attr("id").map(String::from);
        let placeholder = input.value().attr("placeholder").map(String::from);
        let required = input.value().attr("required").is_some();

        // Build selector
        let selector = if let Some(ref id_val) = id {
            format!("#{}", id_val)
        } else if let Some(ref name_val) = name {
            format!("input[name='{}']", name_val)
        } else {
            format!("input:nth-of-type({})", index + 1)
        };

        // Find associated label
        let label = Self::find_label(document, &id, &name);

        // Calculate confidence
        let confidence = Self::calculate_confidence(&id, &name, &label, &placeholder);

        // Build alternatives
        let mut alternatives = Vec::new();
        if let Some(ref id_val) = id {
            alternatives.push(format!("#{}", id_val));
        }
        if let Some(ref name_val) = name {
            alternatives.push(format!("input[name='{}']", name_val));
            alternatives.push(format!("[name='{}']", name_val));
        }

        Some(FieldDescription {
            selector,
            field_type,
            label,
            placeholder,
            name,
            id,
            required,
            confidence,
            alternatives,
        })
    }

    /// Analyze textarea element
    fn analyze_textarea(
        document: &Html,
        textarea: scraper::ElementRef,
        index: usize,
    ) -> Option<FieldDescription> {
        let name = textarea.value().attr("name").map(String::from);
        let id = textarea.value().attr("id").map(String::from);
        let placeholder = textarea.value().attr("placeholder").map(String::from);
        let required = textarea.value().attr("required").is_some();

        let selector = if let Some(ref id_val) = id {
            format!("#{}", id_val)
        } else if let Some(ref name_val) = name {
            format!("textarea[name='{}']", name_val)
        } else {
            format!("textarea:nth-of-type({})", index + 1)
        };

        let label = Self::find_label(document, &id, &name);
        let confidence = Self::calculate_confidence(&id, &name, &label, &placeholder);

        let mut alternatives = Vec::new();
        if let Some(ref id_val) = id {
            alternatives.push(format!("#{}", id_val));
        }
        if let Some(ref name_val) = name {
            alternatives.push(format!("textarea[name='{}']", name_val));
        }

        Some(FieldDescription {
            selector,
            field_type: FieldType::Textarea,
            label,
            placeholder,
            name,
            id,
            required,
            confidence,
            alternatives,
        })
    }

    /// Analyze select element
    fn analyze_select(
        document: &Html,
        select: scraper::ElementRef,
        index: usize,
    ) -> Option<FieldDescription> {
        let name = select.value().attr("name").map(String::from);
        let id = select.value().attr("id").map(String::from);
        let required = select.value().attr("required").is_some();

        let selector = if let Some(ref id_val) = id {
            format!("#{}", id_val)
        } else if let Some(ref name_val) = name {
            format!("select[name='{}']", name_val)
        } else {
            format!("select:nth-of-type({})", index + 1)
        };

        let label = Self::find_label(document, &id, &name);
        let confidence = Self::calculate_confidence(&id, &name, &label, &None);

        let mut alternatives = Vec::new();
        if let Some(ref id_val) = id {
            alternatives.push(format!("#{}", id_val));
        }
        if let Some(ref name_val) = name {
            alternatives.push(format!("select[name='{}']", name_val));
        }

        Some(FieldDescription {
            selector,
            field_type: FieldType::Select,
            label,
            placeholder: None,
            name,
            id,
            required,
            confidence,
            alternatives,
        })
    }

    /// Find label for a field
    fn find_label(document: &Html, id: &Option<String>, name: &Option<String>) -> Option<String> {
        // Try label[for="id"]
        if let Some(id_val) = id {
            let label_selector = Selector::parse(&format!("label[for='{}']", id_val)).ok()?;
            if let Some(label) = document.select(&label_selector).next() {
                return Some(label.text().collect::<String>().trim().to_string());
            }
        }

        // Try label containing input
        if let Some(name_val) = name {
            let label_selector = Selector::parse("label").ok()?;
            for label in document.select(&label_selector) {
                let input_selector =
                    Selector::parse(&format!("input[name='{}']", name_val)).ok()?;
                if label.select(&input_selector).next().is_some() {
                    return Some(label.text().collect::<String>().trim().to_string());
                }
            }
        }

        None
    }

    /// Calculate confidence score for field detection
    fn calculate_confidence(
        id: &Option<String>,
        name: &Option<String>,
        label: &Option<String>,
        placeholder: &Option<String>,
    ) -> f32 {
        let mut score: f32 = 0.5; // Base score

        if id.is_some() {
            score += 0.2;
        }
        if name.is_some() {
            score += 0.15;
        }
        if label.is_some() {
            score += 0.1;
        }
        if placeholder.is_some() {
            score += 0.05;
        }

        score.min(1.0)
    }

    /// Detect form purpose from fields and action
    fn detect_purpose(fields: &[FieldDescription], action: &Option<String>) -> FormPurpose {
        // Check action URL for keywords
        if let Some(ref action_url) = action {
            let action_lower = action_url.to_lowercase();
            if action_lower.contains("login") || action_lower.contains("signin") {
                return FormPurpose::Login;
            }
            if action_lower.contains("register") || action_lower.contains("signup") {
                return FormPurpose::Registration;
            }
            if action_lower.contains("search") {
                return FormPurpose::Search;
            }
            if action_lower.contains("contact") {
                return FormPurpose::Contact;
            }
            if action_lower.contains("payment") || action_lower.contains("checkout") {
                return FormPurpose::Payment;
            }
        }

        // Analyze field names and labels
        let has_password = fields.iter().any(|f| f.field_type == FieldType::Password);
        let has_email = fields.iter().any(|f| f.field_type == FieldType::Email);
        let text_fields = fields.iter().filter(|f| f.field_type == FieldType::Text).count();

        // Login: password + (email or username)
        if has_password && (has_email || text_fields == 1) && fields.len() <= 3 {
            return FormPurpose::Login;
        }

        // Registration: password + email + multiple fields
        if has_password && has_email && fields.len() > 3 {
            return FormPurpose::Registration;
        }

        // Search: single text field
        if fields.len() == 1 && fields[0].field_type == FieldType::Text {
            return FormPurpose::Search;
        }

        FormPurpose::Unknown
    }

    /// Find submit button in form
    fn find_submit_button(form: scraper::ElementRef) -> Option<String> {
        // Try input[type="submit"]
        let submit_selector = Selector::parse("input[type='submit']").ok()?;
        if let Some(submit) = form.select(&submit_selector).next() {
            if let Some(id) = submit.value().attr("id") {
                return Some(format!("#{}", id));
            }
            return Some("input[type='submit']".to_string());
        }

        // Try button[type="submit"]
        let button_selector = Selector::parse("button[type='submit']").ok()?;
        if let Some(button) = form.select(&button_selector).next() {
            if let Some(id) = button.value().attr("id") {
                return Some(format!("#{}", id));
            }
            return Some("button[type='submit']".to_string());
        }

        // Try any button
        let any_button_selector = Selector::parse("button").ok()?;
        if let Some(button) = form.select(&any_button_selector).next() {
            if let Some(id) = button.value().attr("id") {
                return Some(format!("#{}", id));
            }
            return Some("button".to_string());
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type_from_input() {
        assert_eq!(FieldType::from_input_type("email"), FieldType::Email);
        assert_eq!(FieldType::from_input_type("password"), FieldType::Password);
        assert_eq!(FieldType::from_input_type("text"), FieldType::Text);
    }

    #[test]
    fn test_analyze_login_form() {
        let html = r#"
            <form action="/login" method="post">
                <label for="username">Username</label>
                <input type="text" id="username" name="username" required>
                
                <label for="password">Password</label>
                <input type="password" id="password" name="password" required>
                
                <button type="submit">Login</button>
            </form>
        "#;

        let forms = FormAnalyzer::analyze_html(html);
        assert_eq!(forms.len(), 1);

        let form = &forms[0];
        assert_eq!(form.purpose, FormPurpose::Login);
        assert_eq!(form.fields.len(), 2);
        assert!(form.submit_button.is_some());

        // Find username field
        let username_field = form.find_field("username");
        assert!(username_field.is_some(), "Username field should be found");
        let field = username_field.unwrap();
        let score = field.similarity_score("username");
        assert!(score > 0.3, "Similarity score should be > 0.3, got {}", score);
    }

    #[test]
    fn test_field_similarity_score() {
        let field = FieldDescription {
            selector: "#email".to_string(),
            field_type: FieldType::Email,
            label: Some("Email Address".to_string()),
            placeholder: Some("Enter your email".to_string()),
            name: Some("email".to_string()),
            id: Some("email".to_string()),
            required: true,
            confidence: 1.0,
            alternatives: vec![],
        };

        let score = field.similarity_score("email");
        assert!(score > 0.3, "Score for 'email' should be > 0.3, got {}", score);

        let username_score = field.similarity_score("username");
        assert!(
            username_score < 0.3,
            "Score for 'username' should be < 0.3, got {}",
            username_score
        );
    }
}
