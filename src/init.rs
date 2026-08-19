use std::fs;
use std::path::Path;

/// Scaffold a new Euglena project in a directory named `name`.
pub fn run(name: &str) {
    let project = Path::new(name);

    if project.exists() {
        eprintln!("euglena: directory '{}' already exists", name);
        std::process::exit(1);
    }

    // Create directory layout
    fs::create_dir_all(project.join("src")).expect("failed to create src/");
    fs::create_dir_all(project.join("tests")).expect("failed to create tests/");

    // Write manifest.json — the single source of truth for cell name + organelles.
    fs::write(project.join("manifest.json"), manifest_template(name))
        .expect("failed to write manifest.json");

    // Write a starter nucleus gene.
    fs::write(
        project.join("src/nucleus.gene.code"),
        nucleus_gene_template(),
    )
    .expect("failed to write src/nucleus.gene.code");

    println!("Created Euglena app '{}'", name);
    println!();
    println!(
        "  {}/manifest.json          <- cell name and organelles",
        name
    );
    println!(
        "  {}/src/nucleus.gene.code  <- boot gene (auto-detected)",
        name
    );
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  euglena run   (needs a `code` interpreter on PATH — e.g. `cdlvsm install code`)");
    println!();
    println!("Add more genes as src/*.gene.code — euglena-cli links them automatically.");
    println!("Shared organelles are resolved from ../euglena-organelles and parent folders.");
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

fn manifest_template(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "title": "{name}",
  "description": "A new Euglena application.",
  "organelles": {{}}
}}
"#,
        name = name
    )
}

fn nucleus_gene_template() -> &'static str {
    r#"-> Nucleus gene — core lifecycle handler.
->
-> This gene is auto-linked by euglena-cli because it ends in `.gene.code`.
-> Delete or rename to `.code` if you do not want it auto-included.

EuglenaHasBeenBorn = Particle ∩ { _class ∈ "EuglenaHasBeenBorn", cell_name ∈ String }

booted = false

EuglenaHasBeenBorn => {
    booted = true
    return EuglenaHasBeenBorn { cell_name = cell_name }
}
"#
}
