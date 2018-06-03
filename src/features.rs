use cargo::core::summary::Summary;

/// Kind of feature
#[derive(Debug)]
pub enum Feature {
    /// Its dependencies are other crates.
    Dependency(String),

    /// Its dependencies are exclusively other features.
    Meta(String),

    /// Its has no dependencies
    Flag(String),
}

fn dependency_names(summary: &Summary) -> Vec<String> {
    summary
        .dependencies()
        .iter()
        .map(|p| p.name().into())
        .collect()
}

pub fn discriminate_features(summary: &Summary) -> Vec<Feature> {
    let dependencies = dependency_names(summary);

    summary
        .features()
        .iter()
        .map(|(ref name, ref deps)| {
            if deps.len() == 0 {
                Feature::Flag(name.to_string())
            } else if deps.iter().all(|dep| dependencies.contains(&dep)) {
                Feature::Dependency(name.to_string())
            } else {
                Feature::Meta(name.to_string())
            }
        })
        .collect()
}
