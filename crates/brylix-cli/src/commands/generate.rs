//! `brylix generate` command implementations.

use console::{style, Emoji};
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::Path;

static CHECK: Emoji<'_, '_> = Emoji("✅ ", "");
static WARN: Emoji<'_, '_> = Emoji("⚠️  ", "");
static FILE: Emoji<'_, '_> = Emoji("📄 ", "");

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert to plural (simple version)
fn to_plural(s: &str) -> String {
    if s.ends_with('s') || s.ends_with('x') || s.ends_with("ch") || s.ends_with("sh") {
        format!("{}es", s)
    } else if s.ends_with('y') && !s.ends_with("ay") && !s.ends_with("ey") && !s.ends_with("oy") && !s.ends_with("uy") {
        format!("{}ies", &s[..s.len()-1])
    } else {
        format!("{}s", s)
    }
}

/// Get template content (embedded in binary)
fn get_template(name: &str) -> Option<&'static str> {
    match name {
        "entity" => Some(include_str!("../../templates/entity.rs.hbs")),
        "service" => Some(include_str!("../../templates/service.rs.hbs")),
        "repository" => Some(include_str!("../../templates/repository.rs.hbs")),
        "resolver" => Some(include_str!("../../templates/resolver.rs.hbs")),
        "migration" => Some(include_str!("../../templates/migration.rs.hbs")),
        _ => None,
    }
}

/// Generate a SeaORM entity.
pub fn entity(name: &str) {
    println!(
        "{} Generating entity: {}",
        CHECK,
        style(name).cyan().bold()
    );

    let snake_name = to_snake_case(name);
    let table_name = to_plural(&snake_name);

    let template = match get_template("entity") {
        Some(t) => t,
        None => {
            eprintln!("{} Entity template not found", WARN);
            return;
        }
    };

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("entity", template).unwrap();

    let data = json!({
        "entity_name": name,
        "entity_name_snake": snake_name,
        "table_name": table_name,
        "fields": []
    });

    let output = handlebars.render("entity", &data).unwrap();

    let path = format!("src/model/{}.rs", snake_name);
    if Path::new(&path).exists() {
        eprintln!("{} File already exists: {}", WARN, path);
        return;
    }

    fs::write(&path, output).expect("Failed to write file");
    println!("  {} Created {}", FILE, style(&path).green());

    println!();
    println!("  Don't forget to add to src/model/mod.rs:");
    println!("    pub mod {};", snake_name);
}

/// Generate a service.
pub fn service(name: &str) {
    println!(
        "{} Generating service: {}",
        CHECK,
        style(name).cyan().bold()
    );

    let snake_name = to_snake_case(name);
    let plural_name = to_plural(&snake_name);

    let template = match get_template("service") {
        Some(t) => t,
        None => {
            eprintln!("{} Service template not found", WARN);
            return;
        }
    };

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("service", template).unwrap();

    let data = json!({
        "entity_name": name,
        "entity_name_snake": snake_name,
        "entity_name_plural": plural_name,
    });

    let output = handlebars.render("service", &data).unwrap();

    let path = format!("src/service/{}_service.rs", snake_name);
    if Path::new(&path).exists() {
        eprintln!("{} File already exists: {}", WARN, path);
        return;
    }

    fs::write(&path, output).expect("Failed to write file");
    println!("  {} Created {}", FILE, style(&path).green());

    println!();
    println!("  Don't forget to add to src/service/mod.rs:");
    println!("    mod {}_service;", snake_name);
    println!("    pub use {}_service::{}Service;", snake_name, name);
}

/// Generate a repository.
pub fn repository(name: &str) {
    println!(
        "{} Generating repository: {}",
        CHECK,
        style(name).cyan().bold()
    );

    let snake_name = to_snake_case(name);
    let plural_name = to_plural(&snake_name);

    let template = match get_template("repository") {
        Some(t) => t,
        None => {
            eprintln!("{} Repository template not found", WARN);
            return;
        }
    };

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("repository", template).unwrap();

    let data = json!({
        "entity_name": name,
        "entity_name_snake": snake_name,
        "entity_name_plural": plural_name,
    });

    let output = handlebars.render("repository", &data).unwrap();

    let path = format!("src/repository/{}_repository.rs", snake_name);
    if Path::new(&path).exists() {
        eprintln!("{} File already exists: {}", WARN, path);
        return;
    }

    fs::write(&path, output).expect("Failed to write file");
    println!("  {} Created {}", FILE, style(&path).green());

    println!();
    println!("  Don't forget to add to src/repository/mod.rs:");
    println!("    mod {}_repository;", snake_name);
    println!("    pub use {}_repository::{}Repository;", snake_name, name);
}

/// Generate a GraphQL resolver.
pub fn resolver(name: &str) {
    println!(
        "{} Generating resolver: {}",
        CHECK,
        style(name).cyan().bold()
    );

    let snake_name = to_snake_case(name);
    let plural_name = to_plural(&snake_name);

    let template = match get_template("resolver") {
        Some(t) => t,
        None => {
            eprintln!("{} Resolver template not found", WARN);
            return;
        }
    };

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("resolver", template).unwrap();

    let data = json!({
        "entity_name": name,
        "entity_name_snake": snake_name,
        "entity_name_plural": plural_name,
    });

    let output = handlebars.render("resolver", &data).unwrap();

    let path = format!("src/graphql/{}.rs", snake_name);
    if Path::new(&path).exists() {
        eprintln!("{} File already exists: {}", WARN, path);
        return;
    }

    fs::write(&path, output).expect("Failed to write file");
    println!("  {} Created {}", FILE, style(&path).green());

    println!();
    println!("  Don't forget to:");
    println!("    1. Add to src/graphql/mod.rs: mod {};", snake_name);
    println!("    2. Add to src/graphql/types.rs: pub use {}::{}Dto;", snake_name, name);
    println!("    3. Merge resolvers in query.rs and mutation.rs");
}

/// Generate all (entity, service, repository, resolver).
pub fn all(name: &str) {
    println!(
        "\n{} Generating all for: {}\n",
        CHECK,
        style(name).cyan().bold()
    );

    entity(name);
    println!();

    service(name);
    println!();

    repository(name);
    println!();

    resolver(name);

    println!();
    println!("{}", style("Generation complete!").green().bold());
}
