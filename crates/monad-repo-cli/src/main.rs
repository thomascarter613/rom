use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use monad_repo_artifacts::{
    read_graph_json, read_index_json, write_repository_artifacts, ArtifactPaths,
};
use monad_repo_discovery::discover_repository;
use monad_repo_model::{
    Graph, RelationshipType, Resource, ResourceId, ResourceIndex, ResourceKind,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "monad")]
#[command(about = "Monad OS repository control plane exploration CLI")]
#[command(version)]
struct Cli {
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Repo {
        #[command(subcommand)]
        command: RepoCommand,
    },
    Resources {
        #[command(subcommand)]
        command: ResourcesCommand,
    },
    Resource {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    Graph {
        #[arg(long, default_value = "text")]
        format: GraphFormat,
    },
}

#[derive(Debug, Subcommand)]
enum RepoCommand {
    Discover {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ResourcesCommand {
    List {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ResourceCommand {
    Show {
        id: String,

        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum GraphFormat {
    Text,
    Json,
    Mermaid,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Repo { command } => match command {
            RepoCommand::Discover { json } => repo_discover(&cli.root, json),
        },
        Commands::Resources { command } => match command {
            ResourcesCommand::List { json } => resources_list(&cli.root, json),
        },
        Commands::Resource { command } => match command {
            ResourceCommand::Show { id, json } => resource_show(&cli.root, &id, json),
        },
        Commands::Graph { format } => graph(&cli.root, format),
    }
}

fn repo_discover(root: &Path, json: bool) -> Result<()> {
    let index = discover_repository(root)
        .with_context(|| format!("failed to discover repository at `{}`", root.display()))?;

    let written = write_repository_artifacts(root, &index)
        .with_context(|| format!("failed to write artifacts under `{}`", root.display()))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&index)?);
        return Ok(());
    }

    println!("Repository discovery complete.");
    println!();
    println!("Resources: {}", index.resources.len());
    println!("Relationships: {}", index.relationships.len());
    println!("Warnings: {}", index.warnings.len());
    println!();
    println!("Wrote:");
    println!("- {}", display_path(&written.index_json));
    println!("- {}", display_path(&written.graph_json));

    if !index.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &index.warnings {
            println!("- {warning}");
        }
    }

    Ok(())
}

fn resources_list(root: &Path, json: bool) -> Result<()> {
    let index = load_index(root)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&index.resources)?);
        return Ok(());
    }

    if index.resources.is_empty() {
        println!("No resources found.");
        println!("Run `monad repo discover` first.");
        return Ok(());
    }

    let mut resources = index.resources.clone();
    resources.sort_by(|left, right| {
        left.kind
            .as_str()
            .cmp(right.kind.as_str())
            .then(left.name.cmp(&right.name))
            .then(left.path.cmp(&right.path))
    });

    print_resource_table(&resources);

    Ok(())
}

fn resource_show(root: &Path, id: &str, json: bool) -> Result<()> {
    let index = load_index(root)?;
    let requested = ResourceId::new(id);

    let Some(resource) = index
        .resources
        .iter()
        .find(|resource| resource.id == requested)
    else {
        anyhow::bail!("resource not found: {id}");
    };

    if json {
        println!("{}", serde_json::to_string_pretty(resource)?);
        return Ok(());
    }

    print_resource_details(resource, &index);

    Ok(())
}

fn graph(root: &Path, format: GraphFormat) -> Result<()> {
    let paths = ArtifactPaths::for_root(root);
    let graph = read_graph_json(&paths.graph_json)
        .with_context(|| format!("failed to read `{}`", display_path(&paths.graph_json)))?;

    match format {
        GraphFormat::Text => print_graph_text(&graph),
        GraphFormat::Json => println!("{}", serde_json::to_string_pretty(&graph)?),
        GraphFormat::Mermaid => print_graph_mermaid(&graph),
    }

    Ok(())
}

fn load_index(root: &Path) -> Result<ResourceIndex> {
    let paths = ArtifactPaths::for_root(root);

    read_index_json(&paths.index_json).with_context(|| {
        format!(
            "failed to read `{}`. Run `monad repo discover` first.",
            display_path(&paths.index_json)
        )
    })
}

fn print_resource_table(resources: &[Resource]) {
    println!("{:<32}  {:<16}  {:<24}  {}", "ID", "KIND", "NAME", "PATH");
    println!("{:<32}  {:<16}  {:<24}  {}", "--", "----", "----", "----");

    for resource in resources {
        println!(
            "{:<32}  {:<16}  {:<24}  {}",
            resource.id,
            resource.kind.as_str(),
            resource.name,
            resource.path
        );
    }
}

fn print_resource_details(resource: &Resource, index: &ResourceIndex) {
    println!("ID: {}", resource.id);
    println!("Kind: {}", resource.kind.as_str());
    println!("Name: {}", resource.name);
    println!("Path: {}", resource.path);
    println!("Lifecycle: {:?}", resource.lifecycle);
    println!("Source: {:?}", resource.source);

    if !resource.owners.is_empty() {
        println!("Owners: {}", resource.owners.join(", "));
    }

    if !resource.metadata.is_empty() {
        println!();
        println!("Metadata:");
        print_map(&resource.metadata);
    }

    if !resource.spec.is_empty() {
        println!();
        println!("Spec:");
        print_map(&resource.spec);
    }

    if !resource.actions.is_empty() {
        println!();
        println!("Actions:");
        print_map(&resource.actions);
    }

    let outgoing = index
        .relationships
        .iter()
        .filter(|relationship| relationship.from == resource.id)
        .collect::<Vec<_>>();

    let incoming = index
        .relationships
        .iter()
        .filter(|relationship| relationship.to == resource.id)
        .collect::<Vec<_>>();

    if !outgoing.is_empty() {
        println!();
        println!("Outgoing relationships:");
        for relationship in outgoing {
            println!(
                "- {} {}",
                relationship.relationship_type.as_str(),
                relationship.to
            );
        }
    }

    if !incoming.is_empty() {
        println!();
        println!("Incoming relationships:");
        for relationship in incoming {
            println!(
                "- {} {}",
                relationship.relationship_type.as_str(),
                relationship.from
            );
        }
    }
}

fn print_map(map: &BTreeMap<String, String>) {
    for (key, value) in map {
        println!("- {key}: {value}");
    }
}

fn print_graph_text(graph: &Graph) {
    println!("Nodes: {}", graph.nodes.len());
    println!("Edges: {}", graph.edges.len());

    if !graph.nodes.is_empty() {
        println!();
        println!("Nodes:");
        for node in &graph.nodes {
            println!("- {} [{}] {}", node.id, node.kind.as_str(), node.path);
        }
    }

    if !graph.edges.is_empty() {
        println!();
        println!("Edges:");
        for edge in &graph.edges {
            println!(
                "- {} {} {}",
                edge.from,
                edge.relationship_type.as_str(),
                edge.to
            );
        }
    }
}

fn print_graph_mermaid(graph: &Graph) {
    println!("graph TD");

    for node in &graph.nodes {
        println!(
            "  {}[\"{}<br/>{}\"]",
            mermaid_id(&node.id),
            escape_mermaid_label(&node.id.to_string()),
            escape_mermaid_label(&node.path)
        );
    }

    for edge in &graph.edges {
        println!(
            "  {} -->|{}| {}",
            mermaid_id(&edge.from),
            edge.relationship_type.as_str(),
            mermaid_id(&edge.to)
        );
    }
}

fn mermaid_id(id: &ResourceId) -> String {
    id.as_str()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn escape_mermaid_label(value: &str) -> String {
    value.replace('"', "&quot;")
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[allow(dead_code)]
fn relationship_type_label(relationship_type: RelationshipType) -> &'static str {
    relationship_type.as_str()
}

#[allow(dead_code)]
fn resource_kind_label(kind: ResourceKind) -> &'static str {
    kind.as_str()
}
