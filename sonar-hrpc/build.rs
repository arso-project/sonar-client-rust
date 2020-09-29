fn main() {
    let mut config = hrpc_build::Config::new();

    #[cfg(feature = "serde")]
    // config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    serde(&mut config, "Record");
    serde(&mut config, "PullResponse");
    serde(&mut config, "QueryResponse");
    config.type_attribute(".sonar.Link", "#[derive(serde::Serialize)]");
    // config.type_attribute(
    //     ".sonar.Record",
    //     "#[derive(serde::Serialize, serde::Deserialize)]",
    // );
    // config.type_attribute(
    //     ".sonar.PullResponse",
    //     "#[derive(serde::Serialize, serde::Deserialize)]",
    // );
    config.type_attribute(".sonar.Record", "#[serde(default)]");
    // #[cfg(feature = "serde")]
    // config.field_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.field_attribute(
        ".sonar.Record.key",
        "#[serde(serialize_with = \"crate::as_hex\", deserialize_with = \"crate::from_hex\")]",
    );
    config.field_attribute(
        ".sonar.Record.timestamp",
        "#[serde(deserialize_with = \"crate::u32_from_integer\")]",
    );
    config.extern_path(".sonar.Json", "crate::Json");
    config
        .compile_protos(&["src/schema.proto"], &["src"])
        .unwrap();
}

fn serde(config: &mut hrpc_build::Config, name: &str) {
    config.type_attribute(
        format!(".sonar.{}", name),
        "#[derive(serde::Serialize, serde::Deserialize)]",
    );
}
