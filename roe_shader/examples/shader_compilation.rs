fn main() -> Result<(), Box<dyn std::error::Error>> {
    let in_dir = std::path::PathBuf::from("roe_shader/examples/glsl/");
    let out_dir = std::path::PathBuf::from("roe_shader/examples/gen/spirv/");
    roe_shader::compile_shaders_into_spirv(in_dir, out_dir)?;
    Ok(())
}
