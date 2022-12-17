use cfd_raytrace::Opd;
use serde_generate::SourceInstaller;
use serde_reflection::{Tracer, TracerConfig};

fn main() {
    let mut tracer = Tracer::new(TracerConfig::default());

    // Trace the desired top-level type(s).
    tracer.trace_simple_type::<Opd>().unwrap();

    // Obtain the registry of Serde formats and serialize it in YAML (for instance).
    let registry = tracer.registry().unwrap();
    let data = serde_yaml::to_string(&registry).unwrap();
    println!("{data}");

    // Create Python class definitions.
    let mut source = Vec::new();
    let config = serde_generate::CodeGeneratorConfig::new("opd".to_string())
        .with_encodings(vec![serde_generate::Encoding::Bincode]);
    let generator = serde_generate::python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    let path = Path::new("opdpy");
    let install = serde_generate::python3::Installer::new(path.to_path_buf(), None);
    install.install_module(&config, &registry).unwrap();
    install.install_bincode_runtime().unwrap();
    install.install_serde_runtime().unwrap();
}
